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
    style(
        theme.text,
        theme.background,
        theme.text_inverse,
        theme.background_accent,
    )
}

fn style(fg: Color, bg: Color, fg_selected: Color, bg_selected: Color) -> Style {
    Style {
        text_color: fg,
        background: iced::Background::Color(bg),
        selected_text_color: fg_selected,
        selected_background: iced::Background::Color(bg_selected),
        border: Border::default(),
    }
}
