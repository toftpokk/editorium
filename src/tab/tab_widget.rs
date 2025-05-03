// // TODO rename Tab -> Textbox

// use std::fmt::Debug;

// use crate::Message;

// use iced::advanced::graphics::core::Element;
// use iced::advanced::layout::{self, Layout};
// use iced::advanced::renderer;
// use iced::advanced::widget::{self, Widget};
// use iced::mouse;
// use iced::widget::{Column, canvas, column};
// use iced::{Color, Length, Rectangle, Size};
// use iced::{Renderer, border};
// struct Theme {}
// // impl<Message> canvas::Program<Message, Renderer> for TabWidget
// // where
// //     Renderer: iced::advanced::text::Renderer,
// // {
// //     type State;
// //     fn draw(
// //         &self,
// //         state: &Self::State,
// //         renderer: &Renderer,
// //         theme: &Renderer,
// //         bounds: Rectangle,
// //         cursor: iced::advanced::mouse::Cursor,
// //     ) -> Vec<canvas::Geometry<Renderer>> {
// //         todo!()
// //     }
// // }
// struct Example {
//     radius: f32,
// }
// impl Example {
//     fn new() -> Self {
//         Example { radius: 50.0 }
//     }
//     fn update(&mut self, message: Message) {
//         // match message {
//         //     Message::RadiusChanged(radius) => {
//         //         self.radius = radius;
//         //     }
//         // }
//     }
//     fn view(&self) -> Element<Message> {
//         let content = column![
//             widget(),
//             // text!("Radius: {:.2}", self.radius),
//             // slider(1.0..=100.0, self.radius, Message::RadiusChanged).step(0.01),
//         ]
//         .padding(20)
//         .spacing(20)
//         .max_width(500);
//         // .align_x(Center);
//         center(content).into()
//     }
// }

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::border::Radius;
use iced::{Border, border};
use iced::{Color, Element, Length, Rectangle, Size};
use iced::{Shadow, mouse};

pub struct TabWidget {}

pub fn tab_widget() -> TabWidget {
    TabWidget {}
}

impl<Message, Theme, Renderer: renderer::Renderer> Widget<Message, Theme, Renderer> for TabWidget {
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fill).height(Length::Fill);

        let size = Size::new(limits.max().width, 20.0);

        iced::advanced::layout::Node::new(limits.resolve(Length::Fill, Length::Fill, size))
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(0.0),
                ..renderer::Quad::default()
            },
            Color::BLACK,
        );
    }
}
impl<'a, Message, Theme, Renderer: renderer::Renderer> From<TabWidget>
    for Element<'a, Message, Theme, Renderer>
{
    fn from(value: TabWidget) -> Self {
        Self::new(value)
    }
}

// // Copied from cosmic-text
// // impl<Message, Theme, Renderer>  Widget<Message, Theme, Renderer> for TabWidget
// // where
// //     Renderer: iced::advanced::Renderer,
// // {
// //     fn size(&self) -> iced::Size<Length> {
// //         Size::new(Length::Fill, Length::Fill)
// //     }

// //     fn layout(
// //         &self,
// //         tree: &mut iced::advanced::widget::Tree,
// //         renderer: &Renderer,
// //         limits: &iced::advanced::layout::Limits,
// //     ) -> iced::advanced::layout::Node {
// //         let limits = limits.width(Length::Fill).height(Length::Fill);

// //         let mut font_system = FONT_SYSTEM.get().unwrap().write().unwrap();

// //         // FIXME should editor be written during layout?
// //         let editor = &mut self.editor.write().unwrap();

// //         editor.borrow_with(&mut font_system).shape_as_needed(true); // FIXME Scroll

// //         editor.with_buffer(|buffer| {
// //             let mut lines = 0;
// //             for line in buffer.lines.iter() {
// //                 // one buffer line can wrap into multiple
// //                 if let Some(opt) = line.layout_opt() {
// //                     lines += opt.len()
// //                 }
// //             }

// //             let height = lines as f32 * buffer.metrics().line_height;
// //             let size = Size::new(limits.max().width, height);

// //             iced::advanced::layout::Node::new(limits.resolve(Length::Fill, Length::Fill, size))
// //         })
// //     }

