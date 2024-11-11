#![allow(dead_code)]
use std::{ffi::OsStr, fs, path::{Path, PathBuf}};

use crossterm::style::Color;

/*
    Color theme default
    :root {
        --bg-color: rgb(30, 30, 30);        /* Background (dark gray) */
        --fg-color: rgb(210, 210, 210);     /* Foreground text (light gray) */
        --keyword-color: rgb(255, 210, 85); /* Warm Yellow Highlight */
        --comment-color: rgb(120, 150, 120);/* Comments (dim green) */
        --string-color: rgb(190, 230, 120); /* Strings (soft green) */
        --number-color: rgb(255, 215, 85);  /* Numbers (soft yellow) */
    }
*/

/// Representation of a line of text.
/// `start` and `end` are inclusive, i.e., `end` will usually point to `\n` unless it the line does not end
#[derive(Debug)]
pub struct Line {
    pub start: usize,
    pub end: usize,
}

impl Line {
    pub fn len(&self) -> usize {
        self.end - self.start + 1
    }
}

pub struct Buffer {
    /// The actual data in the buffer
    pub data: Vec<char>,
    /// Indexes into the lines in the buffer
    pub lines: Vec<Line>,
    /// The x position of the top left corner
    pub x: u16,
    /// The y position of the top left corner
    pub y: u16,
    /// The width of the buffer
    pub width: usize,
    /// The height of the buffer
    pub height: usize,
    /// Line offset while printing to `Display`
    pub offset_y: usize,
    /// Character offset while printing to `Display`
    pub offset_x: usize,
    /// Cursor position in the data
    pub cursor_pos: usize,
    /// Remember the previous x offset of the cursor in line
    previous_offset: Option<usize>, // TODO: Remember to reset this when an update is made in the buffer
    pub file_path: Option<PathBuf>,
    /// Background color
    pub bg_color: Color,
    /// Foreground color
    pub fg_color: Color,
    // TODO: Add Comments colors, highlighting colors, literal values colors (strings, numbers)
}

impl Buffer {
    /// Returns a new empty `Buffer`
    pub fn new(x: u16, y: u16, width: usize, height: usize) -> Self {
        let mut buffer = Self {
            data: vec![],
            lines: vec![],
            x,
            y,
            width,
            height,
            offset_y: 0,
            offset_x: 0,
            cursor_pos: 0,
            file_path: None,
            previous_offset: None,
            bg_color: Color::Rgb {
                r: 30,
                g: 30,
                b: 30,
            },
            fg_color: Color::Rgb {
                r: 210,
                g: 210,
                b: 210,
            },
        };

        buffer.recalculate_lines();

        buffer
    }

    /// Returns a new filled `Buffer` with the contents of file `filename`.
    /// If file does not exist, or opening file failed, returns an empty `Buffer`.
    ///
    /// **NOTE:**
    /// For now we replace CRLF to LF
    pub fn from_file(filename: &str, x: u16, y: u16, width: usize, height: usize) -> Self {
        let path = Path::new(filename);
        let (data, file_path) = if path.is_file() {
            // If the path is a valid file, read its content
            match fs::read(&path) {
                Ok(bytes) => (
                    bytes
                        .into_iter()
                        .map(|b| b as char)
                        .filter(|c| *c != '\r') // Convert CRLF to LF
                        .collect(),
                    Some(path.to_path_buf()),
                ),
                Err(_) => (vec![], Some(path.to_path_buf())),
            }
        } else if path.is_dir() {
            // If no filename or it's a directory, set empty data and None for file_path
            (vec![], None)
        } else {
            // If the path is invalid for some reason (file, but not readable)
            (vec![], Some(path.to_path_buf()))
        };

        // Initialize the buffer
        let mut buffer = Self {
            data,
            lines: vec![],
            x,
            y,
            width,
            height,
            offset_y: 0,
            offset_x: 0,
            cursor_pos: 0,
            file_path,
            previous_offset: None,
            bg_color: Color::Rgb {
                r: 30,
                g: 30,
                b: 30,
            },
            fg_color: Color::Rgb {
                r: 210,
                g: 210,
                b: 210,
            },
        };
        buffer.recalculate_lines();

        buffer
    }

    pub fn file_name(&self) -> String {
        match &self.file_path {
            Some(path) => path
                .file_name()
                .unwrap_or(OsStr::new("NO NAME"))
                .to_str()
                .unwrap_or("NO NAME")
                .to_string(),
            None => "NO NAME".to_string(),
        }
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.width = w;
        self.height = h;
    }

