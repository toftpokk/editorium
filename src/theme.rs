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
#[derive(Default)]
pub struct MyTheme {}

// iced::Theme requires a default style
impl DefaultStyle for MyTheme {
    fn default_style(&self) -> iced::daemon::Appearance {
        iced::daemon::Appearance {
            background_color: Color::from_rgb(0.0, 1.0, 0.0),
            text_color: Color::from_rgb(0.0, 1.0, 0.0),
        }
    }
}
