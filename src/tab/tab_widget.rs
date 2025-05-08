use cosmic_text::{Attrs, AttrsList, BufferLine, Edit, LineEnding, Metrics, Motion, SyntaxEditor};
use iced::{
    Element, Length, Padding, Rectangle, Size,
    advanced::{Layout, Widget, graphics::text::editor, image, layout, widget},
    event::Status,
    keyboard, mouse,
};
use std::{
    cell::{Cell, RefCell},
    cmp,
    sync::RwLock,
    time::{self, Instant},
};

use crate::{Message, font_system, swash_cache};

// widget vars for settings & input, state vars for generated state
pub struct TabWidget<'a> {
    editor: &'a RwLock<SyntaxEditor<'static, 'static>>,
    metrics: Metrics,

    // time between clicks for ClickKind.
    click_timing: time::Duration,
    auto_scroll: Option<(f32, (i32, i32))>,

    width: Length,
    height: Length,
    padding: Padding,
}

pub fn tab_widget<'a>(
    editor: &'a RwLock<SyntaxEditor<'static, 'static>>,
    metrics: Metrics,
) -> TabWidget<'a> {
    TabWidget {
        editor,
        metrics,
        click_timing: time::Duration::from_millis(500),
        auto_scroll: None,
        // TODO support tabs, currently allows only space-indentation
        width: Length::Fill,
        height: Length::Fill,
        padding: Padding::new(5.0),
    }
}

enum ClickKind {
    Single,
    Double,
    Triple,
}

struct State {
    dragging: bool,
    // last click
    click_last: Option<(ClickKind, time::Instant, (f32, f32))>,
    undo_buffer: Vec<cosmic_text::Change>,
    redo_buffer: Vec<cosmic_text::Change>,
    // gutter_width set on first draw
    // is a Cell because written in draw
    gutter_width: Cell<i32>,
    max_line_width: Cell<f32>,
    render_handle: RefCell<Option<image::Handle>>,
    parial_scroll: f32,

    modifiers_shift: bool, // solely for shift scroll
}

impl State {
    fn new() -> Self {
        Self {
            dragging: false,
            click_last: None,
            undo_buffer: Vec::new(),
            redo_buffer: Vec::new(),
            gutter_width: Cell::new(0),
            render_handle: RefCell::new(None),
            modifiers_shift: false,
            max_line_width: Cell::new(0.0),
            parial_scroll: 0.0,
        }
    }
}

impl<'a> TabWidget<'a> {
    fn finish_change(&self, editor: &mut SyntaxEditor<'static, 'static>, state: &mut State) {
        if state.redo_buffer.len() > 0 {
            state.redo_buffer.clear();
        }
        if let Some(change) = editor.finish_change() {
            state.undo_buffer.push(change);
        }
    }
    fn start_new_change(&self, editor: &mut SyntaxEditor<'static, 'static>, state: &mut State) {
        self.finish_change(editor, state);
        editor.start_change();
    }
}

