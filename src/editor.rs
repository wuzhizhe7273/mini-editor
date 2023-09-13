use std::{
    env,
    time::{Duration, Instant},
};

use crossterm::{
    event::{KeyCode, KeyModifiers},
    style::Color,
    terminal::disable_raw_mode,
};

use crate::{document::Document, row::Row, terminal::Terminal};
const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: Color = Color::Rgb {
    r: 239,
    g: 239,
    b: 239,
};
const STATUS_FG_COLOR: Color = Color::Rgb {
    r: 63,
    g: 63,
    b: 63,
};
const QUIT_TIMES: u8 = 3;
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
    quit_times: u8,
}

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

// 状态信息
struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}
impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                self.die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                self.die(error);
            }
        }
    }
    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.terminal.cursor_hide()?;
        self.terminal.cursor_position(&Position::default())?;
        if self.should_quit {
            self.terminal.clear_screen()?;
            println!("Goodbye.\r");
        } else {
            self.draw_rows()?;
            // 状态栏绘制
            self.draw_status_bar()?;
            self.draw_message_bar();
            //光标移动
            self.terminal.cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            })?;
        }
        self.terminal.cursor_show()?;
        self.terminal.flush()
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }
        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = self.terminal.read_key()?;
        match pressed_key {
            (code, KeyModifiers::CONTROL) => match code {
                KeyCode::Char('x') => {
                    if self.quit_times > 0 && self.document.is_dirty() {
                        self.status_message = StatusMessage::from(format!(
                            "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                            self.quit_times
                        ));
                        self.quit_times -= 1;
                        return Ok(());
                    }
                    self.should_quit = true;
                }

                KeyCode::Char('s') => {
                    self.save();
                }
                _ => (),
            },
            (key, KeyModifiers::NONE) => match key {
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End => {
                    self.move_cursor(key);
                }
                // 删除
                KeyCode::Delete => {
                    self.document.delete(&self.cursor_position);
                }
                // 退格
                KeyCode::Backspace => {
                    if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                        self.move_cursor(KeyCode::Left);
                        self.document.delete(&self.cursor_position);
                    }
                }
                //换行
                KeyCode::Enter => {
                    self.document.insert(&self.cursor_position, '\n');
                    self.move_cursor(KeyCode::Right);
                }
                // 字符
                KeyCode::Char(c) => {
                    self.document.insert(&self.cursor_position, c);
                    self.move_cursor(KeyCode::Right);
                }
                _ => (),
            },
            _ => (),
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }
    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();

        loop {
            self.status_message = StatusMessage::from(format!("{prompt}{result}"));
            self.refresh_screen()?;
            if let (code, KeyModifiers::NONE) = self.terminal.read_key()? {
                match code {
                    KeyCode::Enter => {
                        break;
                    }
                    KeyCode::Backspace => {
                        if !result.is_empty() {
                            result.truncate(result.len() - 1);
                        }
                    }
                    KeyCode::Char(c) => result.push(c),
                    _ => (),
                }
            }
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
    fn move_cursor(&mut self, key: KeyCode) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    };
                }
            }
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height
                } else {
                    height
                }
            }
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }
        self.cursor_position = Position { x, y };
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    pub fn default() -> Self{
        // 获取文件名，初始化提示信息
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-S = save | Ctrl-X = quit");
        // 打开文档
        let mut document=Document::default();
        if args.len() > 1 {
            let file_name = &args[1];
            let doc= Document::open(file_name);
            match doc {
                Ok(d) => {
                    document=d;
                },
                Err(_) =>{
                    initial_status = format!("ERR:Could not Open file:{file_name}");
                },
            }
        }
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times:QUIT_TIMES
        }
    }

    fn draw_status_bar(&mut self)->Result<(),std::io::Error>{
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };
        // 获取文件名
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        // 拼接文件信息
        status = format!(
            "{} - {} lines {}",
            file_name,
            self.document.len(),
            modified_indicator
        );
        // 展示当前行数
        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        // 空白填充
        let len = status.len() + line_indicator.len();
        if width > status.len() {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{status}{line_indicator}");
        status.truncate(width);
        self.terminal.set_bg_color(STATUS_BG_COLOR)?;
        self.terminal.set_fg_color(STATUS_FG_COLOR)?;
        println!("{status}\r");
        self.terminal.reset_bg_color()?;
        self.terminal.reset_fg_color()?;
        Ok(())
    }
    fn draw_message_bar(&mut self) {
        let _=self.terminal.clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{text}");
        }
    }
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto editor -- version {VERSION}");
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);
        println!("{welcome_message}\r");
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{row}\r");
    }
    fn draw_rows(&mut self) -> Result<(), std::io::Error> {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            self.terminal.clear_current_line()?;
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                // 空行前导~
                println!("~ \r");
            }
        }
        Ok(())
    }
    fn die(&mut self, e:std::io::Error) {
        let _ = self.terminal.clear_screen();
        let _ = self.terminal.flush();
        let _ = disable_raw_mode();
        panic!("{}",e);
    }
}
