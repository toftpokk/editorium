use std::usize;

use iced::{
    Alignment, Element, Font, Length, Padding, Pixels, Task, highlighter,
    widget::{Container, button, column, container, pick_list, row, scrollable, text, text_editor},
};

#[derive(Debug, Clone)]
enum Message {
    Increment,
    ThemeSelected(highlighter::Theme),
    Edit(text_editor::Action),
}

fn main() -> Result<(), iced::Error> {
    iced::application("Editorium", App::update, App::view).run_with(App::new)
}

struct App {
    current_project: String,
    value: u64,
    text_content: text_editor::Content,
}

impl App {
    fn new() -> (App, Task<Message>) {
        (
            App {
                current_project: "none".into(),
                value: 1,
                text_content: text_editor::Content::with_text(
                    "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n27\n28\n29\n30\n31\n32\n33\n34\n35\n36\n37\n38\n39\n40\n41\n42\n43\n44\n45\n46\n47\n48\n49\n50\n51\n52\n53\n54\n55\n56\n57\n58\n59\n60\n61\n62\n63\n64\n65\n66\n67\n68\n69\n70\n71\n72\n73\n74\n75\n76\n77\n78\n79\n80\n81\n82\n83\n84\n85\n86\n87\n88\n89\n90\n9",
                ),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.value += 1,
            Message::ThemeSelected(theme) => {
                println!("{}", theme)
            }
            Message::Edit(action) => {
                self.text_content.perform(action);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // let file_selection = row![]
        // let nav_bar = row![
        //     // current_project
        //     // current git branch
        //     // run
        // ];
        // column![
        // container().width(Length::Fill)
        // nav_bar
        // pick_list(highlighter::Theme::ALL, Some(highlighter::Theme::SolarizedDark), Message::ThemeSelected)
        // ].into()

        let font_size = 15.0;
        let line_height = 1.1;

        let second_scroll_id = iced::widget::scrollable::Id::new("2");

        row![
            line_number(
                self.text_content.line_count(),
                font_size,
                line_height,
                second_scroll_id
            ),
            text_editor(&self.text_content)
                .font(Font::MONOSPACE)
                .size(font_size)
                .line_height(line_height)
                .padding(Padding {
                    top: 0.0,
                    bottom: 0.0,
                    left: 5.0,
                    right: 0.0,
                })
                .height(Length::Fill)
                .on_action(Message::Edit),
        ]
        .into()
    }
}

fn line_number(
    line_count: usize,
    font_size: f32,
    line_height: f32,
    scroller: scrollable::Id,
) -> Element<'static, Message> {
    let mut lines: Vec<Element<Message>> = Vec::new();
    let box_height = text::LineHeight::from(line_height).to_absolute(Pixels(font_size));

    for i in 1..line_count.saturating_add(1) {
        let container = container(
            text(i)
                .font(Font::MONOSPACE)
                .size(font_size)
                .line_height(line_height),
        )
        .width(Length::Fixed(40.0))
        .align_x(Alignment::End)
        .align_y(Alignment::End)
        .height(box_height);
        lines.push(container.into());
    }

    scrollable(column(lines).padding(Padding {
        top: 0.0,
        bottom: 0.0,
        left: 0.0,
        right: 15.0,
    }))
    .id(scroller)
    .height(Length::Fill)
    .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::default().scroller_width(0).width(0),
    ))
    .into()
}
