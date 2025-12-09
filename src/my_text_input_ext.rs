use gpui::{App, Entity, SharedString, Window};
use gpui_component::input::InputState;

pub trait MyTextInputExt {
    fn get_my_text(&self, cx: &mut App) -> SharedString;
    fn set_my_text(&self, text: String, window: &mut Window, cx: &mut App);
}

impl MyTextInputExt for Entity<InputState> {
    fn get_my_text(&self, cx: &mut App) -> SharedString {
        self.read(cx).value()
    }
    fn set_my_text(&self, text: String, window: &mut Window, cx: &mut App) {
        self.update(cx, |state, cx| {
            state.set_value(text, window, cx);
        });
    }
}
