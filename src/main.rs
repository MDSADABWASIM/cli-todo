use ncurses::{
    getch,
    ll::{curs_set, endwin, erase, getmaxy, init_pair, refresh, start_color},
    *,
};
use std::{
    cmp, env,
    fs::File,
    io::{self, BufRead, Write},
    ops::{Add, Mul},
    process, usize,
};

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

type Id = usize;

#[derive(Default, Clone, Copy)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Mul for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Vec2 {
    fn new(row: i32, col: i32) -> Self {
        Self { x: row, y: col }
    }
}
enum LayoutKind {
    Vert,
    Horz,
}

struct Layout {
    kind: LayoutKind,
    size: Vec2,
    pos: Vec2,
}

impl Layout {
    fn new(kind: LayoutKind, pos: Vec2) -> Self {
        Self {
            kind,
            pos,
            size: Vec2::new(0, 0),
        }
    }

    fn available_pos(&self) -> Vec2 {
        use LayoutKind::*;
        match self.kind {
            Vert => self.pos + self.size * Vec2::new(0, 1),
            Horz => self.pos + self.size * Vec2::new(1, 0),
        }
    }

    fn add_widget(&mut self, size: Vec2) {
        use LayoutKind::*;
        match self.kind {
            Vert => {
                self.size.y += size.y;
                self.size.x = cmp::max(self.size.x, size.x)
            }
            Horz => {
                self.size.x += size.x;
                self.size.y = cmp::max(self.size.y, size.y)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Status {
    Todo,
    Done,
}

impl Status {
    fn toggle(&self) -> Self {
        match self {
            Status::Todo => Status::Done,
            Status::Done => Status::Todo,
        }
    }
}

#[derive(Default)]
struct Ui {
    layouts: Vec<Layout>,
}

impl Ui {
    fn begin(&mut self, pos: Vec2, kind: LayoutKind) {
        assert!(self.layouts.is_empty());
        self.layouts.push(Layout {
            kind,
            size: Vec2::new(0, 0),
            pos,
        })
    }

    fn begin_layout(&mut self, layout_kind: LayoutKind) {
        let layout = self
            .layouts
            .last()
            .expect("Can't create a layout outside of Ui::begin or Ui::end");
        let pos = layout.available_pos();
        self.layouts.push(Layout {
            kind: layout_kind,
            size: Vec2::new(0, 0),
            pos,
        })
    }

    fn end_layout(&mut self) {
        let layout = self
            .layouts
            .pop()
            .expect("Unbalanced Ui::begin_layout or Ui::end_layout calls");
        self.layouts
            .last_mut()
            .expect("Unbalanced Ui::begin_layout or Ui::end_layout calls")
            .add_widget(layout.size);
    }

    fn label_fix_width(&mut self, text: &str, width: i32, pair: i16) {
        let layout = self
            .layouts
            .last_mut()
            .expect("Trying to render label outside of any layout");
        let pos = layout.available_pos();
        mv(pos.y, pos.x);
        attron(COLOR_PAIR(pair));
        addstr(text);
        attroff(COLOR_PAIR(pair));
        layout.add_widget(Vec2::new(width, 1));
    }
    fn label(&mut self, text: &str, pair: i16) {
        self.label_fix_width(text, text.len() as i32, pair);
    }

    fn end(&mut self) {
        self.layouts
            .pop()
            .expect("Unbalanced Ui::begin() or Ui::end() calls");
    }
}

fn load_state(todos: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) {
    let file = File::open(file_path).unwrap();
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line.unwrap()) {
            Some((Status::Todo, title)) => todos.push(title.to_string()),
            Some((Status::Done, title)) => dones.push(title.to_string()),
            None => {
                eprintln!("{} {}: Error ill-formed item line", file_path, index + 1);
                process::exit(1);
            }
        }
    }
}

fn save_state(todos: &[String], dones: &[String], file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        writeln!(file, "TODO: {}", todo).unwrap();
    }
    for done in dones.iter() {
        writeln!(file, "DONE: {}", done).unwrap();
    }
}

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
fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let file_path = match args.next() {
        Some(file_path) => file_path,
        None => {
            eprintln!("Usage: cli-todo app");
            eprintln!("Error: file-path not provided");
            process::exit(1);
        }
    };
    let mut dones = Vec::<String>::new();
    let mut todos = Vec::<String>::new();
    let mut todo_curr = 0;
    let mut done_curr = 0;

    load_state(&mut todos, &mut dones, &file_path);

    initscr();
    noecho();
    let mut quit = false;
    unsafe {
        curs_set(0);
        start_color();
        init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
        init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);
        let mut panel = Status::Todo;
        let mut ui = Ui::default();
        while !quit {
            erase();

            let mut x = 0;
            let mut y = 0;
            getmaxyx(stdscr(), &mut y, &mut x);

            ui.begin(Vec2::new(0, 0), LayoutKind::Horz);
            {
                ui.begin_layout(LayoutKind::Vert);
                {
                    if panel == Status::Todo {
                        ui.label_fix_width("[TODO]", x / 2, REGULAR_PAIR);
                    } else {
                        ui.label_fix_width(" TODO ", x / 2, REGULAR_PAIR);
                    }
                    for (index, todo) in todos.iter().enumerate() {
                        ui.label_fix_width(
                            &format!("- [ ] {}", todo),
                            x / 2,
                            if index == todo_curr && panel == Status::Todo {
                                HIGHLIGHT_PAIR
                            } else {
                                REGULAR_PAIR
                            },
                        );
                    }
                }
                ui.end_layout();

                ui.begin_layout(LayoutKind::Vert);
                {
                    if panel == Status::Done {
                        ui.label_fix_width("[DONE]", x / 2, REGULAR_PAIR);
                    } else {
                        ui.label_fix_width(" DONE ", x / 2, REGULAR_PAIR);
                    }
                    for (index, done) in dones.iter().enumerate() {
                        ui.label_fix_width(
                            &format!("- [X] {}", done),
                            x / 2,
                            if index == done_curr && panel == Status::Done {
                                HIGHLIGHT_PAIR
                            } else {
                                REGULAR_PAIR
                            },
                        );
                    }
                }
                ui.end_layout();
            }
            ui.end();
            refresh();

            let key = getch();
            match key as u8 as char {
                'q' => quit = true,
                'K' => match panel {
                    Status::Todo => list_drag_up(&mut todos, &mut todo_curr),
                    Status::Done => list_drag_up(&mut dones, &mut done_curr),
                },
                'J' => match panel {
                    Status::Todo => list_drag_down(&mut todos, &mut todo_curr),
                    Status::Done => list_drag_down(&mut dones, &mut done_curr),
                },
                'k' => match panel {
                    Status::Todo => list_up(&mut todo_curr),
                    Status::Done => list_up(&mut done_curr),
                },
                'j' => match panel {
                    Status::Todo => list_down(&todos, &mut todo_curr),
                    Status::Done => list_down(&dones, &mut done_curr),
                },
                '\n' => match panel {
                    Status::Todo => list_transfer(&mut dones, &mut todos, &mut todo_curr),
                    Status::Done => list_transfer(&mut todos, &mut dones, &mut done_curr),
                },
                '\t' => panel = panel.toggle(),
                _ => {}
            }
        }
        save_state(&todos, &dones, &file_path);
        endwin();
    }
}
