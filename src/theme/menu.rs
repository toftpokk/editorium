use super::MyTheme;
use iced::{
    Border, Color,
    widget::container,
    widget::overlay::menu::{Catalog, Style, StyleFn},
};

impl Catalog for MyTheme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style {
        class(self)
    }
}

pub fn primary(theme: &MyTheme) -> Style {
    let fg = Color::from_rgb(0.0, 1.0, 0.0);

    style(fg, fg, fg)
}

fn style(fg: Color, bg: Color, bg_hover: Color) -> Style {
    Style {
        background: iced::Background::Color(bg),
        border: Border::default(),
        text_color: fg,
        selected_text_color: fg,
        selected_background: iced::Background::Color(bg),
    }
}
