use anyhow::{Context as _, Result};
use futures::{SinkExt, StreamExt};
use gpui_component::{
    button,
    input::{self, InputState},
    label,
};
use std::{any::Any, path::PathBuf, thread};

use alanthinker_dynamic_get_field_trait::DynamicGetter;
use gpui::*;

use crate::{
    my_context_ext::MyContextExt,
    my_style_data::{SetMyStyleData, StylableElement},
};

pub fn load_layout(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let content = std::fs::read(path.as_path())?;
    let json = pjson::PJsonReader::from_pjson(&content);
    let json = String::from_utf8_lossy(&json).to_string();
    let json_value: serde_json::Value = serde_json::from_str(&json)?;
    Ok(json_value)
}

pub trait SetMyLayoutData {
    fn set_layout_data(&mut self, data: serde_json::Value);
}

pub fn init_layout_data<T: 'static>(cx: &mut Context<T>, layout_path: String) -> serde_json::Value
where
    T: SetMyLayoutData,
{
    // If style_path file does not exist, replace with ../../{style_path}
    let style_path = if std::path::Path::new(&layout_path).exists() {
        layout_path
    } else {
        format!("../../{}", layout_path)
    };

    let style_path2 = style_path.clone();

    let layout_data: serde_json::Value = match load_layout(&PathBuf::from(style_path)) {
        Ok(data) => {
            tracing::info!("✅ Layout reloaded successfully.");
            data
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            thread::sleep(std::time::Duration::from_secs(1));
            panic!("Failed to load Layout.")
        }
    };

    watch_layout_data(cx, style_path2);

    layout_data
}

fn watch_layout_data<T: 'static>(cx: &mut Context<T>, layout_path: String)
where
    T: SetMyLayoutData,
{
    let (th_sender, mut th_receiver) = futures::channel::mpsc::channel::<serde_json::Value>(100); // std::sync::mpsc::channel::<serde_json::Value>();

    let be = cx.background_executor().clone();

    std::thread::spawn(|| {
        if let Err(e) = run_watcher(PathBuf::from(layout_path), be, th_sender) {
            tracing::error!("File watcher failed: {:?}", e);
        }
    });

    cx.my_spawn(async move |entity, cx| {
        loop {
            match th_receiver.next().await {
                Some(r) => {
                    tracing::info!("Refresh the window");

                    entity
                        .upgrade()
                        .context("entity upgrade fail.")?
                        .update(cx, |this, cx| {
                            this.set_layout_data(r);
                            cx.notify(); // Must notify UI to update
                        })?;
                }
                None => {}
            }
        }

        #[allow(unreachable_code)]
        Ok(())
    })
    .detach();
}

// File watcher (hot reload)
fn run_watcher(
    path: PathBuf,
    be: BackgroundExecutor,
    mut sender: futures::channel::mpsc::Sender<serde_json::Value>,
) -> Result<()> {
    use notify::{recommended_watcher, RecursiveMode, Watcher};

    let path2 = path.clone();
    let mut watcher =
        recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                if event
                    .paths
                    .iter()
                    .any(|path| path.canonicalize().ok() == path.canonicalize().ok())
                {
                    match load_layout(&path) {
                        Ok(style_data) => {
                            tracing::info!("✅ Styles reloaded successfully.");
                            be.block(async {
                                let _ = sender.send(style_data).await;
                            });
                        }
                        Err(e) => {
                            tracing::error!("{:?}", e);
                        }
                    }
                }
            }
            Err(e) => tracing::error!("Watch error: {:?}", e),
        })?;

    watcher.watch(&path2, RecursiveMode::NonRecursive)?;
    loop {
        std::thread::park(); // keep alive, prevent watcher from being dropped.
    }
}

pub fn add_div_by_json<E>(value: &serde_json::Value, e: &mut E, cx: &mut Context<E>) -> AnyElement
where
    E: DynamicGetter + SetMyStyleData + Any + 'static,
{
    let mut ele = div();

    ele = set_attributes(ele, value, e);
    ele = set_children(ele, value, e, cx);

    ele.into_any_element()
}

