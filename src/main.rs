use crate::consts::{HIGHLIGHT_PAIR, REGULAR_PAIR};
use crate::ui::Ui;
use directories::ProjectDirs;
use layout::LayoutKind;
use ncurses::*;
use status::Status;
use std::fs::{self, File};
use std::io::{self, BufRead, ErrorKind, Write};
use std::path::PathBuf;
use std::process;
use vec2::Vec2;

mod consts;
mod ctrlc;
mod layout;
mod status;
mod ui;
mod vec2;

fn parse_item(line: &str) -> Option<(Status, &str)> {
    let todo_item = line
        .strip_prefix("TODO: ")
        .map(|title| (Status::Todo, title));
    let done_item = line
        .strip_prefix("DONE: ")
        .map(|title| (Status::Done, title));
    todo_item.or(done_item)
}

fn list_drag_up(list: &mut [String], list_curr: &mut usize) {
    if *list_curr > 0 {
        list.swap(*list_curr, *list_curr - 1);
        *list_curr -= 1;
    }
}

fn list_drag_down(list: &mut [String], list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        list.swap(*list_curr, *list_curr + 1);
        *list_curr += 1;
    }
}

fn list_up(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr -= 1;
    }
}

fn list_down(list: &[String], list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        *list_curr += 1;
    }
}

fn list_first(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr = 0;
    }
}

fn list_last(list: &[String], list_curr: &mut usize) {
    if !list.is_empty() {
        *list_curr = list.len() - 1;
    }
}

fn list_transfer(
    list_dst: &mut Vec<String>,
    list_src: &mut Vec<String>,
    list_src_curr: &mut usize,
) {
    if *list_src_curr < list_src.len() {
        list_dst.push(list_src.remove(*list_src_curr));
        if *list_src_curr >= list_src.len() && !list_src.is_empty() {
            *list_src_curr = list_src.len() - 1;
        }
    }
}

fn list_delete(list: &mut Vec<String>, list_curr: &mut usize) {
    if *list_curr < list.len() {
        list.remove(*list_curr);
        if *list_curr >= list.len() && !list.is_empty() {
            *list_curr = list.len() - 1;
        }
    }
}

fn load_state(
    todos: &mut Vec<String>,
    dones: &mut Vec<String>,
    file_path: &PathBuf,
) -> io::Result<()> {
    let file = File::open(file_path)?;
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line?) {
            Some((Status::Todo, title)) => todos.push(title.to_string()),
            Some((Status::Done, title)) => dones.push(title.to_string()),
            None => {
                eprintln!(
                    "{}:{}: ERROR: ill-formed item line",
                    file_path.display(),
                    index + 1
                );
                process::exit(1);
            }
        }
    }
    Ok(())
}

fn save_state(todos: &[String], dones: &[String], file_path: &PathBuf) {
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        writeln!(file, "TODO: {}", todo).unwrap();
    }
    for done in dones.iter() {
        writeln!(file, "DONE: {}", done).unwrap();
    }
}

