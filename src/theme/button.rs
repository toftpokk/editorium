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
    let fg = Color::from_rgb(0.0, 1.0, 0.0);

    style(fg, fg, fg, status)
}

fn style(fg: Color, bg: Color, bg_hover: Color, status: Status) -> Style {
    // match status {
    //     iced::widget::Status::Active => todo!(),
    //     iced::widget::Status::Hovered => todo!(),
    //     iced::widget::Status::Pressed => todo!(),
    //     iced::widget::Status::Disabled => todo!(),
    // }
    Style {
        background: Some(iced::Background::Color(bg)),
        text_color: fg,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}
