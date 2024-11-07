use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;

use crate::{app::Action, database::*, editor::*, markup::*, utils::*};

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
    editor: TextEditor,
    preview: bool,
    scroll: usize,
}

impl AddCard {
    pub fn new() -> Self {
        Self {
            editor: TextEditor::new(),
            preview: false,
            scroll: 0,
        }
    }

    pub fn on_enter(&mut self, _db: &Database) {
        //todo
    }

    pub fn on_render(&mut self, area: Rect, buf: &mut Buffer) {
        if self.preview {
            Markup::new(self.editor.as_str()).render(area, buf, &mut self.scroll);
        } else {
            self.editor.render(area, buf);
        }
    }

    pub fn on_input(&mut self, key: KeyEvent, db: &mut Database) -> Action {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Tab => return Action::Route(Route::Review),
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        db.add(Card::new(self.editor.as_str().to_owned()));
                        self.editor.clear();
                        self.preview = false;
                        self.scroll = 0;
                        return Action::Render;
                    } else if !self.preview {
                        self.editor.push_char('s');
                        return Action::Render;
                    }
                }
                KeyCode::Char('p') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.preview = !self.preview;
                    } else if !self.preview {
                        self.editor.push_char('p');
                    }
                    return Action::Render;
                }
                _ => {
                    if !self.preview {
                        self.editor.input(key.code, key.modifiers);
                        return Action::Render;
                    }
                }
            }
        }

        Action::None
    }

    pub fn on_paste(&mut self, _s: String) -> Action {
        //todo
        Action::None
    }

    pub fn on_exit(&mut self) {
        self.preview = false;
        self.scroll = 0;
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
    editor: TextEditor,
    preview: bool,
    scroll: usize,
}

impl EditCard {
    pub fn new() -> Self {
        Self {
            card_id: CardId::default(),
            editor: TextEditor::new(),
            preview: false,
            scroll: 0,
        }
    }

    pub fn on_enter(&mut self, card_id: CardId, db: &Database) {
        let card = db.get(&card_id).unwrap();
        self.card_id = card_id;
        self.editor.push_str(card.0.as_str());
        self.editor.move_cursor(CursorMove::Start, false);
    }

    pub fn on_render(&mut self, area: Rect, buf: &mut Buffer) {
        if self.preview {
            Markup::new(self.editor.as_str()).render(area, buf, &mut self.scroll);
        } else {
            self.editor.render(area, buf);
        }
    }

    pub fn on_input(&mut self, key: KeyEvent, db: &mut Database) -> Action {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        let card = db.get_mut(&self.card_id).unwrap();
                        card.0 = self.editor.as_str().to_owned();
                        return Action::Route(Route::Review); // todo: go back
                    } else if !self.preview {
                        self.editor.push_char('s');
                        return Action::Render;
                    }
                }
                KeyCode::Char('c') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        return Action::Route(Route::Review); // todo: go back
                    } else if !self.preview {
                        self.editor.push_char('c');
                        return Action::Render;
                    }
                }
                KeyCode::Char('p') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.preview = !self.preview;
                    } else if !self.preview {
                        self.editor.push_char('p');
                    }
                    return Action::Render;
                }
                _ => {
                    if !self.preview {
                        self.editor.input(key.code, key.modifiers);
                        return Action::Render;
                    }
                }
            }
        }

        Action::None
    }

    pub fn on_paste(&mut self, _s: String) -> Action {
        // todo?
        Action::None
    }

    pub fn on_exit(&mut self) {
        self.editor.clear();
        self.preview = false;
        self.scroll = 0;
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        &[
            SHORTCUT_SAVE,
            SHORTCUT_CANCEL,
            SHORTCUT_PREVIEW,
            SHORTCUT_QUIT,
        ]
    }
}
