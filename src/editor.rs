use ratatui::prelude::*;

pub struct TextEditor {
    input: String,
    cursor: usize,
}

pub enum CursorMove {
    Forward,
    Back,
    Up,
    Down,
    Start,
    End,
}

impl TextEditor {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.input.len()
    }

    pub fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, index: usize) {
        self.cursor = usize::min(index, self.input.len());
    }

    pub fn cursor_add(&mut self, n: usize) {
        self.cursor = usize::min(self.cursor + n, self.input.len());
    }

    pub fn cursor_sub(&mut self, n: usize) {
        self.cursor = self.cursor.saturating_sub(n);
    }

    pub fn push_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
    }

    pub fn push_str(&mut self, s: &str) {
        self.input.push_str(s);
    }

    pub fn move_cursor(&mut self, cm: CursorMove) {
        match cm {
            CursorMove::Forward => {
                if let Some((_, c)) = self.input[self.cursor..].char_indices().next() {
                    self.cursor += c.len_utf8();
                }
            }
            CursorMove::Back => {
                if let Some((_, c)) = self.input[..self.cursor].char_indices().rev().next() {
                    self.cursor -= c.len_utf8();
                }
            }
            CursorMove::Up => {
                let mut n = 0;
                let mut chars = self.input[..self.cursor].chars().rev();
                while let Some(c) = chars.next() {
                    n += c.len_utf8();
                    if c == '\n' {
                        break;
                    }
                }
                self.cursor -= n;
            }
            CursorMove::Down => {
                let mut n = 0;
                let mut chars = self.input[self.cursor..].chars();
                while let Some(c) = chars.next() {
                    n += c.len_utf8();
                    if c == '\n' {
                        break;
                    }
                }
                self.cursor += n;
            }
            CursorMove::Start => self.cursor = 0,
            CursorMove::End => self.cursor = self.input.len(),
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }
}

impl Widget for &TextEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut line_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        let mut cursor_pos = (0, 0);
        let mut current_idx = 0;

        for (line_index, (line, line_split_char_len)) in
            LineParser::new(&self.input, area.width as usize).enumerate()
        {
            // Render line
            Line::raw(line).render(line_area, buf);
            line_area.y += 1;

            // Find cursor position
            let len = line.len() + line_split_char_len;
            let line_end_idx = current_idx + len;
            if current_idx <= self.cursor && line_end_idx >= self.cursor {
                cursor_pos.0 = (self.cursor - current_idx) as u16;
                cursor_pos.1 = line_index as u16;
            }
            current_idx = line_end_idx;
        }

        // Render cursor
        let cursor_cell = &mut buf[(area.x + cursor_pos.0, area.y + cursor_pos.1)];
        cursor_cell.set_bg(Color::Blue);
    }
}

struct LineParser<'a> {
    text: &'a str,
    line_width: usize,
    start: usize,
    chars: std::str::CharIndices<'a>,
}

impl<'a> LineParser<'a> {
    fn new(text: &'a str, line_width: usize) -> Self {
        Self {
            text,
            line_width,
            start: 0,
            chars: text.char_indices(),
        }
    }
}

impl<'a> Iterator for LineParser<'a> {
    type Item = (&'a str, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.text.len() {
            return None;
        }

        let mut width = 0; // todo: utf8
        loop {
            let Some((i, c)) = self.chars.next() else {
                let line = &self.text[self.start..];
                self.start = self.text.len();
                return Some((line, 0));
            };

            match c {
                '\n' => {
                    let line = &self.text[self.start..i];
                    self.start = i + 1;
                    return Some((line, 1));
                }
                _ => {
                    width += 1;
                    if width >= self.line_width {
                        let end = i + c.len_utf8();
                        let line = &self.text[self.start..end];
                        self.start = end;
                        return Some((line, 0));
                    }
                }
            }
        }
    }
}
