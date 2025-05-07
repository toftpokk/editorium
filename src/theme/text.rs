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

pub fn primary(theme: &MyTheme) -> Style {
    style(Some(theme.text))
}

fn style(fg: Option<Color>) -> Style {
    Style {
        color: fg,
    }
}
