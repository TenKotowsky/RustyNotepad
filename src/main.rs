use std::fs::File;
use std::io::{Read, Write};
use druid::widget::{Flex, Button, TextBox};
use druid::{AppLauncher, Data, Lens, Widget, WindowDesc, WidgetExt};
use native_dialog::{FileDialog, MessageDialog, MessageType};
#[derive(Clone, Data, Lens)]
struct HelloState {
    text: String,
    current_file_path: Option<String>,
    window_title: String,
}

fn set_current_file(data: &mut HelloState, path: Option<String>) {
    data.current_file_path = path.clone();
    if let Some(path_str) = path {
        data.window_title = format!("Rusty Notepad {}", path_str);
    } else {
        data.window_title = String::from("Rusty Notepad");
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
        .with_placeholder("Words, words, words")
        .lens(HelloState::text)
        .fix_width(TEXT_BOX_WIDTH)
        .fix_height(TEXT_BOX_HEIGHT);

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
    };

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}
