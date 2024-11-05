use ratatui::prelude::*;

use crate::utils::{STYLE_CURSOR, STYLE_NONE};

pub struct TextEditor {
    input: String,
    line_width: u16,
    cursor: usize,
    cursor_line: usize,
    cursor_line_chars: usize,
    line_starts: Vec<usize>,
    selection_start: Option<usize>,
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
            line_width: 0,
            cursor: 0,
            cursor_line: 0,
            cursor_line_chars: 0,
            line_starts: Vec::new(),
            selection_start: None,
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
        self.input.insert_str(self.cursor, s);
    }

    pub fn move_cursor(&mut self, cm: CursorMove, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }

        match cm {
            CursorMove::Forward => {
                if let Some(c) = self.input[self.cursor..].chars().next() {
                    self.cursor += c.len_utf8();
                }
            }
            CursorMove::Back => {
                if let Some(c) = self.input[..self.cursor].chars().rev().next() {
                    self.cursor -= c.len_utf8();
                }
            }
            CursorMove::Up => {
                if self.cursor_line == 0 {
                    self.cursor = 0;
                } else {
                    self.jump_to_line(self.cursor_line - 1);
                }
            }
            CursorMove::Down => {
                if self.cursor_line == self.line_starts.len() - 1 {
                    self.cursor = self.input.len();
                } else {
                    self.jump_to_line(self.cursor_line + 1);
                }
            }
            CursorMove::Start => self.cursor = 0,
            CursorMove::End => self.cursor = self.input.len(),
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }

    fn jump_to_line(&mut self, i: usize) {
        self.cursor_line = i;
        self.cursor = self.line_starts[self.cursor_line];

        let mut chars = self.input[self.cursor..].chars();
        let mut char_count = 0;
        let mut cursor_offset = 0;

        loop {
            let Some(c) = chars.next() else {
                break;
            };

            if char_count >= self.cursor_line_chars {
                break;
            }

            if c == '\n' {
                break;
            }

            char_count += 1;
            cursor_offset += c.len_utf8();
        }

        self.cursor += cursor_offset;
        self.cursor_line_chars = char_count;
    }
}

impl Widget for &mut TextEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.line_starts.clear();
        self.line_starts.push(0);
        self.line_width = area.width;

        let mut chars = self.input.char_indices();
        let mut char_buffer = [0; 4];
        let mut char_area = area;
        char_area.width = 1;
        char_area.height = 1;

        let mut line_width = 0;
        let mut line_index = 0;
        let selection_start = self.cursor.min(self.selection_start.unwrap_or(self.cursor));
        let selection_end = self.selection_start.unwrap_or(self.cursor).max(self.cursor);

        loop {
            let Some((i, c)) = chars.next() else {
                if selection_end == self.input.len() {
                    Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
                }
                break;
            };

            let is_cursor = i == self.cursor;
            let is_selected = i >= selection_start && i <= selection_end;
            let style = if is_cursor || is_selected {
                STYLE_CURSOR
            } else {
                STYLE_NONE
            };

            if is_cursor {
                self.cursor_line = line_index;
                self.cursor_line_chars = line_width;
            }

            let next_line = match c {
                '\n' => {
                    if is_cursor || is_selected {
                        Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
                    }
                    true
                }
                _ => {
                    Span::styled(&*c.encode_utf8(&mut char_buffer), style).render(char_area, buf);
                    char_area.x += 1;
                    line_width += 1;
                    line_width >= area.width as usize
                }
            };

            if next_line {
                line_width = 0;
                line_index += 1;
                char_area.y += 1;
                char_area.x = area.x;
                self.line_starts.push(i + c.len_utf8());
            }
        }
    }
}
