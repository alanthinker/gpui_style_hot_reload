#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(unused)]
use anyhow::Context as _;

use gpui::{prelude::*, *};
use gpui_component::{
    button::Button,
    input::{self, InputState},
    label, progress, Root,
};

mod my_style_data;
use my_style_data::*;

struct HelloWorld {
    text: SharedString,
    my_input_state: Entity<InputState>,
    my_progress: f32,
    sd: MyStyleData,
}

impl HelloWorld {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let my_input_state = cx.new(|cx| {
            let mut x = InputState::new(window, cx);
            x.set_value("Initi text", window, cx);
            x
        });

        HelloWorld {
            text: SharedString::from(
                "Please modify the styles in the styles.json file and save it; you'll see the window immediately apply the latest styles.",
            ),
            my_input_state: my_input_state,
            my_progress: 0.0,
            sd: init_style_data(cx, "styles.pjson".to_owned()),
        }
    }
}

impl SetMyStyleData for HelloWorld {
    fn set_style_data(&mut self, data: MyStyleData) {
        self.sd = data;
    }
}

impl Render for HelloWorld {
    fn render(
        &mut self,
        #[allow(unused)] window: &mut Window,
        #[allow(unused)] cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let sd = &self.sd;

        div()
            .class("div1", sd)
            .child(
                div()
                    .class("div2", sd)
                    .child(label::Label::new(self.text.clone()).class("label", sd))
                    .child(Button::new("btn1").class("btn1", sd).label("btn1")),
            )
            .child(
                div()
                    .class("div3", sd)
                    .child(input::TextInput::new(&self.my_input_state).class("textinput", sd))
                    .child(
                        Button::new("btn_simple")
                            .class("btn_simple", sd)
                            .label("btn_simple"),
                    )
                    .child(Button::new("btn2").class("btn2", sd).label("btn2"))
                    .child(progress::Progress::new().value(self.my_progress)),
            )
            .child(
                div()
                    .class("div4", sd)
                    .child(div().class("box box1", sd))
                    .child(div().class("box box2", sd))
                    .child(div().class("box box3", sd))
                    .child(div().class("box box4", sd))
                    .child(div().class("box box5", sd))
                    .child(div().class("box box6", sd)),
            )
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("==start==");

    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let app = Application::new();

    app.run(|cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(800.), px(800.0)), cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx: &mut App| {
                    let view = cx.new(|cx| HelloWorld::new(window, cx));
                    cx.new(|cx| Root::new(view.into(), window, cx))
                },
            )
            .unwrap();
        })
        .detach();
    });
}
