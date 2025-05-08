use std::borrow::Cow;

use iced::Font;
use iced_aw::iced_fonts;

// family name ref: https://github.com/danielmbomfim/iced_font_awesome/
pub const ICON_REGULAR: Font = Font {
    family: iced::font::Family::Name("Font Awesome 6 Free"),
    ..iced::Font::DEFAULT
};

pub const ICON_SOLID: Font = Font {
    family: iced::font::Family::Name("Font Awesome 6 Free"),
    weight: iced::font::Weight::Black,
    ..iced::Font::DEFAULT
};

// loads static copy-on-write fonts
// copied from halloy
pub fn load() -> Vec<Cow<'static, [u8]>> {
    vec![
        include_bytes!("../fonts/font-awesome.otf")
            .as_slice()
            .into(),
        include_bytes!("../fonts/font-awesome-solid.otf")
            .as_slice()
            .into(),
        iced_fonts::REQUIRED_FONT_BYTES.into(),
    ]
}

pub fn arrow_dowwn() -> char {
    '\u{f0d7}'
}

pub fn arrow_left() -> char {
    '\u{f0da}'
}

pub fn caret_down() -> char {
    '\u{f107}'
}

pub fn caret_right() -> char {
    '\u{f105}'
}