fn usage() {
    let usage = "Usage: todo [OPTIONS]

Options:
    --help      Print this help message

Controls (Vim-style keymaps):
+------------+-------------------------------------------------+
| Key        | Description                                     |
+------------+-------------------------------------------------+
| k, j       | Move cursor up and down                         |
| K, J       | Drag the current item up and down               |
| h, l       | Switch between TODO (left) and DONE (right)     |
| g, G       | Jump to the start, end of the current item list |
| r          | Rename the current item                         |
| i          | Insert a new item at cursor position            |
| o          | Insert a new item below cursor (vim 'o')        |
| O          | Insert a new item above cursor (vim 'O')        |
| c          | Change item (clear and enter insert mode)       |
| C          | Change entire line (clear and enter insert)     |
| d, x       | Delete the current list item                    |
| q          | Quit                                            |
| TAB        | Switch between the TODO and DONE panels         |
| Enter      | Move item between TODO and DONE                 |
| ESC        | Exit insert/rename mode (back to normal mode)   |
+------------+-------------------------------------------------+
";
    println!("{}", usage);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--help".to_string()) {
        usage();
        process::exit(0);
    }

    ctrlc::init();

    let file_path = if let Some(proj_dirs) = ProjectDirs::from("", "", "todo") {
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            if let Err(e) = fs::create_dir_all(data_dir) {
                eprintln!("Could not create data directory: {}", e);
                process::exit(1);
            }
        }
        data_dir.join("TODO")
    } else {
        PathBuf::from("TODO")
    };

    let mut todos = Vec::<String>::new();
    let mut todo_curr: usize = 0;
    let mut dones = Vec::<String>::new();
    let mut done_curr: usize = 0;

    let mut notification: String;

    match load_state(&mut todos, &mut dones, &file_path) {
        Ok(()) => notification = format!("Loaded file {}", file_path.display()),
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                notification = format!("New file {}", file_path.display())
            } else {
                panic!(
                    "Could not load state from file `{}`: {:?}",
                    file_path.display(),
                    error
                );
            }
        }
    };

    initscr();
    noecho();
    keypad(stdscr(), true);
    timeout(16); // running in 60 FPS for better gaming experience
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    let mut quit = false;
    let mut panel = Status::Todo;
    let mut editing = false;
    let mut editing_cursor = 0;

    let mut ui = Ui::default();
    while !quit && !ctrlc::poll() {
        erase();

        let mut x = 0;
        let mut y = 0;
        getmaxyx(stdscr(), &mut y, &mut x);

        ui.begin(Vec2::new(0, 0), LayoutKind::Vert);
        {
            ui.label_fixed_width(&notification, x, REGULAR_PAIR);
            ui.label_fixed_width("", x, REGULAR_PAIR);

            ui.begin_layout(LayoutKind::Horz);
            {
                ui.begin_layout(LayoutKind::Vert);
                {
                    if panel == Status::Todo {
                        ui.label_fixed_width("TODO", x / 2, HIGHLIGHT_PAIR);
                        for (index, todo) in todos.iter_mut().enumerate() {
                            if index == todo_curr {
                                if editing {
                                    ui.edit_field(todo, &mut editing_cursor, x / 2);

                                    if let Some(key) = ui.key.take() {
                                        // Enter or ESC exits insert/rename mode
                                        if key as u8 as char == '\n' || key == 27 {
                                            editing = false;
                                        }
                                    }
                                } else {
                                    ui.label_fixed_width(
                                        &format!("- [ ] {}", todo),
                                        x / 2,
                                        HIGHLIGHT_PAIR,
                                    );
                                    if let Some('r') = ui.key.map(|x| x as u8 as char) {
                                        editing = true;
                                        editing_cursor = todo.len();
                                        ui.key = None;
                                    }
                                }
                            } else {
                                ui.label_fixed_width(
                                    &format!("- [ ] {}", todo),
                                    x / 2,
                                    REGULAR_PAIR,
                                );
                            }
                        }

                        if let Some(key) = ui.key.take() {
                            match key as u8 as char {
                                'K' => list_drag_up(&mut todos, &mut todo_curr),
                                'J' => list_drag_down(&mut todos, &mut todo_curr),
                                'i' => {
                                    todos.insert(todo_curr, String::new());
                                    editing_cursor = 0;
                                    editing = true;
                                    notification.push_str("What needs to be done?");
                                }
                                'o' => {
                                    // Insert below current item (vim 'o')
                                    let insert_pos = if todos.is_empty() { 0 } else { todo_curr + 1 };
                                    todos.insert(insert_pos, String::new());
                                    todo_curr = insert_pos;
                                    editing_cursor = 0;
                                    editing = true;
                                    notification.push_str("What needs to be done?");
                                }
                                'O' => {
                                    // Insert above current item (vim 'O')
                                    todos.insert(todo_curr, String::new());
                                    editing_cursor = 0;
                                    editing = true;
                                    notification.push_str("What needs to be done?");
                                }
                                'c' => {
                                    // Change current item (vim 'c' - clear and enter insert mode)
                                    if todo_curr < todos.len() {
                                        todos[todo_curr].clear();
                                        editing_cursor = 0;
                                        editing = true;
                                        notification.push_str("Change item...");
                                    }
                                }
                                'C' => {
                                    // Change entire line (vim 'C' - clear entire item and enter insert mode)
                                    if todo_curr < todos.len() {
                                        todos[todo_curr].clear();
                                        editing_cursor = 0;
                                        editing = true;
                                        notification.push_str("Change item...");
                                    }
                                }
                                'd' | 'x' => {
                                    list_delete(&mut todos, &mut todo_curr);
                                    notification.push_str("Into The Abyss!");
                                }
                                'k' => list_up(&mut todo_curr),
                                'j' => list_down(&todos, &mut todo_curr),
                                'g' => list_first(&mut todo_curr),
                                'G' => list_last(&todos, &mut todo_curr),
                                '\n' => {
                                    list_transfer(&mut dones, &mut todos, &mut todo_curr);
                                    notification.push_str("DONE!")
                                }
                                '\t' | 'l' => {
                                    panel = panel.toggle();
                                }
                                'h' => {
                                    // Already in TODO (left panel), stay here
                                }
                                _ => {
                                    ui.key = Some(key);
                                }
                            }
                        }
                    } else {
                        ui.label_fixed_width("TODO", x / 2, REGULAR_PAIR);
                        for todo in todos.iter() {
                            ui.label_fixed_width(&format!("- [ ] {}", todo), x / 2, REGULAR_PAIR);
                        }
                    }
                }
                ui.end_layout();

                ui.begin_layout(LayoutKind::Vert);
                {
                    if panel == Status::Done {
                        ui.label_fixed_width("DONE", x / 2, HIGHLIGHT_PAIR);
                        for (index, done) in dones.iter_mut().enumerate() {
                            if index == done_curr {
                                if editing {
                                    ui.edit_field(done, &mut editing_cursor, x / 2);

                                    if let Some(key) = ui.key.take() {
                                        // Enter or ESC exits insert/rename mode
                                        if key as u8 as char == '\n' || key == 27 {
                                            editing = false;
                                        }
                                    }
                                } else {
                                    ui.label_fixed_width(
                                        &format!("- [x] {}", done),
                                        x / 2,
                                        HIGHLIGHT_PAIR,
                                    );
                                    if let Some('r') = ui.key.map(|x| x as u8 as char) {
                                        editing = true;
                                        editing_cursor = done.len();
                                        ui.key = None;
                                    }
                                }
                            } else {
                                ui.label_fixed_width(
                                    &format!("- [x] {}", done),
                                    x / 2,
                                    REGULAR_PAIR,
                                );
                            }
                        }

                        if let Some(key) = ui.key.take() {
                            match key as u8 as char {
                                'K' => list_drag_up(&mut dones, &mut done_curr),
                                'J' => list_drag_down(&mut dones, &mut done_curr),
                                'k' => list_up(&mut done_curr),
                                'j' => list_down(&dones, &mut done_curr),
                                'g' => list_first(&mut done_curr),
                                'G' => list_last(&dones, &mut done_curr),
                                'i' | 'o' | 'O' => {
                                    notification.push_str(
                                        "Can't insert new DONE items. Only TODO is allowed.",
                                    );
                                }
                                'c' => {
                                    // Change current item (vim 'c' - clear and enter insert mode)
                                    if done_curr < dones.len() {
                                        dones[done_curr].clear();
                                        editing_cursor = 0;
                                        editing = true;
                                        notification.push_str("Change item...");
                                    }
                                }
                                'C' => {
                                    // Change entire line (vim 'C' - clear entire item and enter insert mode)
                                    if done_curr < dones.len() {
                                        dones[done_curr].clear();
                                        editing_cursor = 0;
                                        editing = true;
                                        notification.push_str("Change item...");
                                    }
                                }
                                'd' | 'x' => {
                                    list_delete(&mut dones, &mut done_curr);
                                    notification.push_str("Into The Abyss!");
                                }
                                '\n' => {
                                    list_transfer(&mut todos, &mut dones, &mut done_curr);
                                    notification.push_str("No, not done yet...")
                                }
                                '\t' | 'h' => {
                                    panel = panel.toggle();
                                }
                                'l' => {
                                    // Already in DONE (right panel), stay here
                                }
                                _ => ui.key = Some(key),
                            }
                        }
                    } else {
                        ui.label_fixed_width("DONE", x / 2, REGULAR_PAIR);
                        for done in dones.iter() {
                            ui.label_fixed_width(&format!("- [x] {}", done), x / 2, REGULAR_PAIR);
                        }
                    }
                }
                ui.end_layout();
            }
            ui.end_layout();
        }
        ui.end();

        if let Some('q') = ui.key.take().map(|x| x as u8 as char) {
            quit = true;
        }

        refresh();

        let key = getch();
        if key != ERR {
            notification.clear();
            ui.key = Some(key);
        }
    }

    endwin();

    save_state(&todos, &dones, &file_path);
    println!("Saved state to {}", file_path.display());
}

