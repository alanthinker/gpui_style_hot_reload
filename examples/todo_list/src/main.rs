#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use alanthinker_dynamic_get_field_macro::{DynamicGet, dynamic_method};
use alanthinker_dynamic_get_field_trait::DynamicGetter;

#[allow(unused)]
use anyhow::Context as _;

use anyhow::Ok;

use gpui::{prelude::*, *};
use gpui_component::{Root, checkbox, input::InputState};

use std::{cell::RefCell, rc::Rc};

use gpui_style_hot_reload::my_context_ext::*;
use gpui_style_hot_reload::my_layout_data::*;
use gpui_style_hot_reload::my_style_data::*;
use gpui_style_hot_reload::my_text_input_ext::*;

// === Main Component ===

struct TodoItem {
    pub id: u64,
    pub text: SharedString,
    pub done: Rc<RefCell<bool>>,
}

impl TodoItem {
    fn get_done(&self) -> bool {
        *self.done.borrow()
    }
}

#[derive(DynamicGet)]
struct TodoList {
    new_item_state: Entity<InputState>,
    todo_items: Vec<TodoItem>,
    max_id: u64,

    sd: MyStyleData,
    ld: serde_json::Value,

    _subscriptions: Vec<Subscription>,
}

impl TodoList {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let new_item_state = cx.new(|cx| {
            let mut x = InputState::new(window, cx);
            x.set_value("", window, cx);
            x
        });

        let mut _subscriptions = vec![];

        TodoList {
            new_item_state: new_item_state,
            max_id: 0,
            sd: init_style_data(cx, "styles.pjson".to_owned()),
            ld: init_layout_data(cx, "layout.pjson".to_owned()),
            _subscriptions,
            todo_items: vec![],
        }
    }

    #[dynamic_method(TodoList)]
    fn item_list_elements(
        &self,
    ) -> Box<dyn Fn(&mut TodoList, &mut Context<'_, TodoList>) -> AnyElement + 'static> {
        Box::new({
            move |this, _cx| {
                let sd = &this.sd;
                let mut ele = div();

                for item in &this.todo_items {
                    let done = item.done.clone();
                    let done2 = item.done.clone();
                    let text = item.text.clone();
                    let id = item.id;
                    ele = ele.child(
                        div().class("todo_item", sd).child(
                            checkbox::Checkbox::new(ElementId::Integer(id))
                                .class("todo_item_check", sd)
                                .label(text)
                                .on_click(move |v, _w, _c| {
                                    *done.borrow_mut() = *v;
                                })
                                .checked(*done2.borrow()),
                        ),
                    )
                }

                ele.into_any_element()
            }
        })
    }

    #[dynamic_method(TodoList)]
    fn btn_add_click(
        &self,
        entity: WeakEntity<TodoList>,
    ) -> Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static> {
        my_listener_box(entity, |this, _event, window, cx| {
            this.max_id += 1;
            this.todo_items.push(TodoItem {
                id: this.max_id,
                text: this.new_item_state.get_my_text(cx),
                done: Rc::new(RefCell::new(false)),
            });
            this.new_item_state.set_my_text("".into(), window, cx);
            cx.notify();
            Ok(())
        })
    }

    #[dynamic_method(TodoList)]
    fn btn_remove_done_click(
        &self,
        entity: WeakEntity<TodoList>,
    ) -> Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static> {
        my_listener_box(entity, |this, _event, _window, cx| {
            this.todo_items.retain(|item| !item.get_done());
            cx.notify();
            Ok(())
        })
    }
}

impl SetMyStyleData for TodoList {
    fn set_style_data(&mut self, data: MyStyleData) {
        self.sd = data;
    }

    fn get_style_data(&self) -> &MyStyleData {
        return &self.sd;
    }
}

impl SetMyLayoutData for TodoList {
    fn set_layout_data(&mut self, data: serde_json::Value) {
        self.ld = data;
    }
}

impl Render for TodoList {
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

        let bounds = Bounds::centered(None, size(px(600.), px(500.0)), cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx: &mut App| {
                    let view = cx.new(|cx| TodoList::new(window, cx));
                    cx.new(|cx| Root::new(view.into(), window, cx))
                },
            )
            .unwrap();
        })
        .detach();
    });
}