// //     fn draw(
// //         &self,
// //         tree: &iced::advanced::widget::Tree,
// //         renderer: &mut Renderer,
// //         theme: &Theme,
// //         style: &iced::advanced::renderer::Style,
// //         layout: iced::advanced::Layout<'_>,
// //         cursor: iced::advanced::mouse::Cursor,
// //         viewport: &iced::Rectangle,
// //     ) {
// //         renderer.fill_quad()
// //         // // {
// //         // //     let instant = Instant::now();

// //         // //     let state = tree.state.downcast_ref::<State>();

// //         // let mut editor = self.editor.write().unwrap();

// //         // //     let cosmic_theme = theme.cosmic();
// //         // //     let scrollbar_w = cosmic_theme.spacing.space_xxs as i32;

// //         // let view_w = cmp::min(viewport.width as i32, layout.bounds().width as i32)
// //         //     - self.padding.horizontal() as i32;
// //         // // - scrollbar_w;
// //         // let view_h = cmp::min(viewport.height as i32, layout.bounds().height as i32)
// //         //     - self.padding.vertical() as i32;

// //         // // let scale_factor = style.scale_factor as f32;
// //         // let scale_factor = 1.0;
// //         // let metrics = self.metrics.scale(scale_factor);

// //         // let calculate_image_scaled = |view: i32| -> (i32, f32) {
// //         //     // Get smallest set of physical pixels that fit inside the logical pixels
// //         //     let image = ((view as f32) * scale_factor).floor() as i32;
// //         //     // Convert that back into logical pixels
// //         //     let scaled = (image as f32) / scale_factor;
// //         //     (image, scaled)
// //         // };
// //         // let calculate_ideal = |view_start: i32| -> (i32, f32) {
// //         //     // Search for a perfect match within 16 pixels
// //         //     for i in 0..16 {
// //         //         let view = view_start - i;
// //         //         let (image, scaled) = calculate_image_scaled(view);
// //         //         if view == scaled as i32 {
// //         //             return (image, scaled);
// //         //         }
// //         //     }
// //         //     let (image, scaled) = calculate_image_scaled(view_start);
// //         //     (image, scaled)
// //         // };

// //         // let (image_w, scaled_w) = calculate_ideal(view_w);
// //         // let (image_h, scaled_h) = calculate_ideal(view_h);

// //         // if image_w <= 0 || image_h <= 0 {
// //         //     // Zero sized image
// //         //     return;
// //         // }

// //         // // Lock font system (used throughout)
// //         // let mut font_system = FONT_SYSTEM.get().unwrap().write().unwrap();

// //         // //     // Calculate line number information
// //         // //     let (line_number_chars, editor_offset_x) = if self.line_numbers {
// //         // //         // Calculate number of characters needed in line number
// //         // //         let mut line_number_chars = 1;
// //         // //         let mut line_count = editor.with_buffer(|buffer| buffer.lines.len());
// //         // //         while line_count >= 10 {
// //         // //             line_count /= 10;
// //         // //             line_number_chars += 1;
// //         // //         }

// //         // //         // Calculate line number width
// //         // //         let mut line_number_width = 0.0;
// //         // //         {
// //         // //             let mut line_number_cache = LINE_NUMBER_CACHE.get().unwrap().lock().unwrap();
// //         // //             if let Some(layout_line) = line_number_cache
// //         // //                 .get(
// //         // //                     font_system.raw(),
// //         // //                     LineNumberKey {
// //         // //                         number: 1,
// //         // //                         width: line_number_chars,
// //         // //                     },
// //         // //                 )
// //         // //                 .first()
// //         // //             {
// //         // //                 let line_width = layout_line.w * metrics.font_size;
// //         // //                 if line_width > line_number_width {
// //         // //                     line_number_width = line_width;
// //         // //                 }
// //         // //             }
// //         // //         }

// //         // //         (line_number_chars, (line_number_width + 8.0).ceil() as i32)
// //         // //     } else {
// //         // //         (0, 0)
// //         // //     };

// //         // //     // Save editor offset in state
// //         // //     if state.editor_offset_x.replace(editor_offset_x) != editor_offset_x {
// //         // //         // Mark buffer as needing redraw if editor offset has changed
// //         // //         editor.set_redraw(true);
// //         // //     }

// //         // //     // Set metrics and size
// //         // //     editor.with_buffer_mut(|buffer| {
// //         // //         buffer.set_metrics_and_size(
// //         // //             font_system.raw(),
// //         // //             metrics,
// //         // //             Some((image_w - editor_offset_x) as f32),
// //         // //             Some(image_h as f32),
// //         // //         )
// //         // //     });

