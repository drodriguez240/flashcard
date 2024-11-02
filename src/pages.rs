use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use tui_textarea::TextArea;

use crate::{app::Action, database::*, markup::*, utils::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    AddCard,
    EditCard(CardId),
}

impl Route {
    pub const fn title(self) -> &'static str {
        match self {
            Route::Review => "Review",
            Route::AddCard => "Add Card",
            Route::EditCard(_) => "Edit Card",
        }
    }
}

pub struct Pages {
    pub review: Review,
    pub add_card: AddCard,
    pub edit_card: EditCard,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            review: Review::new(),
            add_card: AddCard::new(),
            edit_card: EditCard::new(),
        }
    }
}

pub struct Review {
    due: Vec<CardId>,
    total: usize,
    progress: usize,
    state: ReviewState,
    text: String,
    scroll: usize,
}

enum ReviewState {
    None,
    Review(CardId),
    Done,
}

impl Review {
    pub const fn new() -> Self {
        Self {
            due: Vec::new(),
            total: 0,
            progress: 0,
            state: ReviewState::None,
            text: String::new(),
            scroll: 0,
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.due.extend(db.iter().rev().map(|(id, _)| id));
        self.total = self.due.len();
        if let Some(id) = self.due.pop() {
            let card = db.get(&id).unwrap();
            self.state = ReviewState::Review(id);
            self.text.push_str(card.0.as_str());
        }
    }

    pub fn on_render(&mut self, area: Rect, buf: &mut Buffer) {
        match self.state {
            ReviewState::None => {
                Line::raw("no cards to review...")
                    .alignment(Alignment::Center)
                    .render(area, buf);
            }
            ReviewState::Review(_) => {
                Markup::new(&self.text).render(area, buf, &mut self.scroll);
            }
            ReviewState::Done => {
                Line::raw("done")
                    .alignment(Alignment::Center)
                    .render(area, buf);
            }
        }
    }

    pub fn on_input(&mut self, key: KeyEvent, db: &mut Database) -> Action {
        match self.state {
            ReviewState::Review(id) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => return Action::Quit,
                        KeyCode::Tab => return Action::Route(Route::AddCard),
                        KeyCode::Char('e') => return Action::Route(Route::EditCard(id)),
                        KeyCode::Delete => {
                            db.remove(&id);
                            if let Some(next_id) = self.due.pop() {
                                self.state = ReviewState::Review(next_id);
                                let card = db.get(&next_id).unwrap();
                                self.text.clear();
                                self.text.push_str(card.0.as_str());
                            } else {
                                self.state = ReviewState::Done;
                            }
                            return Action::Render;
                        }
                        KeyCode::Char(' ') => {
                            // todo: show answer
                        }
                        KeyCode::Up => {
                            // todo: successful recall
                            // fixme: activates when scrolling with touchpad?
                            self.scroll = self.scroll.saturating_sub(1);
                            return Action::Render;
                        }
                        KeyCode::Down => {
                            // todo: unsuccessful recall
                            // fixme: activates when scrolling with touchpad?
                            self.scroll = self.scroll.saturating_add(1);
                            return Action::Render;
                        }
                        KeyCode::Right => {
                            if let Some(next_id) = self.due.pop() {
                                self.due.insert(0, id);
                                self.state = ReviewState::Review(next_id);
                                let card = db.get(&next_id).unwrap();
                                self.text.clear();
                                self.text.push_str(card.0.as_str());
                                return Action::Render;
                            }
                        }
                        _ => {}
                    }
                }
            }
            ReviewState::None | ReviewState::Done => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => return Action::Quit,
                        KeyCode::Tab => return Action::Route(Route::AddCard),
                        _ => {}
                    }
                }
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.due.clear();
        self.total = 0;
        self.progress = 0;
        self.state = ReviewState::None;
        self.text.clear();
        self.scroll = 0;
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        match self.state {
            ReviewState::Review(_) => {
                if self.due.is_empty() {
                    &[
                        SHORTCUT_SCROLL,
                        SHORTCUT_EDIT,
                        SHORTCUT_DELETE,
                        SHORTCUT_MENU,
                        SHORTCUT_QUIT,
                    ]
                } else {
                    &[
                        SHORTCUT_SCROLL,
                        SHORTCUT_EDIT,
                        SHORTCUT_DELETE,
                        SHORTCUT_SKIP,
                        SHORTCUT_MENU,
                        SHORTCUT_QUIT,
                    ]
                }
            }
            ReviewState::None | ReviewState::Done => &[SHORTCUT_MENU, SHORTCUT_QUIT],
        }
    }
}

