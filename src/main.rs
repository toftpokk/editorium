use std::{
    cmp::Ordering,
    collections::HashMap,
    fs, io,
    path::PathBuf,
    str::FromStr,
    sync::{OnceLock, RwLock},
};

use clap::Parser;
use iced::{
    Element, Length, Subscription, Task,
    advanced::{graphics::core::keyboard, subscription},
    event,
    futures::{self, SinkExt},
    stream, time,
    widget::{Container, PaneGrid, button, column, pane_grid, pick_list, row, scrollable},
};
use key_binds::KeyBind;
use log::debug;
use rfd::FileDialog;
use smol::channel;

mod cli;
mod font;
mod key_binds;
mod lsp;
mod project;
mod tab;
mod text_box;
mod theme;

// TODO move
static FONT_SYSTEM: OnceLock<RwLock<cosmic_text::FontSystem>> = OnceLock::new();
static SYNTAX_SYSTEM: OnceLock<cosmic_text::SyntaxSystem> = OnceLock::new();
static SWASH_CACHE: OnceLock<RwLock<cosmic_text::SwashCache>> = OnceLock::new();
static KEY_BINDINGS: OnceLock<HashMap<KeyBind, Message>> = OnceLock::new();

fn font_system() -> &'static RwLock<cosmic_text::FontSystem> {
    FONT_SYSTEM.get().unwrap()
}

fn swash_cache() -> &'static RwLock<cosmic_text::SwashCache> {
    SWASH_CACHE.get().unwrap()
}

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(keyboard::Modifiers, keyboard::Key),
    OpenFileSelector,
    OpenDirectorySelector,
    OpenFile(PathBuf),
    OpenProject(PathBuf),
    TabSelected(usize),
    TabClose(usize),
    TabCloseCurrent,
    TabSearch(String),
    TabSearchOpen,
    TabSearchClose,
    PaneResized(pane_grid::ResizeEvent),
    ProjectTreeSelect(usize),
    SaveFile,
    AutoScroll,
    SetAutoScroll(Option<f32>),
    LSPMessage(String),
    LSPInitialized(()), // TODO rename
    Special(Result<(), channel::SendError<String>>),
}

