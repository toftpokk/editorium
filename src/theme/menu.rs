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
    style(theme.text, theme.background)
}

fn style(fg: Color, bg: Color) -> Style {
    Style {
        text_color: fg,
        background: iced::Background::Color(bg),
        selected_text_color: fg,
        selected_background: iced::Background::Color(bg),
        border: Border::default(),
    }
}