pub struct AddCard {
    editor: TextArea<'static>,
}

impl AddCard {
    pub fn new() -> Self {
        Self {
            editor: TextArea::default(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        //todo
    }

    pub fn on_render(&self, area: Rect, buf: &mut Buffer) {
        self.editor.render(area, buf);
    }

    pub fn on_input(&mut self, key: KeyEvent, db: &mut Database) -> Action {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Tab => return Action::Route(Route::Review),
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        let content = self.editor.lines().join("\n");
                        self.editor = TextArea::default();
                        db.add(Card::new(content));
                        return Action::Render;
                    } else {
                        self.editor.input(key);
                        return Action::Render;
                    }
                }
                KeyCode::Char('p') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        // todo: toggle preview
                    } else {
                        self.editor.input(key);
                        return Action::Render;
                    }
                }
                _ => {
                    self.editor.input(key);
                    return Action::Render;
                }
            }
        }

        Action::None
    }

    pub fn on_paste(&mut self, s: String) -> Action {
        //todo
        Action::None
    }

    pub fn on_exit(&mut self) {
        //todo
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        &[
            SHORTCUT_SAVE,
            SHORTCUT_PREVIEW,
            SHORTCUT_MENU,
            SHORTCUT_QUIT,
        ]
    }
}

pub struct EditCard {
    card_id: CardId,
    // editor: TextArea<'static>,
    editor: TextEditor,
}

impl EditCard {
    pub fn new() -> Self {
        Self {
            card_id: CardId::default(),
            // editor: TextArea::default(),
            editor: TextEditor::new(),
        }
    }

    pub fn on_enter(&mut self, card_id: CardId, db: &Database) {
        self.card_id = card_id;
        // self.editor.insert_str(db.get(&card_id).unwrap().0.as_str());
        // self.editor
        //     .input
        //     .push_str(db.get(&card_id).unwrap().0.as_str());
        self.editor.input.push_str(
            r#"
yoyo

this is a line     that is super      long     and should definitively          wrap

kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk

bye"#,
        );
        self.editor.move_cursor(CursorMove::End);
    }

    pub fn on_render(&self, area: Rect, buf: &mut Buffer) {
        self.editor.render(area, buf);
    }

    pub fn on_input(&mut self, key: KeyEvent, db: &mut Database) -> Action {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Tab => return Action::Route(Route::AddCard),
                KeyCode::Right => {
                    self.editor.move_cursor(CursorMove::Forward);
                    return Action::Render;
                }
                KeyCode::Left => {
                    self.editor.move_cursor(CursorMove::Back);
                    return Action::Render;
                }
                KeyCode::Up => {
                    self.editor.move_cursor(CursorMove::Start);
                    return Action::Render;
                }
                KeyCode::Down => {
                    self.editor.move_cursor(CursorMove::End);
                    return Action::Render;
                }
                KeyCode::Char('s') => {
                    // if key.modifiers.contains(KeyModifiers::CONTROL) {
                    //     // todo: save and go back
                    //     let card = db.get_mut(&self.card_id).unwrap();
                    //     card.0 = self.editor.lines().join("\n");
                    //     return Action::Route(Route::Review);
                    // } else {
                    //     self.editor.input(key);
                    //     return Action::Render;
                    // }
                }
                KeyCode::Char('c') => {
                    // if key.modifiers.contains(KeyModifiers::CONTROL) {
                    //     // todo: cancel and go back
                    //     return Action::Route(Route::Review);
                    // } else {
                    //     self.editor.input(key);
                    //     return Action::Render;
                    // }
                }
                _ => {
                    // self.editor.input(key);
                    // return Action::Render;
                }
            }
        }

        Action::None
    }

    pub fn on_paste(&mut self, s: String) -> Action {
        //todo
        Action::None
    }

    pub fn on_exit(&mut self) {
        self.card_id = CardId::default();
        // self.editor = TextArea::default();
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        &[SHORTCUT_SAVE, SHORTCUT_CANCEL, SHORTCUT_QUIT]
    }
}

struct TextEditor {
    input: String,
    cursor: usize,
}

