use super::MyTheme;
use iced::{
    Border, Color, Shadow,
    widget::pick_list::{Catalog, Status, Style, StyleFn},
};

impl Catalog for MyTheme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn primary(theme: &MyTheme, status: Status) -> Style {
    let fg = Color::from_rgb(0.0, 1.0, 0.0);

    style(fg, fg, fg)
}

fn style(fg: Color, bg: Color, bg_hover: Color) -> Style {
    Style {
        text_color: Color::from_rgb(0.0, 1.0, 0.0),
        placeholder_color: Color::from_rgb(0.0, 1.0, 0.0),
        handle_color: Color::from_rgb(0.0, 1.0, 0.0),
        background: iced::Background::Color(bg),
        border: Border::default(),
    }
}
