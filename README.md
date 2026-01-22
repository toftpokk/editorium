# Editorium

A code editor for learning purposes

## Features Compared to Other Editors
- resizable panels
- faster initial file load time
- undo/redo (incomplete)

## Inspired by
- Bruce-Hopkins/code-editor-prototype
- pop-os/cosmic-edit
- iced text_editor
- squidowl/halloy

## Oniguruma Build Errors
Oniguruma dependency archived https://github.com/rust-onig/rust-onig/issues/203
The fix is to compile with zig backend, with a different gnu version

    cargo zigbuild --target x86_64-unknown-linux-gnu.2.17