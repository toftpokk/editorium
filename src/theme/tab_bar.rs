use super::MyTheme;
use iced::{Border, Color, border::Radius};
use iced_aw::tab_bar::{Catalog, Status, Style, StyleFn};

impl Catalog for MyTheme {
    type Class<'a> = StyleFn<'a, Self, Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(
        &self,
        class: &Self::Class<'_>,
        status: iced_aw::card::Status,
    ) -> iced_aw::tab_bar::Style {
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
            text_color: fg_hover,
            border_color: Some(fg),
            border_width: 1.0,
            tab_label_background: iced::Background::Color(bg_hover),
            tab_label_border_color: fg,
            tab_label_border_width: 1.0,
            icon_color: fg,
            icon_background: Some(iced::Background::Color(bg)),
            icon_border_radius: Radius::default(),
            background: None,
        },
        _ => Style {
            text_color: fg,
            border_color: Some(fg),
            border_width: 1.0,
            tab_label_background: iced::Background::Color(bg),
            tab_label_border_color: fg,
            tab_label_border_width: 1.0,
            icon_color: fg,
            icon_background: Some(iced::Background::Color(bg)),
            icon_border_radius: Radius::default(),
            background: None,
        },
    }
}
