#![windows_subsystem = "windows"]
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
use druid::widget::{Flex, Button, TextBox, Controller};
use druid::{AppLauncher, Data, Lens, Widget, WindowDesc, WidgetExt, EventCtx, Event, Env, UpdateCtx};
use native_dialog::{FileDialog, MessageDialog, MessageType};

#[derive(Clone, Data, Lens)]
struct HelloState {
    text: String,
    current_file_path: Option<String>,
    window_title: String,
    history: Arc<RwLock<Vec<String>>>,
    redo_history: Arc<RwLock<Vec<String>>>,
    undo: bool,
    redo: bool,
}

fn set_current_file(data: &mut HelloState, path: Option<String>) {
    data.current_file_path = path.clone();
    if let Some(path_str) = path {
        data.window_title = format!("Rusty Notepad {}", path_str);
    } else {
        data.window_title = String::from("Rusty Notepad");
    }
}

pub struct KeyController;

impl<W: Widget<HelloState>> Controller<HelloState, W> for KeyController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut HelloState, env: &Env) {
        if let Event::KeyDown(key_event) = event {
            if key_event.mods.ctrl() && key_event.key == druid::keyboard_types::Key::Character("z".into()) {
                // Undo
                let mut history = data.history.write().unwrap();
                if let Some(last) = history.pop() {
                    data.redo_history.write().unwrap().push(data.text.clone());
                    data.text = last;
                    data.redo = true;
                }
                if history.is_empty() {
                    data.undo = false;
                }
            } else if key_event.mods.ctrl() && key_event.key == druid::keyboard_types::Key::Character("y".into()) {
                // Redo
                let mut redo_history = data.redo_history.write().unwrap();
                if let Some(next) = redo_history.pop() {
                    data.history.write().unwrap().push(data.text.clone());
                    data.text = next;
                    data.redo = true;
                }
                if redo_history.is_empty() {
                    data.redo = false;
                }
            }
        }
        child.event(ctx, event, data, env);
    }

    fn update(&mut self, child: &mut W, ctx: &mut UpdateCtx<'_, '_>, old_data: &HelloState, data: &HelloState, env: &Env) {
        if old_data.text != data.text && !data.undo && !data.redo {
            let mut history = data.history.write().unwrap();
            history.push(old_data.text.clone());
            data.redo_history.write().unwrap().clear();
        } else if data.text.is_empty() {
            let mut history = data.history.write().unwrap();
            history.clear();
            data.redo_history.write().unwrap().clear();
        }

        let mut data_mut = data.clone();
        data_mut.redo = !data_mut.redo_history.read().unwrap().is_empty();

        child.update(ctx, old_data, &data_mut, env);
    }
}

