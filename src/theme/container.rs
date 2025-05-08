use super::MyTheme;
use iced::{
    Border, Color, Shadow,
    widget::container::{Catalog, Style, StyleFn},
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
        text_color: Some(fg),
        background: Some(iced::Background::Color(bg)),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}
