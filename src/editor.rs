use ratatui::prelude::*;

use crate::utils::{STYLE_CURSOR, STYLE_NONE};

pub struct TextEditor {
    input: String,
    line_width: u16,
    cursor_index: usize,
    cursor_line_index: usize,
    cursor_line_chars: usize,
    line_start_indexes: Vec<usize>,
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
            cursor_index: 0,
            cursor_line_index: 0,
            cursor_line_chars: 0,
            line_start_indexes: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.input.len()
    }

    pub fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor_index
    }

    pub fn set_cursor(&mut self, index: usize) {
        self.cursor_index = usize::min(index, self.input.len());
    }

    pub fn cursor_add(&mut self, n: usize) {
        self.cursor_index = usize::min(self.cursor_index + n, self.input.len());
    }

    pub fn cursor_sub(&mut self, n: usize) {
        self.cursor_index = self.cursor_index.saturating_sub(n);
    }

    pub fn push_char(&mut self, c: char) {
        self.input.insert(self.cursor_index, c);
    }

    pub fn push_str(&mut self, s: &str) {
        self.input.insert_str(self.cursor_index, s);
    }

    pub fn move_cursor(&mut self, cm: CursorMove) {
        match cm {
            CursorMove::Forward => {
                if let Some((_, c)) = self.input[self.cursor_index..].char_indices().next() {
                    self.cursor_index += c.len_utf8();
                }
            }
            CursorMove::Back => {
                if let Some((_, c)) = self.input[..self.cursor_index].char_indices().rev().next() {
                    self.cursor_index -= c.len_utf8();
                }
            }
            CursorMove::Up => {
                if self.cursor_line_index == 0 {
                    self.cursor_index = 0;
                } else {
                    self.jump_to_line(self.cursor_line_index - 1);
                }
            }
            CursorMove::Down => {
                if self.cursor_line_index == self.line_start_indexes.len() - 1 {
                    self.cursor_index = self.input.len();
                } else {
                    self.jump_to_line(self.cursor_line_index + 1);
                }
            }
            CursorMove::Start => self.cursor_index = 0,
            CursorMove::End => self.cursor_index = self.input.len(),
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_index = 0;
    }

    fn jump_to_line(&mut self, i: usize) {
        self.cursor_line_index = i;
        self.cursor_index = self.line_start_indexes[self.cursor_line_index];

        let mut chars = self.input[self.cursor_index..].chars();
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

        self.cursor_index += cursor_offset;
        self.cursor_line_chars = char_count;
    }
}

impl Widget for &mut TextEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.line_start_indexes.clear();
        self.line_width = area.width;

        let mut chars = self.input.char_indices();
        let mut line_width = 0;
        let mut cursor_line = 0;
        let mut cursor_line_chars = 0;
        let mut char_buffer = [0; 4];
        let mut char_area = area;
        char_area.width = 1;
        char_area.height = 1;

        self.line_start_indexes.push(0);

        loop {
            let Some((i, c)) = chars.next() else {
                if self.cursor_index == self.input.len() {
                    self.cursor_line_index = cursor_line;
                    Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
                }
                break;
            };

            let is_cursor = self.cursor_index == i;
            let style = if is_cursor {
                self.cursor_line_index = cursor_line;
                self.cursor_line_chars = cursor_line_chars;
                STYLE_CURSOR
            } else {
                cursor_line_chars += 1;
                STYLE_NONE
            };

            match c {
                '\n' => {
                    if is_cursor {
                        Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
                    }

                    line_width = 0;
                    cursor_line_chars = 0;
                    cursor_line += 1;
                    char_area.y += 1;
                    char_area.x = area.x;
                    self.line_start_indexes.push(i + c.len_utf8());
                }
                _ => {
                    Span::styled(&*c.encode_utf8(&mut char_buffer), style).render(char_area, buf);

                    char_area.x += 1;
                    line_width += 1;

                    if line_width >= area.width {
                        line_width = 0;
                        cursor_line_chars = 0;
                        cursor_line += 1;
                        char_area.y += 1;
                        char_area.x = area.x;
                        self.line_start_indexes.push(i + c.len_utf8());
                    }
                }
            }
        }
    }
}
