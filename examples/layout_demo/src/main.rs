#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use alanthinker_dynamic_get_field_macro::{dynamic_method, DynamicGet};
use alanthinker_dynamic_get_field_trait::DynamicGetter;

#[allow(unused)]
use anyhow::Context as _;

use anyhow::Ok;
use async_io::Timer;

use gpui::{prelude::*, *};
use gpui_component::{
    button::Button,
    input::{self, InputState},
    progress,
    scroll::ScrollbarShow,
    Root, Theme,
};
use tracing::*;

use std::time::Duration;

use gpui_style_hot_reload::my_context_ext::*;
use gpui_style_hot_reload::my_layout_data::*;
use gpui_style_hot_reload::my_style_data::*;
use gpui_style_hot_reload::my_text_input_ext::*;

// === Main Component ===

#[derive(DynamicGet)]
struct HelloWorld {
    text: SharedString,
    text2: SharedString,
    my_input_state: Entity<InputState>,
    my_progress: f32,
    sd: MyStyleData,
    ld: serde_json::Value,
    /// We need to keep the subscriptions alive with the Example entity.
    ///
    /// So if the Example entity is dropped, the subscriptions are also dropped.
    /// This is important to avoid memory leaks.
    _subscriptions: Vec<Subscription>,
}

impl HelloWorld {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let my_input_state = cx.new(|cx| {
            let mut x = InputState::new(window, cx);
            x.set_value("Init text", window, cx);
            x
        });

        let mut _subscriptions = vec![];

        // Register Entity events
        _subscriptions.push(cx.subscribe_in(&my_input_state, window, {
            move |this, _, ev: &input::InputEvent, _window, cx| match ev {
                input::InputEvent::Change => {
                    let value = this.my_input_state.get_my_text(cx);
                    this.text = format!("Hello, {}!", value).into();
                    cx.notify()
                }
                _ => {}
            }
        }));

        // Simulate asynchronously fetching data when the window is first loaded.
        cx.my_spawn_in(window, async move |entity, window| {
            // Update entity
            entity.update_in(window, |this, window, cx| {
                this.my_input_state
                    .set_my_text("Getting data...".into(), window, cx);
            })?;

            let text = reqwest::get("https://bing.com").await?.text().await?;

            info!("Successfully fetched data, length: {} bytes", text.len());

            // Update entity
            entity.update_in(window, |this, window, cx| {
                let text2 = format!("text len: {}", text.len());
                this.my_input_state.set_my_text(text2, window, cx);
            })?;

            Ok(())
        })
        .detach();

