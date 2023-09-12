use std::env;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::{terminal::Terminal, document::Document, row::Row};
const VERSION: &str = env!("CARGO_PKG_VERSION");
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position:Position,
    document:Document,
    offset:Position,
}

#[derive(Default)]
pub struct Position {
    pub x:usize,
    pub y:usize,
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
        self.terminal.clear_screen()?;
        self.terminal.cursor_position(&Position { x: 0, y: 0 })?;
        if self.should_quit {
            self.terminal.clear_screen()?;
            println!("Goodbye.\r")
        } else {
            self.draw_rows()?;
            self.terminal.cursor_position(&self.cursor_position)?;
        }
        self.terminal.cursor_show()?;
        self.terminal.flush()
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = self.terminal.read_key()?;
        match pressed_key {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (key,KeyModifiers::NONE)=>{
                match key {
                    KeyCode::Up|KeyCode::Down|KeyCode::Left|KeyCode::Right|KeyCode::PageUp|KeyCode::PageDown|KeyCode::Home|KeyCode::End=>{
                        self.move_cursor(key);
                    }
                    _=>()
                }
            }
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn move_cursor(&mut self,key:KeyCode){
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            KeyCode::Up =>y=y.saturating_sub(1),
            KeyCode::Down =>{
                if y < height {
                    y = y.saturating_add(1);
                }
            },
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
            },
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            },
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _=>()
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }
        self.cursor_position=Position{x,y};
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
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
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let file_name = &args[1];
            Document::open(&file_name).unwrap_or_default()
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset:Position::default()
        }
    }


    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row);
    }
    fn draw_rows(&mut self) -> Result<(), std::io::Error> {
        print!("\r");
        let height=self.terminal.size().height;
        for terminal_row in 0..height- 1{
            self.terminal.clear_current_line()?;
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y)  {
                self.draw_row(row);
            }else if self.document.is_empty()  && terminal_row == height / 3 {
                self.draw_welcome_message()
            }else {
                println!("~ \r")
            }
        }
        Ok(())
    }
    fn die(&mut self, e: std::io::Error) {
        let _ = self.terminal.clear_screen();
        let _ = self.terminal.flush();
        panic!("{}", e);
    }
}