fn build_root_widget() -> impl Widget<HelloState> {
    const TEXT_BOX_WIDTH: f64 = 1920.0;
    const TEXT_BOX_HEIGHT: f64 = 985.0;
    const BUTTON_WIDTH: f64 = 100.0;
    const BUTTON_HEIGHT: f64 = 50.0;

    fn save_as(data: &mut HelloState) {
        match FileDialog::new()
            .set_filename("TextDocument.txt")
            .add_filter("Text (.txt)", &["txt"])
            .add_filter("Microsoft Word text document (.doc)", &["doc"])
            .add_filter("Rich Text Format (.rtf)", &["rtf"])
            .add_filter("All files", &["*"])
            .show_save_single_file()
        {
            Ok(Some(path)) => {
                match File::create(&path) {
                    Ok(mut file) => {
                        if let Err(err) = file.write_all(data.text.as_bytes()) {
                            eprintln!("Error writing to file: {:?}", err);
                        } else {
                            set_current_file(data, Some(String::from(path.display().to_string())));
                            println!("Text saved to file: {:?}", path);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error creating file: {:?}", err);
                    }
                }
            }
            _ => ()
        }
    }

    let save_handler = |_ctx: &mut druid::EventCtx, _data: &mut HelloState, _env: &druid::Env| {
        if _data.current_file_path.is_none() {
            save_as(_data)
        } else {
            if let Some(ref path) = _data.current_file_path {
                match File::create(path) {
                    Ok(mut file) => {
                        if let Err(err) = file.write_all(_data.text.as_bytes()) {
                            eprintln!("Error writing to file: {:?}", err);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error creating file: {:?}", err);
                    }
                }
            }
        }
    };

    let save_as_handler = |_ctx: &mut druid::EventCtx, _data: &mut HelloState, _env: &druid::Env| {
        save_as(_data)
    };

    let open_handler = |_ctx: &mut druid::EventCtx, _data: &mut HelloState, _env: &druid::Env| {
        match FileDialog::new()
            .add_filter("Text (.txt)", &["txt"])
            .add_filter("Microsoft Word text document (.doc)", &["doc"])
            .add_filter("Rich Text Format (.rtf)", &["rtf"])
            .add_filter("All files", &["*"])
            .show_open_single_file()
        {
            Ok(Some(path)) => {
                match File::open(&path) {
                    Ok(mut file) => {
                        let mut s = String::new();
                        match file.read_to_string(&mut s) {
                            Ok(_) => {
                                _data.text = s;
                                set_current_file(_data, Some(String::from(path.display().to_string())));
                            }
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    };

    let new_handler = |_ctx: &mut druid::EventCtx, _data: &mut HelloState, _env: &druid::Env| {
        if _data.text.len() > 0 {
            match MessageDialog::new()
                .set_type(MessageType::Info)
                .set_title("Are you sure you want to create a new file?")
                .set_text("You will lose any unsaved changes")
                .show_confirm()
                .unwrap()
            {
                true => {
                    let mut history = _data.history.write().unwrap();
                    history.clear();
                    _data.text = String::new();
                    _data.current_file_path = None; // Clear current file path
                }
                _ => ()
            }
        }
    };

    let new_button = Button::new("New")
        .fix_width(BUTTON_WIDTH)
        .fix_height(BUTTON_HEIGHT)
        .on_click(new_handler);

    let save_button = Button::new("Save")
        .fix_width(BUTTON_WIDTH)
        .fix_height(BUTTON_HEIGHT)
        .on_click(save_handler);

    let save_as_button = Button::new("Save As")
        .fix_width(BUTTON_WIDTH)
        .fix_height(BUTTON_HEIGHT)
        .on_click(save_as_handler);

    let open_button = Button::new("Open")
        .fix_width(BUTTON_WIDTH)
        .fix_height(BUTTON_HEIGHT)
        .on_click(open_handler);

    let buttons = Flex::row()
        .with_child(new_button)
        .with_spacer(20.0)
        .with_child(save_button)
        .with_spacer(20.0)
        .with_child(save_as_button)
        .with_spacer(20.0)
        .with_child(open_button)
        .padding((0.0, 10.0));

    let textbox = TextBox::multiline()
        .with_line_wrapping(false)
        .with_placeholder("Words, words, words")
        .lens(HelloState::text)
        .fix_width(TEXT_BOX_WIDTH)
        .fix_height(TEXT_BOX_HEIGHT)
        .controller(KeyController);

    Flex::column()
        .with_child(buttons)
        .with_child(textbox)
}

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(|data: &HelloState, _env: &_| {
            if let Some(ref current_file_path) = data.current_file_path {
                format!("Rusty Notepad ({})", current_file_path)
            } else {
                String::from("Rusty Notepad")
            }
        });
    let initial_state = HelloState {
        text: String::new(),
        current_file_path: None,
        window_title: String::from("Rusty Notepad"),
        history: Arc::new(RwLock::new(Vec::new())),
        redo_history: Arc::new(RwLock::new(Vec::new())), // Initialize redo history
        undo: false,
        redo: false,
    };

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}