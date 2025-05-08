use std::sync::RwLock;

use cosmic_text::{Metrics, SyntaxEditor};
mod text_box;

pub fn text_box<'a>(
    editor: &'a RwLock<SyntaxEditor<'static, 'static>>,
    metrics: Metrics,
) -> text_box::TextBox<'a> {
    text_box::TextBox::new(editor, metrics)
}
