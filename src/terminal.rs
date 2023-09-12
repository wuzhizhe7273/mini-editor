use std::io::{Stdout, stdout, Write, self};

use crossterm::{terminal, QueueableCommand, cursor, event::{read, KeyCode, KeyModifiers, Event, KeyEvent, KeyEventKind}};

use crate::Position;

pub struct Size {
    pub width: u16,
    pub height: u16,
}
pub struct Terminal {
    size: Size,
    _stdout:Stdout
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let size = terminal::size()?;
        terminal::enable_raw_mode()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1,
            },
            _stdout:stdout()
        })
    }
    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn clear_screen(&mut self)->Result<(),io::Error>{
        // self._stdout.queue(terminal::Clear(terminal::ClearType::Purge))?;
        self._stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }
    pub fn clear_current_line(&mut self)->Result<(),io::Error>{
        self._stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        Ok(())
    }
    pub fn cursor_position(&mut self,position:&Position)->Result<(),io::Error>{
        let Position{x,  y}=position;
        let x=x.saturating_add(0);
        let y=y.saturating_add(0);
        let x=x as u16;
        let y =y as u16;
        self._stdout.queue(cursor::MoveTo(x,y))?;
        Ok(())

    }
    pub fn cursor_hide(&mut self)->Result<(),io::Error>{
        self._stdout.queue(cursor::Hide)?;
        Ok(())
    }
    pub fn cursor_show(&mut self)->Result<(),io::Error>{
        self._stdout.queue(cursor::Show)?;
        Ok(())
    }
    pub fn flush(&mut self)->Result<(),io::Error>{
        self._stdout.flush()?;
        Ok(())
    }
    pub fn read_key(&self)->Result<(KeyCode,KeyModifiers),std::io::Error>{
        loop {
            if let Some(Event::Key(KeyEvent { code, modifiers, kind:KeyEventKind::Press, state:_ }))=read().into_iter().next(){
                
                return Ok((code,modifiers));
            }
        }
    }
}