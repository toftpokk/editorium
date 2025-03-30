use std::collections::HashMap;

use iced::keyboard::{Key, Modifiers};

use crate::Message;

#[derive(Hash, PartialEq, Eq)]
pub struct KeyBind {
    pub modifiers: Modifiers,
    pub key: Key,
}

pub fn default() -> HashMap<KeyBind, Message> {
    let mut key_bind = HashMap::new();

    key_bind.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("s".into()),
        },
        Message::SaveFile,
    );

    key_bind
}