impl TextEditor {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
        }
    }

    fn move_cursor(&mut self, cm: CursorMove) {
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
            CursorMove::Start => self.cursor = 0,
            CursorMove::End => self.cursor = self.input.len(),
        }
    }

    fn push_char(&mut self, c: char) {
        self.input.push(c);
        todo!()
    }

    fn push_str(&mut self, s: &str) {
        self.input.push_str(s);
        todo!()
    }

    fn render(&self, mut area: Rect, buf: &mut Buffer) {
        const STYLE_CURSOR: Style = Style::new().bg(Color::White).fg(Color::Black);
        const STYLE_NONE: Style = Style::new();

        let mut col = 0;
        let mut row = 0;
        let mut i = 0;
        let mut sum = self.input.len();
        let mut line_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        for (newlines, line) in LineParser::new(&self.input, area.width as usize).enumerate() {
            //todo
            // i += line.len();
            let len = line.len() + 1;
            // if i + len < self.cursor {
            //     sum -= len;
            //     i += len;
            // }
            let line_end = i + len;
            if i <= self.cursor && line_end >= self.cursor {
                // let diff = line_end - self.cursor;
                col = self.cursor - i;
                row = newlines;
                // dbg!(col);
                // dbg!(line_end);
                // dbg!(newlines);
                // dbg!(self.input.len());
                // let extra_newlines = line_end - self.input.len();
                // if line_end >= self.input.len() {
                //     col += 1;
                // }
                if self.cursor == self.input.len() {
                    // col += 1;
                }
            }
            i = line_end;
            // row += 1;
            Line::raw(line).render(line_area, buf);
            line_area.y += 1;
        }

        buf[(area.x + col as u16, area.y + row as u16)].set_bg(Color::Green);

        // let mut char_area = Rect::new(area.x, area.y, 1, 1);
        // let wrapped = textwrap::fill(
        //     &self.input,
        //     textwrap::Options::new(area.width as usize)
        //         .wrap_algorithm(textwrap::WrapAlgorithm::FirstFit),
        // );

        // fn count_newlines_until(s: &str, i: usize) -> usize {
        //     let mut newlines = 0;
        //     let mut chars = s.char_indices();
        //     loop {
        //         let Some((_i, c)) = chars.next() else {
        //             break;
        //         };

        //         if _i > i {
        //             break;
        //         }

        //         if c == '\n' {
        //             newlines += 1;
        //         }
        //     }
        //     newlines
        // }

        // let mut offset = 0;
        // for (i, c) in wrapped.char_indices() {
        //     let is_cursor_pos = i == self.cursor;
        //     match c {
        //         '\n' => {
        //             if is_cursor_pos {
        //                 Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
        //             }
        //             char_area.y += 1;
        //             char_area.x = area.x;
        //         }
        //         _ => {
        //             let style = if is_cursor_pos {
        //                 STYLE_CURSOR
        //             } else {
        //                 STYLE_NONE
        //             };
        //             Span::styled(c.to_string(), style).render(char_area, buf);
        //             char_area.x += 1;
        //         }
        //     }
        // }

        // if self.cursor == self.input.len() {
        //     Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
        // }

        // if let Some((i, c)) = self.input.char_indices().find(|(i, _)| *i == self.cursor) {
        //     //todo

        // }

        // for (i, c) in self.input.char_indices() {
        //     let is_cursor_pos = i == self.cursor;
        //     match c {
        //         '\n' => {
        //             if is_cursor_pos {
        //                 Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
        //             }
        //             char_area.y += 1;
        //             char_area.x = area.x;
        //         }
        //         _ => {
        //             let style = if is_cursor_pos {
        //                 STYLE_CURSOR
        //             } else {
        //                 STYLE_NONE
        //             };
        //             Span::styled(c.to_string(), style).render(char_area, buf);
        //             char_area.x += c.len_utf8() as u16;
        //         }
        //     }
        // }

        // if self.cursor == self.input.len() {
        //     Span::styled(" ", STYLE_CURSOR).render(char_area, buf);
        // }
    }
}

enum CursorMove {
    Forward,
    Back,
    Start,
    End,
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
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.text.len() {
            return None;
        }

        let mut width = 0; // todo: utf8
        loop {
            let Some((i, c)) = self.chars.next() else {
                let line = &self.text[self.start..];
                self.start = self.text.len();
                return Some(line);
            };

            match c {
                '\n' => {
                    let line = &self.text[self.start..i];
                    self.start = i + 1;
                    return Some(line);
                }
                _ => {
                    width += 1;
                    if width >= self.line_width {
                        let end = i + c.len_utf8();
                        let line = &self.text[self.start..end];
                        self.start = end;
                        return Some(line);
                    }
                }
            }
        }
    }
}