impl<'a, Theme, Renderer> Widget<Message, Theme, Renderer> for TabWidget<'a>
where
    Renderer:
        image::Renderer<Handle = image::Handle> + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::new())
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let font_system = font_system().write().expect("font system not writable");
        let editor = self.editor.write().expect("editor not writable");

        // width = self.limits
        // height =
        // if self.limits == shrink
        //   expand to buffer layout runs
        match self.height {
            Length::Fill | Length::FillPortion(_) | Length::Fixed(_) => {
                layout::Node::new(limits.max())
            }
            Length::Shrink => {
                let min_bounds = editor.with_buffer(|buf| {
                    let (w, h) = buf.layout_runs().fold((0.0, 0.0), |(w, h), run| {
                        (run.line_w.max(w), h + run.line_height)
                    });
                    Size::new(w, h)
                });

                iced::advanced::layout::Node::new(
                    limits
                        .height(min_bounds.height)
                        .max()
                        .expand(Size::new(0.0, self.padding.vertical())),
                )
            }
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let time_draw = Instant::now();
        let state = tree.state.downcast_ref::<State>();

        let mut font_system = font_system().write().expect("font system not writable");
        let mut swash_cache = swash_cache().write().expect("swash cache not writable");
        let mut editor = self.editor.write().expect("editor not writable");

        let view_w = cmp::min(viewport.width as i32, layout.bounds().width as i32)
            - self.padding.horizontal() as i32;
        // - scrollbar_w;
        let view_h = cmp::min(viewport.height as i32, layout.bounds().height as i32);
        -self.padding.vertical() as i32;

        let image_w = view_w as i32;
        let image_h = view_h as i32;
        // let image_w: u32 = 500;
        // let image_h: u32 = 500;

        // gutter shifting
        let mut line_count = editor.with_buffer(|buffer| buffer.lines.len());
        let mut line_number_chars = 1;
        while line_count >= 10 {
            line_count /= 10;
            line_number_chars += 1;
        }

        let gutter_width = {
            let text = format!("{:>line_number_chars$}", 1);

            let attrs = Attrs::new().family(cosmic_text::Family::Monospace);
            let mut buffer_line = BufferLine::new(
                text,
                LineEnding::default(),
                AttrsList::new(&attrs),
                cosmic_text::Shaping::Advanced,
            );
            let layout = buffer_line.layout(
                &mut font_system,
                1.0,
                None,
                cosmic_text::Wrap::None,
                None,
                8,
            );

            let layout_line = &layout[0];

            let line_number_width = layout_line.w * self.metrics.font_size;
            let padding_x = 40.0;
            (line_number_width + padding_x).ceil() as i32
        };
        state.gutter_width.replace(gutter_width);

        // get max line width
        let max_line_width = editor.with_buffer(|buffer| {
            let mut max_line_width = 0.0;
            for run in buffer.layout_runs() {
                if run.line_w > max_line_width {
                    max_line_width = run.line_w;
                }
            }

            max_line_width
        });
        state.max_line_width.replace(max_line_width);

        // set metrics to buffer & set size of buffer (for mouse) & optimize shape_as_needed
        editor.with_buffer_mut(|buffer| {
            buffer.set_metrics_and_size(
                &mut font_system,
                self.metrics,
                Some((image_w - gutter_width) as f32),
                Some(image_h as f32),
            );
        });

        // shaping takes 80% of the total drawing time, maybe behind redraw flag?
        // disabling syntax highlighting (using Editor instead) does *not* improve speed
        // shape only necessary lines
        editor.shape_as_needed(&mut font_system, true);

        let mut pixels_u8 = vec![0; image_w as usize * image_h as usize * 4];
        if editor.redraw() {
            let pixels = unsafe {
                std::slice::from_raw_parts_mut(
                    pixels_u8.as_mut_ptr() as *mut u32,
                    pixels_u8.len() / 4,
                )
            };

            let (gutter, gutter_foreground) = {
                let convert_color = |color: syntect::highlighting::Color| {
                    cosmic_text::Color::rgba(color.r, color.g, color.b, color.a)
                };
                let syntax_theme = editor.theme();
                let gutter = syntax_theme
                    .settings
                    .gutter
                    .map_or(editor.background_color(), convert_color);
                let gutter_foreground = syntax_theme
                    .settings
                    .gutter_foreground
                    .map_or(editor.foreground_color(), convert_color);
                (gutter, gutter_foreground)
            };

            draw_rect(
                pixels,
                Canvas {
                    w: image_w,
                    h: image_h,
                },
                Canvas {
                    w: gutter_width,
                    h: image_h,
                },
                Offset { x: 0, y: 0 },
                gutter,
            );

            // line number drawing is significant, maybe cache it?
            // draw line numbers
            /*editor.with_buffer(|buffer| {
                let mut last_line_number = 0;
                for run in buffer.layout_runs() {
                    let line_number = run.line_i.saturating_add(1);
                    if line_number == last_line_number {
                        continue;
                    }
                    last_line_number = line_number;

                    let attrs = Attrs::new().family(cosmic_text::Family::Monospace);
                    let text = format!("{:>line_number_chars$}", line_number);
                    let mut buffer_line = BufferLine::new(
                        text,
                        LineEnding::default(),
                        AttrsList::new(&attrs),
                        cosmic_text::Shaping::Advanced,
                    );
                    let layout = buffer_line.layout(
                        &mut font_system,
                        1.0,
                        None,
                        cosmic_text::Wrap::None,
                        None,
                        8,
                    );

                    let layout_line = &layout[0];

                    // scaling layout_line to fit metrics font size
                    let max_ascent = layout_line.max_ascent * self.metrics.font_size;
                    let max_descent = layout_line.max_descent * self.metrics.font_size;

                    // getting line y offset compared to glyph
                    let glyph_height = max_ascent + max_descent;
                    let centering_offset = (self.metrics.line_height - glyph_height) / 2.0;
                    let line_y = run.line_top + centering_offset + max_ascent;

                    for glyph in layout_line.glyphs.to_vec() {
                        let physical_glyph = glyph.physical((0.0, line_y), self.metrics.font_size);

                        let padding_x_start = 20;
                        swash_cache.with_pixels(
                            &mut font_system,
                            physical_glyph.cache_key,
                            gutter_foreground,
                            |x, y, color| {
                                draw_rect(
                                    pixels,
                                    Canvas {
                                        w: image_w as i32,
                                        h: image_h as i32,
                                    },
                                    Canvas { h: 1, w: 1 },
                                    Offset {
                                        x: physical_glyph.x + x + padding_x_start,
                                        y: physical_glyph.y + y,
                                    },
                                    color,
                                );
                            },
                        );
                    }
                }
            });*/

            // FIXME: cosmic text highlight lines until end of buffer, not end of line
            let scroll_x = editor.with_buffer(|buffer| buffer.scroll().horizontal as i32);
            editor.draw(&mut font_system, &mut swash_cache, |x, y, w, h, color| {
                let mut w = w as i32;
                let mut x = x;
                // adjust drawing if x is behind gutter
                if x < scroll_x {
                    let hidden_w = scroll_x - x;
                    if hidden_w >= w {
                        return;
                    }
                    x = scroll_x;
                    w = w - hidden_w;
                }
                draw_rect(
                    pixels,
                    Canvas {
                        w: image_w as i32,
                        h: image_h as i32,
                    },
                    Canvas { w: w, h: h as i32 },
                    Offset {
                        x: gutter_width + x - scroll_x,
                        y,
                    },
                    color,
                );
            });

            let handle = image::Handle::from_rgba(image_w as u32, image_h as u32, pixels_u8);

            state.render_handle.replace(Some(handle));

            editor.set_redraw(false);
        }

        if let Some(handle) = state.render_handle.borrow().as_ref() {
            let size = Size::new(view_w as f32, view_h as f32);

            let bounds = Rectangle::new(
                layout.position() + [self.padding.left, self.padding.right].into(),
                size,
            );

            let image = image::Image::from(handle).filter_method(image::FilterMethod::Nearest);

            renderer.draw_image(image, bounds);
        }
        log::info!("draw time: {:02?}", time_draw.elapsed());

        // --- POC: font rendering with iced_font ----
        // Verdict: not possible because it renders once for all text, cannot use
        // highlighter, fill_editor cannot be used since it requires iced editor
        // editor.with_buffer(|buffer| {
        //     let mut text = String::new();
        //     for line in buffer.lines.iter() {
        //         text.push_str(line.text());
        //         text.push_str(line.ending().as_str());
        //     }

        //     let metrics = buffer.metrics();

        //     let text = iced::advanced::Text {
        //         content: text,
        //         bounds: size,
        //         size: Pixels::from(metrics.font_size),
        //         line_height: LineHeight::from(Pixels::from(metrics.line_height)),
        //         font: iced::Font::MONOSPACE,
        //         horizontal_alignment: alignment::Horizontal::Left,
        //         vertical_alignment: alignment::Vertical::Top,
        //         shaping: text::Shaping::Advanced,
        //         wrapping: text::Wrapping::None,
        //     };

        //     renderer.fill_text(
        //         text,
        //         layout.position(),
        //         iced::Color::from_rgb(1.0, 0.0, 0.0),
        //         bounds,
        //     )
        // });
        // --- ---
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced::event::Status {
        let state = tree.state.downcast_mut::<State>();
        let gutter_width = state.gutter_width.get();

        let mut font_system = font_system().write().expect("font system is not writable");
        let mut editor = self.editor.write().expect("editor is not writable");
        let mut editor = editor.borrow_with(&mut font_system);
        let (buffer_size, buffer_scroll) =
            editor.with_buffer(|buffer| (buffer.size(), buffer.scroll()));

        let mut status = Status::Ignored;
        match event {
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.modifiers_shift = modifiers.shift()
            }
            iced::Event::Keyboard(event) => {
                if let Some(binding) = Binding::from_keyboard_event(event.clone()) {
                    match binding {
                        Binding::Enter => {
                            self.start_new_change(&mut editor, state);
                            editor.action(cosmic_text::Action::Enter)
                        }
                        Binding::Unindent => {
                            self.start_new_change(&mut editor, state);
                            editor.action(cosmic_text::Action::Unindent)
                        }
                        Binding::Tab => {
                            self.start_new_change(&mut editor, state);
                            editor.insert_string("    ", None);
                            // TODO
                            // if after first non-space character of line, use <tab>
                            // else, tab until equal tab width
                        }
                        Binding::Backspace => {
                            // todo: start new change if previous is not a delete action
                            editor.start_change();
                            editor.action(cosmic_text::Action::Backspace);
                        }
                        Binding::Delete => {
                            // todo: start new change if previous is not a delete action
                            editor.start_change();
                            editor.action(cosmic_text::Action::Delete)
                        }
                        Binding::BackspaceWord => {
                            self.start_new_change(&mut editor, state);
                            if editor.delete_selection() {
                                // selection deleted
                            } else {
                                let cursor_start = editor.cursor();
                                editor.action(cosmic_text::Action::Motion(
                                    cosmic_text::Motion::LeftWord,
                                ));
                                let cursor_end = editor.cursor();
                                editor.delete_range(cursor_end, cursor_start);
                                editor.set_cursor(cursor_end);
                            }
                        }
                        Binding::DeleteWord => {
                            self.start_new_change(&mut editor, state);
                            if editor.delete_selection() {
                                // selection deleted
                            } else {
                                let cursor_start = editor.cursor();
                                editor.action(cosmic_text::Action::Motion(
                                    cosmic_text::Motion::RightWord,
                                ));
                                let cursor_end = editor.cursor();
                                editor.delete_range(cursor_start, cursor_end);
                                editor.set_cursor(cursor_start);
                            }
                        }
                        Binding::Copy => {
                            if let Some(selection) = editor.copy_selection() {
                                clipboard
                                    .write(iced::advanced::clipboard::Kind::Standard, selection);
                            }
                        }
                        Binding::Cut => {
                            self.start_new_change(&mut editor, state);
                            if let Some(content) = editor.copy_selection() {
                                clipboard.write(iced::advanced::clipboard::Kind::Standard, content);
                                editor.action(cosmic_text::Action::Delete);
                            }
                        }
                        Binding::Paste => {
                            if let Some(content) =
                                clipboard.read(iced::advanced::clipboard::Kind::Standard)
                            {
                                self.start_new_change(&mut editor, state);
                                editor.insert_string(&content, None);
                            }
                        }
                        Binding::Move(binding_motion) => {
                            self.start_new_change(&mut editor, state);
                            if let Some((start, end)) = editor.selection_bounds() {
                                editor.set_selection(cosmic_text::Selection::None);

                                match binding_motion {
                                    // just move cursor
                                    BindingMotion::Home
                                    | BindingMotion::End
                                    | BindingMotion::DocumentStart
                                    | BindingMotion::DocumentEnd => {
                                        editor.action(cosmic_text::Action::Motion(
                                            binding_motion.to_cosmic_motion(),
                                        ))
                                    }

                                    // set cursor to start/end of selection
                                    BindingMotion::Left
                                    | BindingMotion::Up
                                    | BindingMotion::WordLeft
                                    | BindingMotion::PageUp => editor.set_cursor(start),

                                    BindingMotion::Right
                                    | BindingMotion::Down
                                    | BindingMotion::PageDown
                                    | BindingMotion::WordRight => editor.set_cursor(end),
                                }
                            } else {
                                editor.action(cosmic_text::Action::Motion(
                                    binding_motion.to_cosmic_motion(),
                                ))
                            }
                        }
                        Binding::Select(binding_motion) => {
                            let cursor = editor.cursor();

                            if editor.selection_bounds().is_none() {
                                editor.set_selection(cosmic_text::Selection::Normal(cursor));
                            }

                            editor.action(cosmic_text::Action::Motion(
                                binding_motion.to_cosmic_motion(),
                            ));

                            // deselect if go back to same position
                            if let Some((start, end)) = editor.selection_bounds() {
                                if start.line == end.line && start.index == end.index {
                                    editor.set_selection(cosmic_text::Selection::None);
                                }
                            }
                        }
                        Binding::Unfocus => {
                            editor.set_selection(cosmic_text::Selection::None);
                        }
                        Binding::SelectAll => {
                            let has_content = editor.with_buffer(|buffer| {
                                // buffer has content
                                buffer.lines.len() > 1
                                    || buffer
                                        .lines
                                        .first()
                                        .is_some_and(|line| !line.text().is_empty())
                            });

                            if has_content {
                                let cursor = editor.cursor();
                                editor.set_selection(cosmic_text::Selection::Normal(
                                    cosmic_text::Cursor {
                                        line: 0,
                                        index: 0,
                                        ..cursor
                                    },
                                ));

                                editor.action(cosmic_text::Action::Motion(
                                    cosmic_text::Motion::BufferEnd,
                                ));
                            }
                        }
                        Binding::Undo => {
                            if let Some(change) = &mut editor.finish_change() {
                                change.reverse();
                                editor.apply_change(&change);
                                state.redo_buffer.push(change.clone());
                            } else {
                                if let Some(change) = &mut state.undo_buffer.pop() {
                                    change.reverse();
                                    editor.apply_change(&change);
                                    state.redo_buffer.push(change.clone());
                                }
                            }
                        }
                        Binding::Redo => {
                            if let Some(_) = &mut editor.finish_change() {
                                // no redos allowed if changing
                            } else {
                                if let Some(change) = &mut state.redo_buffer.pop() {
                                    change.reverse();
                                    editor.apply_change(&change);
                                }
                            }
                        }
                    }
                    status = Status::Captured;
                } else if let keyboard::Event::KeyPressed {
                    text, modifiers, ..
                } = event
                {
                    if !modifiers.logo() && !modifiers.control() && !modifiers.alt() {
                        if let Some(text) = text {
                            if let Some(c) = text.chars().find(|c| !c.is_control()) {
                                editor.start_change();
                                editor.insert_string(&c.to_string(), None);
                                status = Status::Captured
                            }
                        }
                    }
                }
            }
            iced::Event::Mouse(event) => match event {
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    self.start_new_change(&mut editor, state);
                    if let Some(pos) = cursor.position_in(layout.bounds()) {
                        let mut x = pos.x - self.padding.left - gutter_width as f32;
                        let y = pos.y - self.padding.top;

                        // checks if x, y not in gutter
                        if x >= 0.0
                            && x < buffer_size.0.unwrap_or(0.0)
                            && y >= 0.0
                            && y < buffer_size.1.unwrap_or(0.0)
                        {
                            x += buffer_scroll.horizontal;
                            // handle click kind
                            let kind = if let Some((kind, timing, at)) = state.click_last.take() {
                                if timing.elapsed() < self.click_timing && x == at.0 && y == at.1 {
                                    match kind {
                                        // rotate between kinds
                                        ClickKind::Single => ClickKind::Double,
                                        ClickKind::Double => ClickKind::Triple,
                                        ClickKind::Triple => ClickKind::Single,
                                    }
                                } else {
                                    ClickKind::Single
                                }
                            } else {
                                ClickKind::Single
                            };

                            match kind {
                                ClickKind::Single => editor.action(cosmic_text::Action::Click {
                                    x: x as i32,
                                    y: y as i32,
                                }),
                                ClickKind::Double => {
                                    editor.action(cosmic_text::Action::DoubleClick {
                                        x: x as i32,
                                        y: y as i32,
                                    })
                                }
                                ClickKind::Triple => {
                                    editor.action(cosmic_text::Action::TripleClick {
                                        x: x as i32,
                                        y: y as i32,
                                    })
                                }
                            }
                            state.click_last = Some((kind, Instant::now(), (x, y)));
                            state.dragging = true;
                        }
                    }
                    status = Status::Captured;
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                    state.dragging = false;
                    self.auto_scroll = None;

                    status = Status::Captured;
                    shell.publish(Message::SetAutoScroll(None));
                }
                iced::mouse::Event::CursorMoved { .. } => {
                    if state.dragging {
                        if let Some(pos) = cursor.position() {
                            // cares when cursor is outside of window
                            let mut x =
                                pos.x - layout.bounds().x - self.padding.left - gutter_width as f32;
                            let y = pos.y - layout.bounds().y - self.padding.top;

                            x += buffer_scroll.horizontal;

                            editor.action(cosmic_text::Action::Drag {
                                x: x as i32,
                                y: y as i32,
                            });
                            let auto_scroll = editor.with_buffer(|buffer| {
                                //TODO: ideal auto scroll speed
                                let speed = 1.01;
                                if y < 0.0 {
                                    Some(y * speed)
                                } else if y > buffer.size().1.unwrap_or(0.0) {
                                    Some((y - buffer.size().1.unwrap_or(0.0)) * speed)
                                } else {
                                    None
                                }
                            });
                            status = Status::Captured;
                            shell.publish(Message::SetAutoScroll(auto_scroll));
                        }
                    }
                }
                // TODO scroll past editor bounds
                iced::mouse::Event::WheelScrolled { delta } => {
                    if let Some(_) = cursor.position_in(layout.bounds()) {
                        let (x, lines_y) = match delta {
                            iced::mouse::ScrollDelta::Lines { x, y } => {
                                // method from iced text_editor
                                let lines_y = if y.abs() > 0.0 {
                                    y.signum() * -(y.abs() * 4.0).max(1.0)
                                } else {
                                    0.0
                                };
                                (x * 4.0, lines_y)
                            }
                            iced::mouse::ScrollDelta::Pixels { x, y } => {
                                // method from iced text_editor
                                let lines_y = -y / 4.0;

                                (x, lines_y)
                            }
                        };

                        let mut lines_y = lines_y + state.parial_scroll;
                        state.parial_scroll = lines_y.fract();
                        lines_y = lines_y.trunc();

                        // Note: mouse event + modifiers is still in PR https://github.com/iced-rs/iced/pull/2733
                        if state.modifiers_shift {
                            // scroll only y
                            // Note: skipping set_scroll/action makes it a tad faster
                            if lines_y != 0.0 {
                                editor.with_buffer_mut(|buffer| {
                                    let mut scroll = buffer.scroll();
                                    let buffer_w = buffer.size().0.unwrap_or(0.0);

                                    scroll.horizontal +=
                                        lines_y as f32 * buffer.metrics().font_size;
                                    scroll.horizontal = scroll
                                        .horizontal
                                        .min(state.max_line_width.get() - buffer_w)
                                        .max(0.0);

                                    buffer.set_scroll(scroll);
                                });
                            }
                        } else {
                            // scroll x and y
                            if lines_y != 0.0 {
                                editor.action(cosmic_text::Action::Scroll {
                                    lines: lines_y as i32,
                                });
                            }

                            if x != 0.0 {
                                editor.with_buffer_mut(|buffer| {
                                    let mut scroll = buffer.scroll();
                                    let buffer_w = buffer.size().0.unwrap_or(0.0);

                                    scroll.horizontal += -x;
                                    scroll.horizontal = scroll
                                        .horizontal
                                        .min(state.max_line_width.get() - buffer_w)
                                        .max(0.0);

                                    buffer.set_scroll(scroll);
                                });
                            }
                        }
                        status = Status::Captured;
                    }
                }
                _ => {}
            },
            _ => {}
        };

        status
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if let Some(p) = cursor.position_in(layout.bounds()) {
            let gutter_width = state.gutter_width.get();
            let editor = self.editor.read().unwrap();
            let buffer_size = editor.with_buffer(|buffer| buffer.size());

            let x = p.x - self.padding.left - gutter_width as f32;
            let y = p.y - self.padding.top;
            if x >= 0.0
                && x < buffer_size.0.unwrap_or(0.0)
                && y >= 0.0
                && y < buffer_size.1.unwrap_or(0.0)
            {
                return mouse::Interaction::Text;
            }
        }

        mouse::Interaction::Idle
    }
}

