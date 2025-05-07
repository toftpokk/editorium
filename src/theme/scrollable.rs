use super::MyTheme;
use iced::{
    Border, Color, Shadow,
    widget::container,
    widget::scrollable::{Catalog, Rail, Scroller, Status, Style, StyleFn},
};

impl Catalog for MyTheme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: iced::widget::scrollable::Status) -> Style {
        class(self, status)
    }
}

pub fn primary(theme: &MyTheme, status: Status) -> Style {
    let fg = Color::from_rgb(0.0, 1.0, 0.0);

    style(fg, fg, fg)
}

fn style(fg: Color, bg: Color, bg_hover: Color) -> Style {
    Style {
        container: container::Style {
            text_color: Some(Color::from_rgb(0.0, 1.0, 0.0)),
            background: Some(iced::Background::Color(bg)),
            border: Border::default(),
            shadow: Shadow::default(),
        },
        vertical_rail: Rail {
            background: Some(iced::Background::Color(bg)),
            border: Border::default(),
            scroller: Scroller {
                color: bg,
                border: Border::default(),
            },
        },
        horizontal_rail: Rail {
            background: Some(iced::Background::Color(bg)),
            border: Border::default(),
            scroller: Scroller {
                color: bg,
                border: Border::default(),
            },
        },
        gap: Some(iced::Background::Color(bg)),
    }
}
