use super::MyTheme;
use iced::{
    Border, Color,
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
    style(
        status,
        theme.text,
        theme.text,
        theme.background,
        theme.text_inverse,
        theme.background_accent,
    )
}

fn style(
    status: Status,
    fg: Color,
    fg_placeholder: Color,
    bg: Color,
    fg_hover: Color,
    bg_hover: Color,
) -> Style {
    match status {
        Status::Hovered => Style {
            text_color: fg_hover,
            placeholder_color: fg_placeholder,
            handle_color: Color::from_rgb(0.0, 1.0, 0.0),
            background: iced::Background::Color(bg_hover),
            border: Border::default(),
        },
        _ => Style {
            text_color: fg,
            placeholder_color: fg_placeholder,
            handle_color: Color::from_rgb(0.0, 1.0, 0.0),
            background: iced::Background::Color(bg),
            border: Border::default(),
        },
    }
}
