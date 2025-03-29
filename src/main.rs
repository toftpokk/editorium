use iced::{
    Element, Task,
    widget::{button, column, container, text},
};

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn main() -> Result<(), iced::Error> {
    iced::application("a", App::update, App::view).run_with(App::new)
}

struct App {
    value: u64,
}

impl App {
    fn new() -> (App, Task<Message>) {
        (App { value: 1 }, Task::none())
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.value += 1,
        }
    }

    fn view(&self) -> Element<Message> {
        // button(text(self.value)).on_press(Message::Increment).into()
        container(column!["A", "B"].spacing(10)).into()
    }
}
