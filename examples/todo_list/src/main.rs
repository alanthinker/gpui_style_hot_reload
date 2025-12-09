#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use alanthinker_dynamic_get_field_macro::{dynamic_method, DynamicGet};
use alanthinker_dynamic_get_field_trait::DynamicGetter;

#[allow(unused)]
use anyhow::Context as _;

use anyhow::Ok;

use gpui::{prelude::*, *};
use gpui_component::StyledExt;
use gpui_component::{checkbox, input::InputState, Root};

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
    sort_by_name: bool,
    hide_done_items: bool,
}

impl TodoList {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let new_item_state = cx.new(|cx| {
            let mut x = InputState::new(window, cx);
            x.set_value("", window, cx);
            x
        });

        let mut _subscriptions = vec![];

        let mut todo_items = vec![];
        todo_items.push(TodoItem {
            id: 0,
            text: "Buy milk".into(),
            done: Rc::new(RefCell::new(false)),
        });
        todo_items.push(TodoItem {
            id: 1,
            text: "Buy eggs".into(),
            done: Rc::new(RefCell::new(true)),
        });
        todo_items.push(TodoItem {
            id: 2,
            text: "Buy bread".into(),
            done: Rc::new(RefCell::new(false)),
        });
        todo_items.push(TodoItem {
            id: 3,
            text: "Buy butter".into(),
            done: Rc::new(RefCell::new(false)),
        });

        TodoList {
            new_item_state: new_item_state,
            max_id: 0,
            sd: init_style_data(cx, "styles.pjson".to_owned()),
            ld: init_layout_data(cx, "layout.pjson".to_owned()),
            _subscriptions,
            todo_items,
            sort_by_name: false,
            hide_done_items: false,
        }
    }

    fn sort_items(&mut self) {
        if self.sort_by_name {
            self.todo_items.sort_by(|a, b| {
                a.text
                    .to_lowercase()
                    .cmp(&b.text.to_lowercase())
                    .then_with(|| a.text.cmp(&b.text))
            });
        } else {
            self.todo_items.sort_by_key(|x| x.id);
        }
    }

    #[dynamic_method(TodoList)]
    fn item_list_elements(
        &self,
    ) -> Box<dyn Fn(&mut TodoList, &mut Context<'_, TodoList>) -> AnyElement + 'static> {
        Box::new({
            move |this, cx| {
                let sd = &this.sd;

                let mut ele = div().child(
                    div()
                        .class("row", sd)
                        .paddings(Edges::all(px(10.)))
                        .child(
                            checkbox::Checkbox::new("chkSort")
                                .class("todo_item_check", sd)
                                .label("Sort by name")
                                .checked(this.sort_by_name)
                                .on_click(cx.listener(|this, new_checked, _, _| {
                                    this.sort_by_name = *new_checked;
                                    this.sort_items();
                                })),
                        )
                        .child(
                            div().class("row", sd).child(
                                checkbox::Checkbox::new("chkHide")
                                    .class("todo_item_check", sd)
                                    .label("Hide done items")
                                    .checked(this.hide_done_items)
                                    .on_click(cx.listener(|this, new_checked, _, _| {
                                        this.hide_done_items = *new_checked;
                                    })),
                            ),
                        ),
                );

                for item in &this.todo_items {
                    if this.hide_done_items && item.get_done() {
                        continue;
                    }
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

            this.sort_items();

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
