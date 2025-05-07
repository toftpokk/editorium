use super::MyTheme;
use iced::{
    Border, Color, Shadow,
    widget::button::{Catalog, Status, Style, StyleFn},
};

// when theming for a button, use button's styling catalog
impl Catalog for MyTheme {
    // define styling function as boxed(some_fn)
    type Class<'a> = StyleFn<'a, Self>;

    // define the default boxed(some_fn)
    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    // what to do when styling
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn primary(theme: &MyTheme, status: Status) -> Style {
    style(status, theme.text, theme.background, theme.text_inverse, theme.background_accent)
}

fn style(status: Status, fg: Color, bg: Color, fg_hover: Color, bg_hover: Color) -> Style {
    match status {
         Status::Active => Style {
           background: Some(iced::Background::Color(bg)),
           text_color: fg,
           border: Border::default(),
           shadow: Shadow::default(),
         },
         Status::Hovered => Style {
           background: Some(iced::Background::Color(bg_hover)),
           text_color: fg_hover,
           border: Border::default(),
           shadow: Shadow::default(),
         },
         Status::Pressed => Style {
           background: Some(iced::Background::Color(bg_hover)),
           text_color: fg_hover,
           border: Border::default(),
           shadow: Shadow::default(),
         },
         Status::Disabled => Style {
           background: Some(iced::Background::Color(bg)),
           text_color: fg,
           border: Border::default(),
           shadow: Shadow::default(),
         },
    }
}
