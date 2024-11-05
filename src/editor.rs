use ratatui::prelude::*;

use crate::utils::{STYLE_CURSOR, STYLE_NONE};

pub struct TextEditor {
    input: String,
    line_width: u16,
    cursor_index: usize,
    cursor_column: usize,
    cursor_line_index: usize,
    line_start_indexes: Vec<usize>,
    selection_start: Option<usize>,
    scroll: usize,
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
            cursor_column: 0,
            cursor_line_index: 0,
            line_start_indexes: Vec::new(),
            selection_start: None,
            scroll: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn push_char(&mut self, c: char) {
        if let Some(start) = self.selection_start.take() {
            self.delete_selection(start);
        }
        self.input.insert(self.cursor_index, c);
        self.cursor_index += c.len_utf8();
    }

    pub fn push_str(&mut self, s: &str) {
        if let Some(start) = self.selection_start.take() {
            self.delete_selection(start);
        }
        self.input.insert_str(self.cursor_index, s);
        self.cursor_index += s.len();
    }

    // TODO: maintain metadata between cursor moves?
    pub fn move_cursor(&mut self, cm: CursorMove, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_index);
            }
        } else {
            self.selection_start = None;
        }

        match cm {
            CursorMove::Forward => {
                if let Some(c) = self.input[self.cursor_index..].chars().next() {
                    self.cursor_index += c.len_utf8();
                    // self.cursor_column += 1;

                    // if let Some(next_line_index) = self
                    //     .line_start_indexes
                    //     .get(self.cursor_line_index + 1)
                    //     .copied()
                    // {
                    //     if self.cursor_index == next_line_index {
                    //         self.cursor_line_index += 1;
                    //         self.cursor_column = 0;
                    //     }
                    // }
                }
            }
            CursorMove::Back => {
                if let Some(c) = self.input[..self.cursor_index].chars().rev().next() {
                    self.cursor_index -= c.len_utf8();

                    // if self.cursor_column == 0 {
                    //     self.cursor_line_index -= 1;
                    //     self.cursor_column = self.input
                    //         [self.line_start_indexes[self.cursor_line_index]..self.cursor_index]
                    //         .chars()
                    //         .count();
                    // }
                }
            }
            CursorMove::Up => {
                if self.cursor_line_index == 0 {
                    self.cursor_index = 0;
                    // self.cursor_column = 0;
                } else {
                    self.jump_to_line(self.cursor_line_index - 1);
                }
            }
            CursorMove::Down => {
                if self.cursor_line_index == self.line_start_indexes.len() - 1 {
                    self.cursor_index = self.input.len();
                    // self.cursor_column = self.input
                    //     [self.line_start_indexes[self.cursor_line_index]..self.cursor_index]
                    //     .chars()
                    //     .count();
                } else {
                    self.jump_to_line(self.cursor_line_index + 1);
                }
            }
            CursorMove::Start => {
                self.cursor_index = 0;
                // self.cursor_column = 0;
                // self.cursor_line_index = 0;
            }
            CursorMove::End => {
                self.cursor_index = self.input.len();
                // self.cursor_column = self.input
                //     [self.line_start_indexes[self.cursor_line_index]..self.cursor_index]
                //     .chars()
                //     .count();
            }
        }
    }

    pub fn delete_back(&mut self) {
        match self.selection_start.take() {
            Some(start) => self.delete_selection(start),
            None => {
                if let Some(c) = self.input[..self.cursor_index].chars().rev().next() {
                    self.cursor_index -= c.len_utf8();
                    self.input.remove(self.cursor_index);
                }
            }
        }
    }

    pub fn delete_forward(&mut self) {
        match self.selection_start.take() {
            Some(start) => self.delete_selection(start),
            None => {
                if self.input[self.cursor_index..].chars().next().is_some() {
                    self.input.remove(self.cursor_index);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_index = 0;
    }

    fn delete_selection(&mut self, selection_start: usize) {
        let start = usize::min(self.cursor_index, selection_start);
        let end = {
            let end = usize::max(self.cursor_index, selection_start);
            match self.input[end..].chars().next() {
                Some(c) => end + c.len_utf8(),
                None => end,
            }
        };
        self.input.replace_range(start..end, "");
        self.cursor_index = start;
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

            if char_count >= self.cursor_column {
                break;
            }

            if c == '\n' {
                break;
            }

            char_count += 1;
            cursor_offset += c.len_utf8();
        }

        self.cursor_index += cursor_offset;
        self.cursor_column = char_count;
    }
}

impl Widget for &mut TextEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.cursor_column = 0;
        self.cursor_line_index = 0;
        self.line_width = area.width;
        self.line_start_indexes.clear();

        let mut chars = self.input.char_indices();
        let mut char_buffer = [0; 4];
        let mut char_area = area;
        char_area.width = 1;
        char_area.height = 1;

        let mut line_width = 0;
        let mut line_index = 0;
        let selection_start = self
            .cursor_index
            .min(self.selection_start.unwrap_or(self.cursor_index));
        let selection_end = self
            .selection_start
            .unwrap_or(self.cursor_index)
            .max(self.cursor_index);

        self.line_start_indexes.push(0);
        let mut lines = vec![Line::default()];

        loop {
            let Some((i, c)) = chars.next() else {
                if self.cursor_index == self.input.len() {
                    self.cursor_line_index = line_index;
                    self.cursor_column = line_width;
                }

                if selection_end == self.input.len() {
                    let span = Span::styled(" ", STYLE_CURSOR);
                    lines[line_index].push_span(span);
                }

                break;
            };

            let is_cursor = i == self.cursor_index;
            let is_selected = i >= selection_start && i <= selection_end;
            let style = if is_cursor || is_selected {
                STYLE_CURSOR
            } else {
                STYLE_NONE
            };

            if is_cursor {
                self.cursor_line_index = line_index;
                self.cursor_column = line_width;
            }

            let next_line = match c {
                '\n' => {
                    if is_cursor || is_selected {
                        let span = Span::styled(" ", STYLE_CURSOR);
                        lines[line_index].push_span(span);
                    }
                    true
                }
                // TODO: what about invisible/other whitespace chars?
                _ => {
                    let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
                    char_area.width = char_width as u16;

                    let span = Span::styled(c.to_string(), style);
                    lines[line_index].push_span(span);

                    char_area.x += char_width as u16;
                    line_width += char_width;
                    line_width >= area.width as usize
                }
            };

            if next_line {
                line_width = 0;
                line_index += 1;
                char_area.y += 1;
                char_area.x = area.x;
                self.line_start_indexes.push(i + c.len_utf8());
                lines.push(Line::default());
            }
        }

        let height = area.height as usize;
        let skip_count = if self.cursor_line_index < height {
            0
        } else {
            self.cursor_line_index - height + 1
        };
        self.scroll = skip_count;

        let mut line_area = area;
        line_area.height = 1;

        lines
            .into_iter()
            .skip(skip_count)
            .take(height)
            .for_each(|line| {
                line.render(line_area, buf);
                line_area.y += 1;
            });
    }
}
