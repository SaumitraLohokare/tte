#![allow(dead_code)]
pub struct StatusLine {
    /// The x position of the top left corner
    pub x: u16,
    /// The y position of the top left corner
    pub y: u16,
    /// The width of the status line
    pub width: usize,
    /// The height of the status line
    pub height: usize,
    /// Name of current active file
    pub filename: String,
}

impl StatusLine {
    pub fn new(x: u16, y: u16, width: usize, height: usize, filename: &str) -> Self {
        Self {
            x,
            y,
            width,
            height,
            filename: filename.to_string()
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

    pub fn get_text(&self) -> String {
        let padding = 1;
        let content_width = self.filename.len();

        let mut line = String::with_capacity(self.width);
        line.push(' ');
        
        line.push_str(&self.filename);

        for _ in 0..(self.width - padding - content_width - padding) {
            line.push(' ');
        }

        line.push(' ');

        line
    }
}