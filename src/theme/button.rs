use super::MyTheme;
use iced::{Border, Color, Shadow, widget::button};

// when theming for a button, use button's styling catalog
impl button::Catalog for MyTheme {
    // define styling function as boxed(some_fn)
    type Class<'a> = button::StyleFn<'a, Self>;

    // define the default boxed(some_fn)
    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    // what to do when styling
    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        class(self, status)
    }
}

pub fn primary(
    theme: &MyTheme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let fg = Color::from_rgb(0.0, 1.0, 0.0);

    style(fg, fg, fg, status)
}

fn style(
    fg: Color,
    bg: Color,
    bg_hover: Color,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    // match status {
    //     iced::widget::button::Status::Active => todo!(),
    //     iced::widget::button::Status::Hovered => todo!(),
    //     iced::widget::button::Status::Pressed => todo!(),
    //     iced::widget::button::Status::Disabled => todo!(),
    // }
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: fg,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}