impl<'a, Theme, Renderer> From<TabWidget<'a>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: image::Renderer<Handle = iced::advanced::image::Handle>
        + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn from(value: TabWidget<'a>) -> Self {
        Self::new(value)
    }
}

// event -> binding -> editor.action
pub enum Binding {
    Enter,
    Tab,
    Unindent,
    Backspace,
    BackspaceWord,
    Delete,
    DeleteWord,
    Unfocus,
    Copy,
    Cut,
    Paste,
    SelectAll,
    Move(BindingMotion),
    Select(BindingMotion),
    Undo,
    Redo,
}

pub enum BindingMotion {
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    WordLeft,
    WordRight,
    DocumentStart,
    DocumentEnd,
}

impl BindingMotion {
    fn from_named_key(key: keyboard::key::Named) -> Option<Self> {
        match key {
            keyboard::key::Named::ArrowLeft => Some(Self::Left),
            keyboard::key::Named::ArrowRight => Some(Self::Right),
            keyboard::key::Named::ArrowDown => Some(Self::Down),
            keyboard::key::Named::ArrowUp => Some(Self::Up),
            keyboard::key::Named::PageUp => Some(Self::PageUp),
            keyboard::key::Named::PageDown => Some(Self::PageDown),
            keyboard::key::Named::Home => Some(Self::Home),
            keyboard::key::Named::End => Some(Self::End),
            _ => None,
        }
    }

