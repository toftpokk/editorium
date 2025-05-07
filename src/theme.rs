use iced::{Border, Color, Shadow, daemon::DefaultStyle, widget};

mod button;
mod container;
mod menu;
mod pane_grid;
mod pick_list;
mod scrollable;
mod tab_bar;
mod text;

// iced::Theme requires a default
pub struct MyTheme {
    text: Color,
    background: Color,
    background_light: Color,
    window_background: Color,
}

impl Default for MyTheme {
    fn default() -> Self {
        Self {
            text: Color::from_rgb(1.0, 1.0, 1.0),
            background: Color::from_rgb(0.271, 0.271, 0.271),
            background_light: Color::from_rgb(0.35, 0.35, 0.35),
            window_background: Color::from_rgb(0.271, 0.271, 0.271),
        }
    }
}

// iced::Theme requires a default style
impl DefaultStyle for MyTheme {
    fn default_style(&self) -> iced::daemon::Appearance {
        iced::daemon::Appearance {
            background_color: self.window_background,
            text_color: self.text,
        }
    }
}
