use super::MyTheme;
use iced::Border;
use iced::widget::text_input::{Catalog, Status, Style, StyleFn};
use iced::{Color, border::Radius};

impl Catalog for MyTheme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn primary(theme: &MyTheme, status: Status) -> Style {
    style(
        status,
        theme.text,
        theme.background,
        theme.text_inverse,
        theme.background_accent,
    )
}

fn style(status: Status, fg: Color, bg: Color, fg_hover: Color, bg_hover: Color) -> Style {
    match status {
        Status::Active | Status::Hovered => Style {
            border: Border {
                color: fg,
                width: 0.0,
                radius: Radius::default(),
            },
            icon: fg,
            placeholder: fg,
            value: fg,
            selection: fg,
            background: iced::Background::Color(bg),
        },
        _ => Style {
            border: Border {
                color: fg,
                width: 0.0,
                radius: Radius::default(),
            },
            icon: fg,
            placeholder: fg,
            value: fg,
            selection: fg,
            background: iced::Background::Color(bg),
        },
    }
}
