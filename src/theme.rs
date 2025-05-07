use iced::{Border, Color, Shadow, daemon::DefaultStyle, widget};

mod button;

// iced::Theme requires a default
#[derive(Default)]
pub struct MyTheme {}

// iced::Theme requires a default style
impl DefaultStyle for MyTheme {
    fn default_style(&self) -> iced::daemon::Appearance {
        iced::daemon::Appearance {
            background_color: Color::from_rgb(0.0, 1.0, 0.0),
            text_color: Color::from_rgb(0.0, 1.0, 0.0),
        }
    }
}

impl widget::text::Catalog for MyTheme {
    type Class<'a> = widget::text::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(text_primary)
    }

    fn style(&self, class: &Self::Class<'_>) -> widget::text::Style {
        class(self)
    }
}

fn text(fg: Color, bg: Color, bg_hover: Color) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(Color::from_rgb(0.0, 1.0, 0.0)),
    }
}

pub fn text_primary(theme: &MyTheme) -> iced::widget::text::Style {
    let fg = Color::from_rgb(0.0, 1.0, 0.0);

    text(fg, fg, fg)
}
