#![allow(dead_code)]
use std::fs;

/// Representation of a line of text.
/// `start` and `end` are inclusive, i.e., `end` will usually point to `\n` unless it the line does not end
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
    previous_offset: Option<usize> // TODO: Remember to reset this when an update is made in the buffer
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
            previous_offset: None,
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
        let data = match fs::read(&filename) {
            Ok(bytes) => bytes
                .into_iter()
                .map(|b| b as char)
                .filter(|c| *c != '\r') // Convert CRLF to LF
                .collect(),
            Err(_) => vec![],
        };

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
            previous_offset: None,
        };
        buffer.recalculate_lines();

        buffer
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

        self.lines.push(Line {
            start: previous_begining,
            end: self.data.len(),
        });
    }

    /// Returns the cursor x, y position on Terminal
    /// Position can be negative, which usually means cursor is currently outside the displayable bounds
    pub fn cursor_xy(&self) -> (isize, isize) {
        let mut x = 0isize;
        let mut y = 0isize;

        for Line { start, end } in self.lines.iter() {
            if *start <= self.cursor_pos && *end >= self.cursor_pos {
                x = self.cursor_pos as isize - *start as isize - self.offset_x as isize;

                break;
            } else {
                y += 1;
            }
        }

        (x + self.x as isize, y - self.offset_y as isize + self.y as isize)
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

            if x_offset > line.len() {
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

            if x_offset > line.len() {
                self.previous_offset = Some(x_offset);
                x_offset = line.len() - 1;
            }

            self.cursor_pos = line.start + x_offset;
        }
    }

    pub fn scroll(&mut self) {
        let (x, y) = self.cursor_xy();
        let (w, h) = (self.width, self.height);

        if y < 0 {
            let dy = (-y) as usize;
            self.offset_y -= dy;    // NOTE: This could lead to overflow
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
}