// //         // // Shape and layout as needed
// //         // editor.shape_as_needed(&mut font_system, true);

// //         // //     let mut handle_opt = state.handle_opt.lock().unwrap();
// //         // if editor.redraw() {
// //         //     // Draw to pixel buffer
// //         //     let mut pixels_u8 = vec![0; image_w as usize * image_h as usize * 4];
// //         //     {
// //         //         //             let mut swash_cache = SWASH_CACHE.get().unwrap().lock().unwrap();

// //         //         let pixels = unsafe {
// //         //             std::slice::from_raw_parts_mut(
// //         //                 pixels_u8.as_mut_ptr() as *mut u32,
// //         //                 pixels_u8.len() / 4,
// //         //             )
// //         //         };

// //         //         // if self.line_numbers {
// //         //         //                 let (gutter, gutter_foreground) = {
// //         //         //                     let convert_color = |color: syntect::highlighting::Color| {
// //         O rename Tab -> Textbox
// // Cop//         //                         .gutter
// //         //         //                         .map_or(editor.background_color(), convert_color);
// //         //         //                     let gutter_foreground = syntax_theme
// //         //         //                         .settings
// //         //         //                         .gutter_foreground
// //         //         //                         .map_or(editor.foreground_color(), convert_color);
// //         //         //                     (gutter, gutter_foreground)
// //         //         // };

// //         //         // Ensure fill with gutter color
// //         //         let editor_offset_x = 0; // REMOVE
// //         //         draw_rect(
// //         //             pixels,
// //         //             Canvas {
// //         //                 w: image_w,
// //         //                 h: image_h,
// //         //             },
// //         //             Canvas {
// //         //                 w: editor_offset_x,
// //         //                 h: image_h,
// //         //             },
// //         //             Offset { x: 0, y: 0 },
// //         //             Color::rgba(0, 0, 0, 0),
// //         //         );

// //         //         //                 // Draw line numbers
// //         //         //                 //TODO: move to cosmic-text?
// //         //         //                 editor.with_buffer(|buffer| {
// //         //         //                     let mut line_number_cache =
// //         //         //                         LINE_NUMBER_CACHE.get().unwrap().lock().unwrap();
// //         //         //                     let mut last_line_number = 0;
// //         //         //                     for run in buffer.layout_runs() {
// //         //         //                         let line_number = run.line_i.saturating_add(1);
// //         //         //                         if line_number == last_line_number {
// //         //         //                             // Skip duplicate lines
// //         //         //                             continue;
// //         //         //                         } else {
// //         //         //                             last_line_number = line_number;
// //         //         //                         }

// //         //         //                         if let Some(layout_line) = line_number_cache
// //         //         //                             .get(
// //         //         //                                 font_system.raw(),
// //         //         //                                 LineNumberKey {
// //         //         //                                     number: line_number,
// //         //         //                                     width: line_number_chars,
// //         //         //                                 },
// //         //         //                             )
// //         //         //                             .first()
// //         //         //                         {
// //         //         //                             // These values must be scaled since layout is done at font size 1.0
// //         //         //                             let max_ascent = layout_line.max_ascent * metrics.font_size;
// //         //         //                             let max_descent = layout_line.max_descent * metrics.font_size;

// //         //         //                             // This code comes from cosmic_text::LayoutRunIter
// //         //         //                             let glyph_height = max_ascent + max_descent;
// //         //         //                             let centering_offset = (metrics.line_height - glyph_height) / 2.0;
// //         //         //                             let line_y = run.line_top + centering_offset + max_ascent;

// //         //         //                             for layout_glyph in layout_line.glyphs.iter() {
// //         //         //                                 let physical_glyph =
// //         //         //                                     layout_glyph.physical((0., line_y), metrics.font_size);

// //         //         //                                 swash_cache.with_pixels(
// //         //         //                                     font_system.raw(),
// //         //         //                                     physical_glyph.cache_key,
// //         //         //                                     gutter_foreground,
// //         //         //                                     |x, y, color| {
// //         //         //                                         draw_rect(
// //         //         //                                             pixels,
// //         //         //                                             Canvas {
// //         //         //                                                 w: image_w,
// //         //         //                                                 h: image_h,
// //         //         //                                             },
// //         //         //                                             Canvas { w: 1, h: 1 },
// //         //         //                                             Offset {
// //         //         //                                                 x: physical_glyph.x + x,
// //         //         //                                                 y: physical_glyph.y + y,
// //         //         //                                             },
// //         //         //                                             color,
// //         //         //                                         );
// //         //         //                                     },
// //         //         //                                 );
// //         //         //                             }
// //         //         //                         }
// //         //         //                     }
// //         //         //                 });
// //         //         //             }

