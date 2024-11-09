#![allow(dead_code)]
use std::{
    io::{self, Write},
    process::exit,
};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    execute, queue,
    style::{Print, ResetColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use crate::buffer::{Buffer, Line};

pub struct Display<W: Write> {
    width: u16,
    height: u16,
    out: W,
}

impl<W: Write> Display<W> {
    pub fn new(out: W) -> io::Result<Self> {
        let size = terminal::size()?;

        enable_raw_mode()?;

        let mut display = Self {
            width: size.0,
            height: size.1,
            out,
        };

        execute!(display.out, EnterAlternateScreen, DisableLineWrap)?;

        Ok(display)
    }

    pub fn resize(&mut self, w: u16, h: u16) -> io::Result<()> {
        self.width = w;
        self.height = h;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }

    pub fn begin_draw(&mut self) -> io::Result<()> {
        // We do not clear here because I'm not sure about our implementation of Display yet
        queue!(self.out, MoveTo(0, 0), ResetColor)
    }

    pub fn end_draw(&mut self) -> io::Result<()> {
        self.flush()
    }

    pub fn clear_all(&mut self) -> io::Result<()> {
        queue!(self.out, Clear(terminal::ClearType::All))
    }

    pub fn set_cursor_style(&mut self, style: SetCursorStyle) -> io::Result<()> {
        queue!(self.out, style)
    }

    pub fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        queue!(self.out, MoveTo(x, y))
    }

    pub fn print(&mut self, string: String) -> io::Result<()> {
        queue!(self.out, Print(string))
    }

    pub fn draw_buffer(&mut self, buffer: &Buffer) -> io::Result<()> {
        let mut display_buffer = String::with_capacity(self.width as usize);
        let mut row_idx = 0;

        queue!(self.out, Hide)?;

        for Line { start, end } in buffer
            .lines
            .iter()
            .skip(buffer.offset_y)
            .take(self.height as usize)
        {
            if let Some(data) = buffer.data.get(*start..=*end) {
                display_buffer.clear();
                for ch in data.iter().skip(buffer.offset_x).take(self.width as usize) {
                    if *ch != '\n' {
                        display_buffer.push(*ch);
                    }
                }
                queue!(self.out, MoveTo(0, row_idx), Print(&display_buffer))?;
                row_idx += 1;
            }
        }

        let (cursor_x, cursor_y) = buffer.cursor_xy();

        if cursor_x >= 0
            && cursor_x < self.width as isize
            && cursor_y >= 0
            && cursor_y < self.height as isize
        {
            queue!(self.out, MoveTo(cursor_x as u16, cursor_y as u16), Show,)?;
        }

        Ok(())
    }
}

impl<W: Write> Drop for Display<W> {
    fn drop(&mut self) {
        if let Err(e) = disable_raw_mode() {
            eprintln!("ERROR : Failed to disable terminal raw mode : {e}");
            exit(1);
        }

        if let Err(e) = execute!(self.out, ResetColor, LeaveAlternateScreen, EnableLineWrap) {
            eprintln!("ERROR : Failed to leave alternate screen : {e}");
            exit(1);
        }
    }
}
