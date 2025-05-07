use super::MyTheme;
use iced::{
    Color,
    widget::text::{Catalog, Style, StyleFn},
};

impl Catalog for MyTheme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, item: &Self::Class<'_>) -> Style {
        item(self)
    }
}

pub fn none(theme: &MyTheme) -> Style {
    Style { color: None }
}

pub fn primary(theme: &MyTheme) -> Style {
    let fg = Color::from_rgb(0.0, 1.0, 0.0);

    style(fg, fg, fg)
}

fn style(fg: Color, bg: Color, bg_hover: Color) -> Style {
    Style {
        color: Some(Color::from_rgb(0.0, 1.0, 0.0)),
    }
}