// //         //         //             if self.highlight_current_line {
// //         //         //                 let line_highlight = {
// //         //         //                     let convert_color = |color: syntect::highlighting::Color| {
// //         //         //                         cosmic_text::Color::rgba(color.r, color.g, color.b, color.a)
// //         //         //                     };
// //         //         //                     let syntax_theme = editor.theme();
// //         //         //                     //TODO: ideal fallback for line highlight color
// //         //         //                     syntax_theme
// //         //         //                         .settings
// //         //         //                         .line_highlight
// //         //         //                         .map_or(editor.background_color(), convert_color)
// //         //         //                 };

// //         //         //                 let cursor = editor.cursor();
// //         //         //                 editor.with_buffer(|buffer| {
// //         //         //                     for run in buffer.layout_runs() {
// //         //         //                         if run.line_i != cursor.line {
// //         //         //                             continue;
// //         //         //                         }

// //         //         //                         draw_rect(
// //         //         //                             pixels,
// //         //         //                             Canvas {
// //         //         //                                 w: image_w,
// //         //         //                                 h: image_h,
// //         //         //                             },
// //         //         //                             Canvas {
// //         //         //                                 w: image_w - editor_offset_x,
// //         //         //                                 h: metrics.line_height as i32,
// //         //         //                             },
// //         //         //                             Offset {
// //         //         //                                 x: editor_offset_x,
// //         //         //                                 y: run.line_top as i32,
// //         //         //                             },
// //         //         //                             line_highlight,
// //         //         //                         );
// //         //         //                     }
// //         //         //                 });
// //         //         //             }

// //         //         //             // Draw editor
// //         //         //             let scroll_x = editor.with_buffer(|buffer| buffer.scroll().horizontal as i32);
// //         //         //             editor.draw(font_system.raw(), &mut swash_cache, |x, y, w, h, color| {
// //         //         //                 if x < scroll_x {
// //         //         //                     //TODO: modify width?
// //         //         //                     return;
// //         //         //                 }
// //         //         //                 draw_rect(
// //         //         //                     pixels,
// //         //         //                     Canvas {
// //         //         //                         w: image_w,
// //         //         //                         h: image_h,
// //         //         //                     },
// //         //         //                     Canvas {
// //         //         //                         w: w as i32,
// //         //         //                         h: h as i32,
// //         //         //                     },
// //         //         //                     Offset {
// //         //         //                         x: editor_offset_x + x - scroll_x,
// //         //         //                         y,
// //         //         //                     },
// //         //         //                     color,
// //         //         //                 );
// //         //         //             });

// //         //         //             // Calculate scrollbar
// //         //         //             editor.with_buffer(|buffer| {
// //         //         //                 let mut start_line_opt = None;
// //         //         //                 let mut end_line = 0;
// //         //         //                 let mut max_line_width = 0.0;
// //         //         //                 for run in buffer.layout_runs() {
// //         //         //                     end_line = run.line_i;
// //         //         //                     if start_line_opt.is_none() {
// //         //         //                         start_line_opt = Some(end_line);
// //         //         //                     }
// //         //         //                     if run.line_w > max_line_width {
// //         //         //                         max_line_width = run.line_w;
// //         //         //                     }
// //         //         //                 }

// //         //         //                 let start_line = start_line_opt.unwrap_or(end_line);
// //         //         //                 let lines = buffer.lines.len();
// //         //         //                 let start_y = (start_line * image_h as usize) / lines;
// //         //         //                 let end_y = ((end_line + 1) * image_h as usize) / lines;

// //         //         //                 let rect = Rectangle::new(
// //         //         //                     [image_w as f32 / scale_factor, start_y as f32 / scale_factor].into(),
// //         //         //                     Size::new(
// //         //         //                         scrollbar_w as f32,
// //         //         //                         (end_y as f32 - start_y as f32) / scale_factor,
// //         //         //                     ),
// //         //         //                 );
// //         //         //                 state.scrollbar_v_rect.set(rect);

