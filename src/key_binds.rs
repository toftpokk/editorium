use std::collections::HashMap;

use iced::keyboard::{Key, Modifiers, key};

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

    key_bind.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("w".into()),
        },
        Message::TabCloseCurrent,
    );
    key_bind.insert(
        KeyBind {
            modifiers: Modifiers::ALT,
            key: Key::Character("1".into()),
        },
        Message::TabSelected(0),
    );
    key_bind.insert(
        KeyBind {
            modifiers: Modifiers::ALT,
            key: Key::Character("2".into()),
        },
        Message::TabSelected(1),
    );
    key_bind.insert(
        KeyBind {
            modifiers: Modifiers::ALT,
            key: Key::Character("3".into()),
        },
        Message::TabSelected(2),
    );
    key_bind.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("f".into()),
        },
        Message::TabSearchOpen,
    );

    key_bind
}