    fn to_cosmic_motion(self) -> Motion {
        match self {
            BindingMotion::Left => Motion::Left,
            BindingMotion::Right => Motion::Right,
            BindingMotion::Up => Motion::Up,
            BindingMotion::Down => Motion::Down,
            BindingMotion::PageUp => Motion::PageUp,
            BindingMotion::PageDown => Motion::PageDown,
            BindingMotion::Home => Motion::Home,
            BindingMotion::End => Motion::End,
            BindingMotion::WordLeft => Motion::LeftWord,
            BindingMotion::WordRight => Motion::RightWord,
            BindingMotion::DocumentStart => Motion::BufferStart,
            BindingMotion::DocumentEnd => Motion::BufferEnd,
        }
    }
}

impl Binding {
    fn from_keyboard_event(event: iced::keyboard::Event) -> Option<Self> {
        match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => match key.as_ref() {
                keyboard::Key::Named(keyboard::key::Named::Enter) => Some(Self::Enter),
                keyboard::Key::Named(keyboard::key::Named::Tab) if modifiers.shift() => {
                    Some(Self::Unindent)
                }
                keyboard::Key::Named(keyboard::key::Named::Tab) => Some(Self::Tab),
                keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                    if modifiers.command() {
                        Some(Self::BackspaceWord)
                    } else {
                        Some(Self::Backspace)
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Delete) => {
                    if modifiers.command() {
                        Some(Self::DeleteWord)
                    } else {
                        Some(Self::Delete)
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Self::Unfocus),
                keyboard::Key::Character("c") if modifiers.command() => Some(Self::Copy),
                keyboard::Key::Character("x") if modifiers.command() => Some(Self::Cut),
                keyboard::Key::Character("v") if modifiers.command() => Some(Self::Paste),
                keyboard::Key::Character("a") if modifiers.command() => Some(Self::SelectAll),
                keyboard::Key::Character("z") if modifiers.command() && modifiers.shift() => {
                    Some(Self::Redo)
                }
                keyboard::Key::Character("z") if modifiers.command() => Some(Self::Undo),
                keyboard::Key::Named(name) => {
                    let motion = BindingMotion::from_named_key(name)?;
                    let motion = if modifiers.macos_command() {
                        match motion {
                            BindingMotion::Left => BindingMotion::Home,
                            BindingMotion::Right => BindingMotion::End,
                            _ => motion,
                        }
                    } else {
                        motion
                    };

                    let motion = if modifiers.jump() {
                        match motion {
                            BindingMotion::Left => BindingMotion::WordLeft,
                            BindingMotion::Right => BindingMotion::WordRight,
                            BindingMotion::Home => BindingMotion::DocumentStart,
                            BindingMotion::End => BindingMotion::DocumentEnd,
                            _ => motion,
                        }
                    } else {
                        motion
                    };

                    Some(if modifiers.shift() {
                        Self::Select(motion)
                    } else {
                        Self::Move(motion)
                    })
                }
                _ => None,
            },
            _ => None,
        }
    }
}