        HelloWorld {
            text: SharedString::from("Please modify the styles.pjons file and save it; you will see the window immediately apply the latest styles."),
            text2: SharedString::from("Styles and layouts can be hot-loaded."),
            my_input_state: my_input_state,
            my_progress: 0.0,
            sd: init_style_data(cx, "styles.pjson".to_owned()),
            ld: init_layout_data(cx, "layout.pjson".to_owned()),
            _subscriptions, // Registered events must not be dropped, so store them in the global Entity.
        }
    }

    #[dynamic_method(HelloWorld)]
    fn create_some_elements(
        &self,
    ) -> Box<dyn Fn(&mut HelloWorld, &mut Context<'_, HelloWorld>) -> AnyElement + 'static> {
        Box::new({
            move |this, cx| {
                let sd = &this.sd;

                div()
                    .class("div3", sd)
                    .child(input::Input::new(&this.my_input_state).class("textinput", sd))
                    .child(input::Input::new(&this.my_input_state).class("textinput", sd))
                    .child(
                        Button::new("btn_simple_b")
                            .class("btn_simple", sd)
                            .label("btn_simple")
                            .on_click(Self::btn_simple_click2(cx)),
                    )
                    .child(progress::Progress::new().value(this.my_progress))
                    .into_any_element()
            }
        })
    }

    #[dynamic_method(HelloWorld)]
    fn btn_simple_click(
        &self,
        entity: WeakEntity<HelloWorld>,
    ) -> Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static> {
        my_listener_box(entity, |this, _event, window, cx| {
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            this.my_input_state.set_my_text(
                format!("btn_simple_click: {}", time).into(),
                window,
                cx,
            );
            Ok(())
        })
    }

    fn btn_simple_click2(
        cx: &mut Context<'_, HelloWorld>,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        cx.my_listener(|this, _event, window, cx| {
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            this.my_input_state.set_my_text(
                format!("btn_simple_click: {}", time).into(),
                window,
                cx,
            );
            Ok(())
        })
    }

    #[dynamic_method(HelloWorld)]
    fn btn1_click(
        &self,
        entity: WeakEntity<HelloWorld>,
    ) -> Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static> {
        my_listener_box(entity, |_this, _event, window, cx| {
            // Basically all async requests can be handled with this logic

            // 1. Start async task
            cx.my_spawn_in(window, async move |entity, window| {
                {
                    // Updating entity must be done inside update_in, probably because the this variable cannot cross multiple awaits. (e.g., might block the UI.)

                    // 2. Update entity
                    entity.update_in(window, |this, window, cx| {
                        this.my_input_state
                            .set_my_text("start request".into(), window, cx);

                        cx.notify();
                    })?;

                    // 3. Async resource request
                    let text = reqwest::get("https://bing.com").await?.text().await?;
                    info!("Successfully fetched data, length: {} bytes", text.len());

                    // 4. Again update entity
                    entity.update_in(window, |this, window, cx| {
                        let text2 = format!("bing response text len: {}", text.len());
                        this.my_input_state.set_my_text(text2, window, cx);

                        cx.notify();
                    })?;

                    // Later there can be more async requests and UI updates.
                    let text = reqwest::get("https://baidu.com").await?.text().await?;

                    info!("Successfully fetched data, length: {} bytes", text.len());
                    //
                    entity.update_in(window, |this, window, cx| {
                        let text2 = format!("baidu response text len: {}", text.len());
                        this.my_input_state.set_my_text(text2, window, cx);

                        cx.notify();
                    })?;

                    Ok(())
                }
            })
            .detach();
            Ok(())
        })
    }

    #[dynamic_method(HelloWorld)]
    fn btn2_click(
        &self,
        entity: WeakEntity<HelloWorld>,
    ) -> Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static> {
        my_listener_box(entity, |_this, _event, window, cx| {
            cx.my_spawn_in(window, async move |entity, window| {
                // Update entity
                entity.update_in(window, |this, window, cx| {
                    this.my_input_state
                        .set_my_text("Abc".to_owned(), window, cx);
                    this.my_progress = 5.0;
                    cx.notify();
                })?;

                let mut finish = false;
                loop {
                    // Simulate download process, call async method to prevent blocking the UI thread
                    Timer::after(Duration::from_millis(100)).await;

                    // Update entity
                    entity.update_in(window, |this, _window, cx| {
                        this.my_progress = this.my_progress + 1.0;
                        tracing::info!("my_progress={}", this.my_progress);
                        if this.my_progress >= 100.0 {
                            finish = true;
                        }

                        cx.notify();
                    })?;

                    if finish {
                        break;
                    }
                }
                Ok(())
            })
            .detach();
            Ok(())
        })
    }
}

impl SetMyStyleData for HelloWorld {
    fn set_style_data(&mut self, data: MyStyleData) {
        self.sd = data;
    }

    fn get_style_data(&self) -> &MyStyleData {
        return &self.sd;
    }
}

impl SetMyLayoutData for HelloWorld {
    fn set_layout_data(&mut self, data: serde_json::Value) {
        self.ld = data;
    }
}

impl Render for HelloWorld {
    fn render(
        &mut self,
        #[allow(unused)] window: &mut Window,
        #[allow(unused)] cx: &mut Context<Self>,
    ) -> impl IntoElement {
        //let sd = &self.sd;

        add_div_by_json(&self.ld.clone(), self, cx)
    }
}

// === Launch Application ===

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("==start==");

    unsafe {
        // Make anyhow display full error chain information
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let app = Application::new();

    app.run(|cx: &mut App| {
        gpui_component::init(cx);

        let theme = Theme::global_mut(cx);
        theme.scrollbar_show = ScrollbarShow::Always;

        let bounds = Bounds::centered(None, size(px(800.), px(800.0)), cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx: &mut App| {
                    let view = cx.new(|cx| HelloWorld::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .unwrap();
        })
        .detach();
    });
}