    pub fn recalculate_lines(&mut self) {
        let mut previous_begining = 0;
        self.lines.clear();

        for (i, ch) in self.data.iter().enumerate() {
            if *ch == '\n' {
                self.lines.push(Line {
                    start: previous_begining,
                    end: i,
                });
                previous_begining = i + 1;
            }
        }

        let end = if self.data.len() < 1 {
            0
        } else {
            self.data.len() - 1
        };

        self.lines.push(Line {
            start: previous_begining,
            end,
        });
    }

    /// Returns the cursor x, y position on Terminal
    /// Position can be negative, which usually means cursor is currently outside the displayable bounds
    #[allow(unused_assignments)]
    pub fn cursor_xy(&self) -> (isize, isize) {
        let mut x = 0isize;
        let mut y = 0isize;

        for Line { start, end } in self.lines.iter() {
            if *start <= self.cursor_pos && *end >= self.cursor_pos {
                x = self.cursor_pos as isize - *start as isize - self.offset_x as isize;

                return (
                    x + self.x as isize,
                    y - self.offset_y as isize + self.y as isize,
                );
            } else {
                y += 1;
            }
        }

        let last_line = self
            .lines
            .last()
            .expect("Buffer should always have atleast one line");

        (
            last_line.end as isize - last_line.start as isize + 1 + self.x as isize,
            y - 1 - self.offset_y as isize + self.y as isize,
        )
    }

    pub fn current_line(&self) -> usize {
        let mut current_line = 0;

        for Line { start, end } in self.lines.iter() {
            if *start <= self.cursor_pos && *end >= self.cursor_pos {
                return current_line;
            } else {
                current_line += 1;
            }
        }

        unreachable!("Should never end up here.");
    }

    pub fn move_cursor_right(&mut self, dx: usize) {
        if self.cursor_pos + dx <= self.data.len() {
            self.cursor_pos += dx;
        }

        self.previous_offset = None;
    }

    pub fn move_cursor_left(&mut self, dx: usize) {
        if self.cursor_pos >= dx {
            self.cursor_pos -= dx;
        }

        self.previous_offset = None;
    }

    pub fn move_cursor_up(&mut self, dy: usize) {
        let mut current_line = self.current_line();

        if current_line >= dy {
            let line = &self.lines[current_line];
            let mut x_offset = match self.previous_offset {
                Some(offset) => offset,
                None => self.cursor_pos - line.start,
            };

            current_line -= dy;

            let line = &self.lines[current_line];

            if x_offset >= line.len() {
                self.previous_offset = Some(x_offset);
                x_offset = line.len() - 1;
            }

            self.cursor_pos = line.start + x_offset;
        }
    }

    pub fn move_cursor_down(&mut self, dy: usize) {
        let mut current_line = self.current_line();

        if current_line + dy < self.lines.len() {
            let line = &self.lines[current_line];
            let mut x_offset = match self.previous_offset {
                Some(offset) => offset,
                None => self.cursor_pos - line.start,
            };

            current_line += dy;

            let line = &self.lines[current_line];

            if x_offset >= line.len() {
                self.previous_offset = Some(x_offset);
                x_offset = line.len() - 1;
            }

            self.cursor_pos = line.start + x_offset;
        }
    }

    pub fn scroll(&mut self) {
        let (x, y) = self.cursor_xy();
        let (w, h) = (self.width, self.height);

        let y = y - self.y as isize;
        let x = x - self.x as isize;

        if y < 0 {
            let dy = (-y) as usize;
            assert!(self.offset_y >= dy);
            self.offset_y -= dy; // NOTE: This could lead to overflow
        } else if y >= h as isize {
            let dy = y - h as isize + 1;
            self.offset_y += dy as usize;
        }

        if x < 0 {
            let dx = (-x) as usize;
            self.offset_x -= dx;
        } else if x >= w as isize {
            let dx = x - w as isize + 1;
            self.offset_x += dx as usize;
        }
    }

    pub fn insert_ch(&mut self, ch: char) {
        self.data.insert(self.cursor_pos, ch);
        self.cursor_pos += 1;
    }

    /// Same as backspace key pressed
    pub fn backspace(&mut self) {
        self.cursor_pos -= 1;
        self.data.remove(self.cursor_pos);
    }

    /// Same as delete key pressed
    pub fn delete(&mut self) {
        self.data.remove(self.cursor_pos);
    }

    /// Save the file if the buffer has a valid file_path
    pub fn save(&self) {
        if let Some(path) = &self.file_path {
            // save the data into the path
            let content: String = self.data.iter().collect();
            fs::write(path, content).expect("Failed to save file.");
        }
    }
}
