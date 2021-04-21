use druid::widget::prelude::*;
use druid::widget::{Flex, Label, WidgetExt};
use druid::{AppLauncher, Color, Data, Lens, UnitPoint, WindowDesc};

use print::editor::EditView;

use crate::components::icon_button::IconButton;
use crate::delegate::Delegate;
use crate::print::tool_window::project_tool_window::{FileEntry, ProjectToolWindow};
use std::path::{Path, PathBuf};
use std::sync::Arc;
pub use support::line;
use walkdir::{DirEntry, WalkDir};

pub mod command;
pub mod components;
pub mod delegate;
pub mod menu;
pub mod print;
pub mod support;
pub mod theme;

const LIGHTER_GREY: Color = Color::rgb8(242, 242, 242);

#[derive(Clone, Data, Lens)]
struct AppState {
    title: String,
    workspace: Workspace,
    params: Params,
    entry: FileEntry,
}

#[derive(Clone, Data, Lens)]
struct Workspace {
    pub current_file: Option<Arc<Path>>,
    pub current_dir: Option<Arc<Path>>,
    pub input_text: String,
    pub entry: FileEntry,
}

impl Workspace {
    pub fn set_file(&mut self, path: impl Into<Option<PathBuf>>) {
        let path = path.into().map(Into::into);
        if let Some(dir) = &path {
            self.entry = self.path_to_tree(dir);
        }
        self.current_file = path;
    }

    pub fn set_dir(&mut self, path: impl Into<Option<PathBuf>>) {
        let path = path.into().map(Into::into);
        if let Some(dir) = &path {
            self.entry = self.path_to_tree(dir);
            log::info!("open dir: {:?}", dir);
        }
        self.current_dir = path;
    }

    fn path_to_tree(&mut self, dir: &Arc<Path>) -> FileEntry {
        fn is_hidden(entry: &DirEntry) -> bool {
            if entry.file_type().is_file() {
                return false;
            }

            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        }

        let _buf = dir.to_path_buf();
        let root = FileEntry::new("".to_string());

        let walker = WalkDir::new(dir).into_iter();

        let mut last_root = root;
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry.unwrap();
            let file_name = entry.file_name().to_os_string();
            if entry.file_type().is_dir() {
                //
            }

            last_root
                .children
                .push(FileEntry::new(format!("{:?}", file_name)));
        }

        last_root
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            current_file: None,
            current_dir: None,
            input_text: "".to_string(),
            entry: Default::default(),
        }
    }
}

#[derive(Clone, Data, Lens)]
struct Params {
    debug_layout: bool,
}

fn navigation_bar() -> impl Widget<AppState> {
    let label = Label::new(|data: &Workspace, _: &Env| match &data.current_dir {
        None => {
            format!("")
        }
        Some(path) => {
            format!("{}", path.to_owned().display())
        }
    });

    Flex::row()
        .with_child(label.with_text_color(Color::BLACK))
        .padding(10.0)
        .expand_width()
        .lens(AppState::workspace)
        .background(line::hline())
        .align_horizontal(UnitPoint::LEFT)
}

fn status_bar() -> impl Widget<AppState> {
    let label = Label::new("status bar").with_text_color(Color::BLACK);
    Flex::row()
        .with_default_spacer()
        .with_flex_child(label, 1.0)
        .with_default_spacer()
        .with_flex_child(Label::new("time").with_text_color(Color::BLACK), 1.0)
        .lens(AppState::params)
        .padding(5.0)
        .align_horizontal(UnitPoint::LEFT)
}

fn bottom_tool_window() -> impl Widget<AppState> {
    let text = "Run";
    let label = Label::new(text).with_text_color(Color::BLACK);
    let button = IconButton::from_label(label);
    Flex::row()
        .with_default_spacer()
        .with_flex_child(button, 1.0)
        .lens(AppState::params)
        .background(line::hline())
}

fn center() -> impl Widget<AppState> {
    Flex::row()
        .with_flex_child(ProjectToolWindow::new(), 1.0)
        .with_default_spacer()
        .with_flex_child(EditView::new().center(), 1.0)
        .padding(1.0)
        .background(line::hline())
}

fn make_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(navigation_bar())
        .with_flex_child(center(), 1.0)
        .with_child(bottom_tool_window())
        .with_child(status_bar())
        .background(LIGHTER_GREY)
}

pub fn main() {
    let title = "Print UI";

    let menu = menu::menus();

    let main_window = WindowDesc::new(crate::theme::wrap_in_theme_loader(make_ui()))
        .window_size((720., 600.))
        .with_min_size((620., 300.))
        .menu(menu)
        .title(title);

    let params = Params {
        debug_layout: false,
    };

    let init_state = AppState {
        title: title.to_string(),
        workspace: Workspace::default(),
        params,
        entry: Default::default(),
    };

    AppLauncher::with_window(main_window)
        .delegate(Delegate::default())
        .log_to_console()
        .launch(init_state)
        .expect("Failed to launch application");
}
