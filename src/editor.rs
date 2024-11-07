use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::utils::{STYLE_CURSOR, STYLE_NONE, STYLE_SELECTED};

pub struct TextEditor {
    input: String,
    line_width: u16,
    cursor_index: usize,
    cursor_column: usize,
    cursor_line_index: usize,
    line_start_indexes: Vec<usize>,
    selection_start: Option<usize>,
    scroll: usize,
    line_spans: Vec<Line<'static>>,
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
            line_spans: Vec::new(),
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
                }
            }
            CursorMove::Back => {
                if let Some(c) = self.input[..self.cursor_index].chars().rev().next() {
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
            CursorMove::Start => {
                self.cursor_index = 0;
            }
            CursorMove::End => {
                self.cursor_index = self.input.len();
            }
        }

        if let Some(selector) = self.selection_start {
            if selector == self.cursor_index {
                self.selection_start = None;
            }
        }
    }

    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.cursor_index = self.input.len();
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
        self.cursor_column = 0;
        self.cursor_line_index = 0;
        self.selection_start = None;
        self.line_width = 0;
        self.line_start_indexes.clear();
        self.line_spans.clear();
        self.scroll = 0;
    }

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) {
        let shift = key_modifiers.contains(KeyModifiers::SHIFT);
        let ctrl = key_modifiers.contains(KeyModifiers::CONTROL);

        match key_pressed {
            KeyCode::Right => {
                self.move_cursor(CursorMove::Forward, shift);
            }
            KeyCode::Left => {
                self.move_cursor(CursorMove::Back, shift);
            }
            KeyCode::Up => {
                self.move_cursor(CursorMove::Up, shift);
            }
            KeyCode::Down => {
                self.move_cursor(CursorMove::Down, shift);
            }
            KeyCode::Home => {
                self.move_cursor(CursorMove::Start, shift);
            }
            KeyCode::End => {
                self.move_cursor(CursorMove::End, shift);
            }
            KeyCode::Enter => {
                self.push_char('\n');
            }
            KeyCode::Backspace => {
                self.delete_back();
            }
            KeyCode::Delete => {
                self.delete_forward();
            }
            KeyCode::Char(c) => match c {
                // TODO: whitespace/tabs
                'a' => {
                    if ctrl {
                        self.select_all();
                    } else {
                        self.push_char(c);
                    }
                }
                _ => {
                    self.push_char(c);
                }
            },
            _ => {}
        }
    }

    fn delete_selection(&mut self, selection_start: usize) {
        if selection_start < self.cursor_index {
            self.input
                .replace_range(selection_start..self.cursor_index, "");
            self.cursor_index = selection_start;
        } else if self.cursor_index < selection_start {
            if let Some(c) = self.input[self.cursor_index..].chars().next() {
                let start = self.cursor_index + c.len_utf8();
                let end = self.input[selection_start..]
                    .chars()
                    .next()
                    .map(|c| selection_start + c.len_utf8())
                    .unwrap_or(self.input.len());
                self.input.replace_range(start..end, "");
            }
        }
    }

    fn jump_to_line(&mut self, i: usize) {
        self.cursor_line_index = i;
        self.cursor_index = self.line_start_indexes[i];

        let mut chars = self.input[self.cursor_index..].chars();
        let mut column = 0;
        let mut offset = 0;

        loop {
            if column >= self.cursor_column {
                break;
            }

            let Some(c) = chars.next() else {
                break;
            };

            // TODO: whitespace/tabs

            if c == '\n' {
                break;
            }

            column += unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            offset += c.len_utf8();
        }

        self.cursor_column = column;
        self.cursor_index += offset;
    }
}

impl Widget for &mut TextEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.line_start_indexes.clear();
        self.cursor_column = 0;
        self.cursor_line_index = 0;
        self.line_width = area.width;

        let mut line_width = 0;
        let mut line_index = 0;
        let input_len = self.input.len();
        let selection_start = self
            .cursor_index
            .min(self.selection_start.unwrap_or(self.cursor_index));
        let selection_end = self
            .selection_start
            .unwrap_or(self.cursor_index)
            .max(self.cursor_index);

        self.line_start_indexes.push(0);
        self.line_spans.push(Line::default());

        let mut chars = self.input.char_indices();

        loop {
            let Some((i, c)) = chars.next() else {
                if self.cursor_index == input_len {
                    self.cursor_line_index = line_index;
                    self.cursor_column = line_width;
                    self.line_spans[line_index].push_span(Span::styled(" ", STYLE_CURSOR));
                }
                break;
            };

            let is_cursor = i == self.cursor_index;
            let is_selected = i >= selection_start && i <= selection_end;

            let style = if is_cursor {
                self.cursor_line_index = line_index;
                self.cursor_column = line_width;
                STYLE_CURSOR
            } else if is_selected {
                STYLE_SELECTED
            } else {
                STYLE_NONE
            };

            let next_line = match c {
                '\n' => {
                    if is_cursor || (is_selected && i < input_len) {
                        let span = Span::styled(" ", style);
                        self.line_spans[line_index].push_span(span);
                    }
                    true
                }
                // TODO: whitespace/tabs
                _ => {
                    let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
                    let span = Span::styled(c.to_string(), style);
                    self.line_spans[line_index].push_span(span);
                    line_width += char_width;
                    line_width >= area.width as usize
                }
            };

            if next_line {
                line_width = 0;
                line_index += 1;
                self.line_spans.push(Line::default());
                self.line_start_indexes.push(i + c.len_utf8());
            }
        }

        let height = area.height as usize;
        if self.cursor_line_index > self.scroll {
            let height_diff = self.cursor_line_index - self.scroll;
            let height = height.saturating_sub(1);
            if height_diff > height {
                self.scroll += height_diff - height;
            }
        } else if self.scroll > self.cursor_line_index {
            let height_diff = self.scroll - self.cursor_line_index;
            self.scroll -= height_diff;
        }

        let mut line_area = area;
        line_area.height = 1;

        self.line_spans
            .drain(..)
            .skip(self.scroll)
            .take(height)
            .for_each(|line| {
                line.render(line_area, buf);
                line_area.y += 1;
            });
    }
}
