use ratatui::prelude::*;

use crate::utils::{STYLE_CURSOR, STYLE_NONE};

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
        let mut char_index = 0;
        let mut char_buffer = [0; 4];
        let mut char_area = area;
        char_area.width = 1;
        char_area.height = 1;

        for (line_index, (line, newline)) in
            LineParser::new(&self.input, area.width as usize).enumerate()
        {
            char_area.x = area.x;
            char_area.y = area.y + line_index as u16;

            for c in line.chars() {
                let style = if self.cursor == char_index {
                    STYLE_CURSOR
                } else {
                    STYLE_NONE
                };
                Span::styled(&*c.encode_utf8(&mut char_buffer), style).render(char_area, buf);
                char_index += c.len_utf8();
                char_area.x += 1;
            }

            if self.cursor == char_index {
                Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
            }

            char_index += newline as usize;
        }
    }
}

struct LineParser<'a> {
    text: &'a str,
    line_width: usize,
    start: usize,
    chars: std::str::CharIndices<'a>,
    last_is_newline: bool,
}

impl<'a> LineParser<'a> {
    fn new(text: &'a str, line_width: usize) -> Self {
        Self {
            text,
            line_width,
            start: 0,
            chars: text.char_indices(),
            last_is_newline: false,
        }
    }
}

impl<'a> Iterator for LineParser<'a> {
    type Item = (&'a str, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.text.len() {
            if self.last_is_newline {
                self.last_is_newline = false;
                return Some(("", false));
            } else {
                return None;
            }
        }

        let mut width = 0; // todo: utf8
        loop {
            let Some((i, c)) = self.chars.next() else {
                let line = &self.text[self.start..];
                self.start = self.text.len();
                return Some((line, false));
            };

            match c {
                '\n' => {
                    let line = &self.text[self.start..i];
                    self.start = i + 1;
                    self.last_is_newline = self.start == self.text.len();
                    return Some((line, true));
                }
                _ => {
                    width += 1;

                    if width >= self.line_width {
                        let end = i + c.len_utf8();
                        let line = &self.text[self.start..end];
                        self.start = end;
                        return Some((line, false));
                    }
                }
            }
        }
    }
}