pub fn add_button_by_json<E>(value: &serde_json::Value, e: &mut E, cx: &Context<E>) -> AnyElement
where
    E: DynamicGetter + SetMyStyleData + Any + 'static,
{
    match value {
        serde_json::Value::Object(map) => {
            match &map.get("id").unwrap_or_default() {
                serde_json::Value::String(id) => {
                    let id = ElementId::Name(id.into());
                    let mut ele = button::Button::new(id);

                    match &map.get("label").unwrap_or_default() {
                        serde_json::Value::String(label) => {
                            ele = ele.label(label);
                        }
                        _ => {
                            tracing::error!("button's 'label' attribute must be set");
                        }
                    }

                    match &map.get("on_click").unwrap_or_default() {
                        serde_json::Value::String(on_click) => {
                            if let Some(result) =
                                alanthinker_dynamic_get_field_trait::find_method(on_click)
                            {
                                let view = cx.entity().downgrade();
                                let result = result.call(e, &[&view]);

                                match result {
                                    Some(result) => {
                                        // 👇 Attempt to downcast Box<dyn Any> into ownership of Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>
                                        match result.downcast::<Box<
                                            dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static,
                                        >>() {
                                            Ok(handler_box) => {
                                                let handler: Box<
                                                    dyn Fn(&ClickEvent, &mut Window, &mut App)
                                                        + 'static,
                                                > = *handler_box;
                                                ele = ele.on_click(handler);
                                            }
                                            Err(_e) => {
                                                tracing::error!("Failed to downcast: returned type [ about on click ]");
                                            }
                                        }
                                    }
                                    None => {
                                        tracing::error!("result is None.");
                                    }
                                }
                            } else {
                                tracing::error!("Failed to find method: {}", on_click);
                            }
                        }
                        _ => {
                            // User did not set on_click event
                        }
                    }

                    ele = set_attributes(ele, value, e);

                    return ele.into_any_element();
                }
                _ => {
                    tracing::error!("button's 'id' attribute must be set");
                }
            }
        }
        _ => {
            tracing::error!("Failed to get json Object type.");
        }
    }

    let ele = button::Button::new("");
    let ele = set_attributes(ele, value, e);

    ele.into_any_element()
}

pub fn add_label_by_json<E>(value: &serde_json::Value, e: &E) -> AnyElement
where
    E: DynamicGetter + SetMyStyleData + Any + 'static,
{
    let mut ele = None;
    match value {
        serde_json::Value::Object(map) => {
            let mut bind_ok = false;
            match &map.get("bind").unwrap_or_default() {
                serde_json::Value::String(bind) => {
                    if let Some(lable) = e.get_field(bind) {
                        if let Some(lable) = lable.downcast_ref::<SharedString>() {
                            ele.replace(label::Label::new(lable));
                            bind_ok = true;
                        } else if let Some(lable) = lable.downcast_ref::<String>() {
                            ele.replace(label::Label::new(lable));
                        } else {
                            tracing::error!("Invalid label type");
                            bind_ok = true;
                        }
                    }
                }
                _ => {
                    //
                }
            }

            if !bind_ok {
                match &map.get("label").unwrap_or_default() {
                    serde_json::Value::String(label) => {
                        ele.replace(label::Label::new(label));
                    }
                    _ => {
                        tracing::error!("label's 'bind' or 'label' must be set");
                    }
                }
            }
        }
        _ => {
            tracing::error!("Failed to get json Object type.");
        }
    }

    if ele.is_none() {
        ele.replace(label::Label::new(""));
    }

    match ele {
        Some(mut ele) => {
            ele = set_attributes(ele, value, e);
            ele.into_any_element()
        }
        None => {
            tracing::error!("Failed to create label element");
            div().into_any_element()
        }
    }
}

pub fn add_text_input_by_json<E>(value: &serde_json::Value, e: &E) -> AnyElement
where
    E: DynamicGetter + SetMyStyleData + Any + 'static,
{
    let mut ele = None;
    match value {
        serde_json::Value::Object(map) => match &map.get("bind").unwrap_or_default() {
            serde_json::Value::String(bind) => {
                if let Some(state) = e.get_field(bind) {
                    if let Some(state) = state.downcast_ref::<Entity<InputState>>() {
                        ele.replace(input::TextInput::new(state));
                    } else {
                        tracing::error!(
                            "state.downcast_ref::<Entity<InputState>> fail. bind={}",
                            bind
                        );
                    }
                } else {
                    tracing::error!("e.get_field(bind) fail. bind={}", bind);
                }
            }
            _ => {
                tracing::error!("text_input's 'bind' attribute must be set.");
            }
        },
        _ => {
            tracing::error!("Failed to get json Object type.");
        }
    }

    match ele {
        Some(mut ele) => {
            ele = set_attributes(ele, value, e);
            ele.into_any_element()
        }
        None => {
            tracing::error!("Failed to create text_input element");
            div().into_any_element()
        }
    }
}