// //         //         //                 let (buffer_w_opt, buffer_h_opt) = buffer.size();
// //         //         //                 let buffer_w = buffer_w_opt.unwrap_or(0.0);
// //         //         //                 let buffer_h = buffer_h_opt.unwrap_or(0.0);
// //         //         //                 let scrollbar_h_width = image_w as f32 / scale_factor - scrollbar_w as f32;
// //         //         //                 if buffer_w < max_line_width {
// //         //         //                     let rect = Rectangle::new(
// //         //         //                         [
// //         //         //                             (buffer.scroll().horizontal / max_line_width) * scrollbar_h_width,
// //         //         //                             buffer_h / scale_factor - scrollbar_w as f32,
// //         //         //                         ]
// //         //         //                         .into(),
// //         //         //                         Size::new(
// //         //         //                             (buffer_w / max_line_width) * scrollbar_h_width,
// //         //         //                             scrollbar_w as f32,
// //         //         //                         ),
// //         //         //                     );
// //         //         //                     state.scrollbar_h_rect.set(Some(rect));
// //         //         //                 } else {
// //         //         //                     state.scrollbar_h_rect.set(None);
// //         //         //                 }
// //         //         //             });
// //         //     }

// //         //     //         // Clear redraw flag
// //         //     //         editor.set_redraw(false);

// //         //     //         state.scale_factor.set(scale_factor);
// //         //     //         *handle_opt = Some(image::Handle::from_rgba(
// //         //     //             image_w as u32,
// //         //     //             image_h as u32,
// //         //     //             pixels_u8,
// //         //     //         ));
// //         // }

// //         // //     let image_position = layout.position() + [self.padding.left, self.padding.top].into();
// //         // //     if let Some(ref handle) = *handle_opt {
// //         // //         let image_size = image::Renderer::measure_image(renderer, handle);
// //         // //         let scaled_size = Size::new(scaled_w as f32, scaled_h as f32);
// //         // //         log::debug!(
// //         // //             "text_box image {:?} scaled {:?} position {:?}",
// //         // //             image_size,
// //         // //             scaled_size,
// //         // //             image_position
// //         // //         );
// //         // //         image::Renderer::draw_image(
// //         // //             renderer,
// //         // //             handle.clone(),
// //         // //             image::FilterMethod::Nearest,
// //         // //             Rectangle::new(image_position, scaled_size),
// //         // //             Radians(0.0),
// //         // //             1.0,
// //         // //             [0.0; 4],
// //         // //         );
// //         // //     }

// //         // //     // Draw vertical scrollbar
// //         // //     {
// //         // //         let scrollbar_v_rect = state.scrollbar_v_rect.get();

// //         // //         // neutral_3, 0.7
// //         // //         let track_color = cosmic_theme
// //         // //             .palette
// //         // //             .neutral_3
// //         // //             .without_alpha()
// //         // //             .with_alpha(0.7);

// //         // //         // Draw track quad
// //         // //         renderer.fill_quad(
// //         // //             Quad {
// //         // //                 bounds: Rectangle::new(
// //         // //                     Point::new(image_position.x + scrollbar_v_rect.x, image_position.y),
// //         // //                     Size::new(scrollbar_v_rect.width, layout.bounds().height),
// //         // //                 ),
// //         // //                 border: Border {
// //         // //                     radius: (scrollbar_v_rect.width / 2.0).into(),
// //         // //                     width: 0.0,
// //         // //                     color: Color::TRANSPARENT,
// //         // //                 },
// //         // //                 ..Default::default()
// //         // //             },
// //         // //             Color::from(track_color),
// //         // //         );

// //         // //         let pressed = matches!(&state.dragging, Some(Dragging::ScrollbarV { .. }));

// //         // //         let mut hover = false;
// //         // //         if let Some(p) = cursor_position.position_in(layout.bounds()) {
// //         // //             let x = p.x - self.padding.left;
// //         // //             if x >= scrollbar_v_rect.x && x < (scrollbar_v_rect.x + scrollbar_v_rect.width) {
// //         // //                 hover = true;
// //         // //             }
// //         // //         }

// //         // //         let mut scrollbar_draw =
// //         // //             scrollbar_v_rect + Vector::new(image_position.x, image_position.y);
// //         // //         if !hover && !pressed {
// //         // //             // Decrease draw width and keep centered when not hovered or pressed
// //         // //             scrollbar_draw.width /= 2.0;
// //         // //             scrollbar_draw.x += scrollbar_draw.width / 2.0;
// //         // //         }