fn main() -> Result<(), iced::Error> {
    FONT_SYSTEM.get_or_init(|| RwLock::new(cosmic_text::FontSystem::new()));
    SYNTAX_SYSTEM.get_or_init(|| cosmic_text::SyntaxSystem::new());
    SWASH_CACHE.get_or_init(|| RwLock::new(cosmic_text::SwashCache::new()));
    KEY_BINDINGS.get_or_init(|| key_binds::default());

    // use editorium=debug to get only this crate
    env_logger::init();
    iced::application("Editorium", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .settings(iced::Settings {
            fonts: font::load(),
            ..Default::default()
        })
        .run_with(App::new)
}

struct Pane {
    pane_type: PaneType,
}

#[derive(PartialEq)]
enum PaneType {
    FileTree,
    Editor,
}

impl Pane {
    fn new(pane_type: PaneType) -> Self {
        Self {
            pane_type: pane_type,
        }
    }
}

struct App {
    tabs: tab::TabView,
    project_tree: project::ProjectTree,
    current_project: Option<project::Project>,
    panes: pane_grid::State<Pane>,
    auto_scroll: Option<f32>,
    lsp_client: lsp::Client,
}

fn create_pane() -> pane_grid::State<Pane> {
    let (mut pane_grid_state, pane) = pane_grid::State::new(Pane::new(PaneType::FileTree));

    let (_, split) = pane_grid_state
        .split(pane_grid::Axis::Vertical, pane, Pane::new(PaneType::Editor))
        .unwrap();

    pane_grid_state.resize(split, 0.2);

    pane_grid_state
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let cli = cli::Cli::parse();

        KEY_BINDINGS.get_or_init(|| key_binds::default());

        let mut app = Self {
            tabs: tab::TabView::new(),
            project_tree: project::ProjectTree::new(),
            current_project: None,
            panes: create_pane(),
            auto_scroll: None,
            lsp_client: lsp::Client::new(),
        };

        let task = if let Some(path) = cli.path {
            if path.is_dir() {
                app.open_project(path);
                Task::none()
            } else {
                app.open_file(path.clone()).unwrap()
            }
        } else {
            Task::none()
        };

        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFileSelector => {
                let mut lsp = self.prepare_lsp();
                self.lsp_client = lsp;
                let msg = self.lsp_client.initialize();
                let w = self.lsp_client.new_writer();
                return Task::perform(async move { w.write(msg.unwrap()).await }, Message::Special);
                // if let Some(file_path) =
                //     select_file(&self.current_project.as_ref().map(|p| p.path.clone()))
                // {
                //     return self.open_file(file_path).unwrap();
                // }
            }
            Message::OpenDirectorySelector => {
                if let Some(dir_path) =
                    select_dir(&self.current_project.as_ref().map(|p| p.path.clone()))
                {
                    self.open_project(dir_path);
                }
            }
            Message::TabSelected(tab) => {
                self.tabs.activate(tab);
                self.redraw_active_editor();
            }
            Message::OpenProject(project) => {
                self.open_project(project);
            }
            Message::OpenFile(file_path) => return self.open_file(file_path).unwrap(),
            Message::SaveFile => {
                if let Some(active) = self.tabs.active() {
                    let tab = self.tabs.tab_mut(active).unwrap();
                    match tab.save() {
                        Ok(_) => {}
                        Err(err) => log::error!("could not open directory: {}", err),
                    }
                };
            }
            Message::TabCloseCurrent => {
                if let Some(active) = self.tabs.active() {
                    self.tabs.remove(active);
                    self.redraw_active_editor();
                }
            }
            Message::TabClose(tab) => {
                self.tabs.remove(tab);
                self.redraw_active_editor();
            }
            Message::TabSearch(text) => {
                if let Some(active) = self.tabs.active() {
                    let tab = self.tabs.tab_mut(active).unwrap();
                    return tab.search_open(Some(text));
                }
            }
            Message::TabSearchOpen => {
                if let Some(active) = self.tabs.active() {
                    let tab = self.tabs.tab_mut(active).unwrap();
                    return tab.search_open(None);
                }
            }
            Message::TabSearchClose => {
                if let Some(active) = self.tabs.active() {
                    let tab = self.tabs.tab_mut(active).unwrap();
                    return tab.search_close();
                }
            }
            Message::KeyPressed(modifier, key) => {
                if let Some(value) = KEY_BINDINGS.get().unwrap().get(&key_binds::KeyBind {
                    modifiers: modifier,
                    key: key,
                }) {
                    return self.update(value.clone());
                }
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }
            Message::ProjectTreeSelect(id) => {
                let node = if let Some(node) = self.project_tree.node_mut(id) {
                    if let project::NodeKind::Directory { open } = &mut node.kind {
                        *open = !*open;
                    }
                    Some(node.clone())
                } else {
                    None
                };

                let pos = self.project_tree.position(id).unwrap();

                if let Some(node) = node {
                    match node.kind {
                        project::NodeKind::Directory { open } => {
                            if open {
                                self.tree_open_dir(node.path, pos, node.indent + 1);
                            } else {
                                while let Some(child) = self.project_tree.node_at(pos + 1) {
                                    if child.indent > node.indent {
                                        self.project_tree.remove(child.id);
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                        project::NodeKind::File => return self.open_file(node.path).unwrap(),
                    }
                }
            }
            Message::AutoScroll => {
                if let Some(auto_scroll) = self.auto_scroll {
                    if let Some(active) = self.tabs.active() {
                        let tab = self.tabs.tab_mut(active).unwrap();

                        tab.scroll(auto_scroll)
                    }
                }
            }
            Message::SetAutoScroll(auto_scroll) => {
                self.auto_scroll = auto_scroll;
            }
            Message::Special(_) => {
                log::info!("hello")
            }
            Message::LSPMessage(msg) => {
                log::info!("{}", msg)
            }
            #[allow(unreachable_patterns)]
            _ => {
                todo!()
            }
        }
        Task::none()
    }

    // use mytheme as Theme
    fn view(&self) -> Element<Message, theme::MyTheme> {
        let cwd = PathBuf::from_str("./").expect("could not get cwd");
        let cwd = match fs::canonicalize(cwd) {
            Ok(ok) => ok,
            Err(err) => {
                panic!("could not open directory: {}", err);
            }
        };

        let recent_projects = vec![project::Project::new(cwd)];
        let nav_bar = row![
            pick_list(
                recent_projects,
                self.current_project.clone(),
                |project: project::Project| Message::OpenProject(project.path),
            )
            .handle(pick_list::Handle::Dynamic {
                closed: pick_list::Icon {
                    font: font::ICON_SOLID,
                    code_point: font::arrow_left(),
                    size: None,
                    line_height: iced::widget::text::LineHeight::default(),
                    shaping: iced::widget::text::Shaping::Basic,
                },
                open: pick_list::Icon {
                    font: font::ICON_SOLID,
                    // todo: list of codepoints used
                    code_point: font::arrow_dowwn(),
                    size: None,
                    line_height: iced::widget::text::LineHeight::default(),
                    shaping: iced::widget::text::Shaping::Basic,
                }
            })
            .placeholder("Choose a Project"),
            button("Open File").on_press(Message::OpenFileSelector),
            button("Open Dir").on_press(Message::OpenDirectorySelector) //     // current_project
                                                                        //     // current git branch
                                                                        //     // run
        ];

        let pane_grid = PaneGrid::new(&self.panes, |_, state, _| {
            if state.pane_type == PaneType::Editor {
                pane_grid::Content::new(self.tabs.view())
            } else {
                let file_tree = self.project_tree.view();

                pane_grid::Content::new(
                    scrollable(Container::new(file_tree))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .direction(scrollable::Direction::Both {
                            vertical: scrollable::Scrollbar::default().scroller_width(0).width(0),
                            horizontal: scrollable::Scrollbar::default().scroller_width(0).width(0),
                        }),
                )
            }
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .on_resize(10, Message::PaneResized);

        let content: Element<Message, theme::MyTheme> = column![nav_bar, pane_grid].into();

        // content.explain(iced::Color::from_rgb(1.0, 0.0, 0.0))
        content
    }

    // note: events seem to call on everybody's on_event, with subscription last
    // explanation: this function is run when necessary i.e. "show me (iced) which subscriptions are still ongoing"
    // NOT "show me all events then I will check subscriptions"
    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![event::listen_with(|event, status, _| match event {
            event::Event::Keyboard(keyboard::Event::KeyPressed { modifiers, key, .. }) => {
                match status {
                    event::Status::Captured => None,
                    event::Status::Ignored => Some(Message::KeyPressed(modifiers, key)),
                }
            }
            _ => None,
        })];

        // subscription::run takes in a function that returns a stream of messages
        if self.lsp_client.is_connected() {
            subscriptions.push(Subscription::run_with_id(
                0,
                lsp_worker(self.lsp_client.new_reader()),
            ));
        }
        // if let Some(client) = &self.lsp_client {
        //     if let Some(transport) = client.new_transport_receiver() {
        //         subscriptions.push(Subscription::run_with_id(0, lsp_worker(transport)));
        //     }
        // }
        // TODO maybe give ownership of Stdout reader to subscription?
        // if let Some(client) = &self.lsp_client.stdin {
        //     subscriptions.push(Subscription::run_with_id(0, lsp_worker(transport)));
        // }

        if let Some(_) = self.auto_scroll {
            subscriptions
                .push(time::every(time::Duration::from_millis(10)).map(|_| Message::AutoScroll));
        }

        Subscription::batch(subscriptions)
    }

    // use mytheme as Theme
    fn theme(&self) -> theme::MyTheme {
        theme::MyTheme::default()
    }

    fn open_project(&mut self, path: PathBuf) {
        let path = fs::canonicalize(&path).expect("could not canonicalize");
        self.current_project = Some(project::Project::new(path.clone()));
        self.project_tree.clear();
        self.project_tree.insert(path, 0, 0);
    }

    fn prepare_lsp(&self) -> lsp::Client {
        let mut lsp_client = lsp::Client::new();
        lsp_client
            .connect(PathBuf::from("file:///src/tab.rs"))
            .unwrap();
        lsp_client
    }

    // async fn do_lsp() {
    //     let mut lsp_client = lsp::Client::new();
    //     lsp_client.initialize().await;

    //     let mut buf = vec![0; 1024];
    //     let size = lsp_client.read(&mut buf).await.unwrap();
    //     debug!("hello {:?} {}", buf, size)
    // }

    fn open_file(&mut self, file_path: PathBuf) -> io::Result<Task<Message>> {
        let file_path = fs::canonicalize(&file_path).expect("could not canonicalize");
        if let Some(pos) = self.tabs.position(file_path.clone()) {
            self.tabs.activate(pos);
            self.redraw_active_editor();
            return Ok(Task::none());
        }
        let index = self.tabs.insert(Some(file_path.clone()))?;
        self.tabs.activate(index);
        self.redraw_active_editor();
        // self.
        // // TODO transport falls out of scope when task is done -> cannot read replies from server
        //
        // self.lsp_client = Some(lsp_client);
        Ok(Task::none())
    }

    fn redraw_active_editor(&mut self) {
        if let Some(active) = self.tabs.active() {
            let tab = self.tabs.tab_mut(active).unwrap();
            tab.redraw();
        }
    }

    fn tree_open_dir(&mut self, path: PathBuf, pos: usize, indent: usize) {
        let path = fs::canonicalize(&path).expect("could not canonicalize");

        let read_dir = match fs::read_dir(path) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("could not open directory: {}", err);
                return;
            }
        };
        let mut nodes = Vec::new();
        for dir_entry in read_dir {
            match dir_entry {
                Ok(ok) => {
                    nodes.push(ok.path());
                }
                Err(err) => {
                    log::error!("could not read directory entry: {}", err);
                    return;
                }
            }
        }
        nodes.sort_by(|a, b| {
            if a.is_dir() && !b.is_dir() {
                return Ordering::Less;
            }
            if b.is_dir() && !a.is_dir() {
                return Ordering::Greater;
            }
            a.cmp(b)
        });

        let mut position = pos + 1;
        for node in nodes {
            self.project_tree.insert(node, position, indent);
            position += 1;
        }
    }
}

fn select_dir(working_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = FileDialog::new().set_title("Open a directory...");

    if let Some(working_dir) = working_dir {
        dialog = dialog.set_directory(working_dir)
    }

    if let Some(dir) = dialog.pick_folder() {
        return Some(dir);
    }
    return None;
}

fn select_file(working_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = FileDialog::new().set_title("Open a file...");

    if let Some(working_dir) = working_dir {
        dialog = dialog.set_directory(working_dir)
    }

    if let Some(file) = dialog.pick_file() {
        return Some(file);
    }
    return None;
}

// reads from channel and sends Message::LSPMessage
fn lsp_worker(reader: lsp::Reader) -> impl futures::Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        loop {
            let msg = reader.read().await.unwrap();
            output.send(Message::LSPMessage(msg)).await.unwrap();
        }
    })
}
