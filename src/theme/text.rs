use super::MyTheme;
use iced::{
    Color,
    widget::text::{Catalog, Style, StyleFn},
};

impl Catalog for MyTheme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        // make text obey theme of above
        Box::new(none)
    }

    fn style(&self, item: &Self::Class<'_>) -> Style {
        item(self)
    }
}

pub fn none(theme: &MyTheme) -> Style {
    style(None)
}

fn style(fg: Option<Color>) -> Style {
    Style { color: fg }
}