// //         // //         // neutral_6, 0.7
// //         // //         let base_color = cosmic_theme
// //         // //             .palette
// //         // //             .neutral_6
// //         // //             .without_alpha()
// //         // //             .with_alpha(0.7);
// //         // //         let scrollbar_color = if pressed {
// //         // //             // pressed_state_color, 0.5
// //         // //             cosmic_theme
// //         // //                 .background
// //         // //                 .component
// //         // //                 .pressed
// //         // //                 .without_alpha()
// //         // //                 .with_alpha(0.5)
// //         // //                 .over(base_color)
// //         // //         } else if hover {
// //         // //             // hover_state_color, 0.2
// //         // //             cosmic_theme
// //         // //                 .background
// //         // //                 .component
// //         // //                 .hover
// //         // //                 .without_alpha()
// //         // //                 .with_alpha(0.2)
// //         // //                 .over(base_color)
// //         // //         } else {
// //         // //             base_color
// //         // //         };

// //         // //         // Draw scrollbar quad
// //         // //         renderer.fill_quad(
// //         // //             Quad {
// //         // //                 bounds: scrollbar_draw,
// //         // //                 border: Border {
// //         // //                     radius: (scrollbar_draw.width / 2.0).into(),
// //         // //                     width: 0.0,
// //         // //                     color: Color::TRANSPARENT,
// //         // //                 },
// //         // //                 ..Default::default()
// //         // //             },
// //         // //             Color::from(scrollbar_color),
// //         // //         );
// //         // //     }

// //         // //     // Draw horizontal scrollbar
// //         // //     //TODO: reduce repitition
// //         // //     if let Some(scrollbar_h_rect) = state.scrollbar_h_rect.get() {
// //         // //         /*TODO: horizontal scrollbar track?
// //         // //         // neutral_3, 0.7
// //         // //         let track_color = cosmic_theme
// //         // //             .palette
// //         // //             .neutral_3
// //         // //             .without_alpha()
// //         // //             .with_alpha(0.7);

// //         // //         // Draw track quad
// //         // //         renderer.fill_quad(
// //         // //             Quad {
// //         // //                 bounds: Rectangle::new(
// //         // //                     Point::new(image_position.x, image_position.y + scrollbar_h_rect.y),
// //         // //                     Size::new(
// //         // //                         layout.bounds().width - scrollbar_w as f32,
// //         // //                         scrollbar_h_rect.height,
// //         // //                     ),
// //         // //                 ),
// //         // //                 border: Border {
// //         // //                     radius: (scrollbar_h_rect.height / 2.0).into(),
// //         // //                     width: 0.0,
// //         // //                     color: Color::TRANSPARENT,
// //         // //                 },
// //         // //                 ..Default::default()
// //         // //             },
// //         // //             Color::from(track_color),
// //         // //         );
// //         // //         */
// //         // //         let pressed = matches!(&state.dragging, Some(Dragging::ScrollbarH { .. }));

// //         // //         let mut hover = false;
// //         // //         if let Some(p) = cursor_position.position_in(layout.bounds()) {
// //         // //             let y = p.y - self.padding.top;
// //         // //             if y >= scrollbar_h_rect.y && y < (scrollbar_h_rect.y + scrollbar_h_rect.height) {
// //         // //                 hover = true;
// //         // //             }
// //         // //         }

// //         // //         let mut scrollbar_draw =
// //         // //             scrollbar_h_rect + Vector::new(image_position.x, image_position.y);
// //         // //         if !hover && !pressed {
// //         // //             // Decrease draw width and keep centered when not hovered or pressed
// //         // //             scrollbar_draw.height /= 2.0;
// //         // //             scrollbar_draw.y += scrollbar_draw.height / 2.0;
// //         // //         }

// //         // //         // neutral_6, 0.7
// //         // //         let base_color = cosmic_theme
// //         // //             .palette
// //         // //             .neutral_6
// //         // //             .without_alpha()
// //         // //             .with_alpha(0.7);
// //         // //         let scrollbar_color = if pressed {
// //         // //             // pressed_state_color, 0.5
// //         // //             cosmic_theme
// //         // //                 .background
// //         // //                 .component
// //         // //                 .pressed
// //         // //                 .without_alpha()
// //         // //                 .with_alpha(0.5)
// //         // //                 .over(base_color)
// //         // //         } else if hover {
// //         // //             // hover_state_color, 0.2
// //         // //             cosmic_theme
// //         // //                 .background
// //         // //                 .component
// //         // //                 .hover
// //         // //                 .without_alpha()
// //         // //                 .with_alpha(0.2)
// //         // //                 .over(base_color)
// //         // //         } else {
// //         // //             base_color
// //         // //         };