struct Canvas {
    w: i32,
    h: i32,
}

struct Offset {
    x: i32,
    y: i32,
}

/// This function is called canvas.x * canvas.y number of times
/// each time the text is scrolled or the canvas is resized.
/// If the canvas is moved, it's not called as the pixel buffer
/// is the same, it's just translated for the screen's x, y.
/// canvas is the location of the pixel in the canvas.
/// Screen is the location of the pixel on the screen.
// TODO: improve performance
fn draw_rect(
    buffer: &mut [u32],
    canvas: Canvas,
    offset: Canvas,
    screen: Offset,
    cosmic_color: cosmic_text::Color,
) {
    // Grab alpha channel and green channel
    let mut color = cosmic_color.0 & 0xFF00FF00;
    // Shift red channel
    color |= (cosmic_color.0 & 0x00FF0000) >> 16;
    // Shift blue channel
    color |= (cosmic_color.0 & 0x000000FF) << 16;

    let alpha = (color >> 24) & 0xFF;
    match alpha {
        0 => {
            // Do not draw if alpha is zero.
        }
        255 => {
            // Handle overwrite
            for x in screen.x..screen.x + offset.w {
                if x < 0 || x >= canvas.w {
                    // Skip if y out of bounds
                    continue;
                }

                for y in screen.y..screen.y + offset.h {
                    if y < 0 || y >= canvas.h {
                        // Skip if x out of bounds
                        continue;
                    }

                    let line_offset = y as usize * canvas.w as usize;
                    let offset = line_offset + x as usize;
                    buffer[offset] = color;
                }
            }
        }
        _ => {
            let n_alpha = 255 - alpha;
            for y in screen.y..screen.y + offset.h {
                if y < 0 || y >= canvas.h {
                    // Skip if y out of bounds
                    continue;
                }

                let line_offset = y as usize * canvas.w as usize;
                for x in screen.x..screen.x + offset.w {
                    if x < 0 || x >= canvas.w {
                        // Skip if x out of bounds
                        continue;
                    }

                    // Alpha blend with current value
                    let offset = line_offset + x as usize;
                    let current = buffer[offset];
                    if current & 0xFF000000 == 0 {
                        // Overwrite if buffer empty
                        buffer[offset] = color;
                    } else {
                        let rb = ((n_alpha * (current & 0x00FF00FF))
                            + (alpha * (color & 0x00FF00FF)))
                            >> 8;
                        let ag = (n_alpha * ((current & 0xFF00FF00) >> 8))
                            + (alpha * (0x01000000 | ((color & 0x0000FF00) >> 8)));
                        buffer[offset] = (rb & 0x00FF00FF) | (ag & 0xFF00FF00);
                    }
                }
            }
        }
    }
}