pub fn add_fn_by_json<E>(value: &serde_json::Value, e: &mut E, cx: &mut Context<E>) -> AnyElement
where
    E: DynamicGetter + SetMyStyleData + Any + 'static,
{
    let mut ele = None;
    match value {
        serde_json::Value::Object(map) => match &map.get("name").unwrap_or_default() {
            serde_json::Value::String(name) => {
                if let Some(result) = alanthinker_dynamic_get_field_trait::find_method(name) {
                    let result = result.call(e, &[]);

                    match result {
                        Some(result) => {
                            // 👇 Attempt to downcast Box<dyn Any> into ownership of Box<dyn Fn(&mut E, &mut Context<'_, E>) -> AnyElement + 'static>
                            match result.downcast::<Box<dyn Fn(&mut E, &mut Context<'_, E>) -> AnyElement + 'static>>() {
                                Ok(child_fn) => {
                                    let child = *child_fn;
                                    ele.replace(child(e, cx));
                                }
                                Err(_e) => {
                                    tracing::error!("Failed to downcast: returned type [ about on click ]");
                                }
                            }
                        }
                        None => {
                            tracing::error!("result is None.");
                        }
                    }
                } else {
                    tracing::error!("Failed to find method: '{}'", name);
                }
            }
            _ => {
                tracing::error!("fn's 'name' attribute must be set.");
            }
        },
        _ => {
            tracing::error!("Failed to find element");
        }
    }

    match ele {
        Some(ele) => ele.into_any_element(),
        None => {
            tracing::error!("Failed to create fn element");
            div().into_any_element()
        }
    }
}

fn set_attributes<T, E>(mut ele: T, value: &serde_json::Value, e: &E) -> T
where
    T: Styled,
    E: DynamicGetter + SetMyStyleData + Any + 'static,
{
    match value {
        serde_json::Value::Object(map) => {
            match &map.get("class").unwrap_or_default() {
                serde_json::Value::String(classes) => {
                    let sd = e.get_style_data();
                    ele = ele.class(classes, &sd);
                }
                _ => {
                    // class attribute not set
                }
            }

            match map.get("style") {
                Some(styles) => {
                    let style_rule = serde_json::from_value(styles.clone());
                    match style_rule {
                        Ok(style_rule) => {
                            ele = ele.apply_style_rule(&style_rule);
                        }
                        Err(e) => {
                            tracing::error!("wrong style: {}", e);
                        }
                    }
                }
                None => {
                    // style attribute not set
                }
            }
        }
        _ => {
            tracing::error!("Failed to get json Object type.");
        }
    }

    ele
}

fn set_children<T, E>(mut ele: T, value: &serde_json::Value, e: &mut E, cx: &mut Context<E>) -> T
where
    E: DynamicGetter + SetMyStyleData + Any + 'static,
    T: Styled + ParentElement,
{
    match value {
        serde_json::Value::Object(map) => match &map.get("children").unwrap_or_default() {
            serde_json::Value::Array(children) => {
                for child in children {
                    match child {
                        serde_json::Value::Object(map) => {
                            let etype = map
                                .get("type")
                                .unwrap_or_default()
                                .as_str()
                                .unwrap_or_default();

                            match etype {
                                "div" => ele = ele.child(add_div_by_json(child, e, cx)),
                                "label" => ele = ele.child(add_label_by_json(child, e)),
                                "text_input" => ele = ele.child(add_text_input_by_json(child, e)),
                                "button" => ele = ele.child(add_button_by_json(child, e, cx)),
                                "fn" => ele = ele.child(add_fn_by_json(child, e, cx)),
                                _ => {
                                    tracing::error!("Unknown element: {}", etype);
                                }
                            }
                        }
                        serde_json::Value::String(label) => {
                            ele = ele.child(label::Label::new(label));
                        }
                        _ => {
                            tracing::error!("Failed to get json type.");
                        }
                    }
                }
            }
            _ => {
                // User did not set children
            }
        },
        _ => {
            tracing::error!("Failed to get json Object type.");
        }
    }

    ele
}