// //         // //         // Draw scrollbar quad
// //         // //         renderer.fill_quad(
// //         // //             Quad {
// //         // //                 bounds: scrollbar_draw,
// //         // //                 border: Border {
// //         // //                     radius: (scrollbar_draw.height / 2.0).into(),
// //         // //                     width: 0.0,
// //         // //                     color: Color::TRANSPARENT,
// //         // //                 },
// //         // //                 ..Default::default()
// //         // //             },
// //         // //             Color::from(scrollbar_color),
// //         // //         );
// //         // //     }

// //         // //     let duration = instant.elapsed();
// //         // //     log::debug!("redraw {}, {}: {:?}", view_w, view_h, duration);
// //         // // }
// //         ()
// //     }
// // }

// // enum Theme {}

// // enum Style {}

// // struct Canvas {
// //     w: i32,
// //     h: i32,
// // }

// // struct Offset {
// //     x: i32,
// //     y: i32,
// // }

// // /// This function is called canvas.x * canvas.y number of times
// // /// each time the text is scrolled or the canvas is resized.
// // /// If the canvas is moved, it's not called as the pixel buffer
// // /// is the same, it's just translated for the screen's x, y.
// // /// canvas is the location of the pixel in the canvas.
// // /// Screen is the location of the pixel on the screen.
// // // TODO: improve performance
// // fn draw_rect(
// //     buffer: &mut [u32],
// //     canvas: Canvas,
// //     offset: Canvas,
// //     screen: Offset,
// //     cosmic_color: cosmic_text::Color,
// // ) {
// //     // Grab alpha channel and green channel
// //     let mut color = cosmic_color.0 & 0xFF00FF00;
// //     // Shift red channel
// //     color |= (cosmic_color.0 & 0x00FF0000) >> 16;
// //     // Shift blue channel
// //     color |= (cosmic_color.0 & 0x000000FF) << 16;

// //     let alpha = (color >> 24) & 0xFF;
// //     match alpha {
// //         0 => {
// //             // Do not draw if alpha is zero.
// //         }
// //         255 => {
// //             // Handle overwrite
// //             for x in screen.x..screen.x + offset.w {
// //                 if x < 0 || x >= canvas.w {
// //                     // Skip if y out of bounds
// //                     continue;
// //                 }

// //                 for y in screen.y..screen.y + offset.h {
// //                     if y < 0 || y >= canvas.h {
// //                         // Skip if x out of bounds
// //                         continue;
// //                     }

// //                     let line_offset = y as usize * canvas.w as usize;
// //                     let offset = line_offset + x as usize;
// //                     buffer[offset] = color;
// //                 }
// //             }
// //         }
// //         _ => {
// //             let n_alpha = 255 - alpha;
// //             for y in screen.y..screen.y + offset.h {
// //                 if y < 0 || y >= canvas.h {
// //                     // Skip if y out of bounds
// //                     continue;
// //                 }

// //                 let line_offset = y as usize * canvas.w as usize;
// //                 for x in screen.x..screen.x + offset.w {
// //                     if x < 0 || x >= canvas.w {
// //                         // Skip if x out of bounds
// //                         continue;
// //                     }

// //                     // Alpha blend with current value
// //                     let offset = line_offset + x as usize;
// //                     let current = buffer[offset];
// //                     if current & 0xFF000000 == 0 {
// //                         // Overwrite if buffer empty
// //                         buffer[offset] = color;
// //                     } else {
// //                         let rb = ((n_alpha * (current & 0x00FF00FF))
// //                             + (alpha * (color & 0x00FF00FF)))
// //                             >> 8;
// //                         let ag = (n_alpha * ((current & 0xFF00FF00) >> 8))
// //                             + (alpha * (0x01000000 | ((color & 0x0000FF00) >> 8)));
// //                         buffer[offset] = (rb & 0x00FF00FF) | (ag & 0xFF00FF00);
// //                     }
// //                 }
// //             }
// //         }
// //     }
// // }
