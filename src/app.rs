use crossterm::event::{Event, KeyEventKind};
use ratatui::{prelude::*, CompletedFrame, DefaultTerminal};

use crate::{database::*, pages::*, utils::*};

pub struct App {
    running: bool,
    route: Route,
    pages: Pages,
    db: Database,
}

pub enum Action {
    None,
    Render,
    Route(Route),
    Quit,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            route: Route::Review,
            pages: Pages::new(),
            db: Database::new(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        self.pages.review.on_enter(&self.db);
        self.render(&mut terminal)?;

        while self.running {
            let action = match crossterm::event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match self.route {
                            Route::Review => {
                                self.pages
                                    .review
                                    .on_input(key.code, key.modifiers, &mut self.db)
                            }
                            Route::AddCard => {
                                self.pages
                                    .add_card
                                    .on_input(key.code, key.modifiers, &mut self.db)
                            }
                            Route::EditCard(_) => {
                                self.pages
                                    .edit_card
                                    .on_input(key.code, key.modifiers, &mut self.db)
                            }
                        }
                    } else {
                        Action::None
                    }
                }
                Event::Resize(_, _) => Action::Render,
                _ => Action::None,
            };

            match action {
                Action::None => {}
                Action::Render => {
                    self.render(&mut terminal)?;
                }
                Action::Route(route) => {
                    match self.route {
                        Route::Review => self.pages.review.on_exit(),
                        Route::AddCard => self.pages.add_card.on_exit(),
                        Route::EditCard(_) => self.pages.edit_card.on_exit(),
                    }

                    self.route = route;

                    match route {
                        Route::Review => self.pages.review.on_enter(&self.db),
                        Route::AddCard => self.pages.add_card.on_enter(&self.db),
                        Route::EditCard(id) => self.pages.edit_card.on_enter(id, &self.db),
                    }

                    self.render(&mut terminal)?;
                }
                Action::Quit => {
                    self.running = false;
                }
            }
        }

        Ok(())
    }

    fn render<'a>(
        &'a mut self,
        terminal: &'a mut DefaultTerminal,
    ) -> std::io::Result<CompletedFrame> {
        terminal.draw(|frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();

            let [header, body, footer] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .areas(area);

            // Header
            Line::raw(format!("Lazycard: {}", self.route.title()))
                .alignment(Alignment::Center)
                .render(header, buf);

            // Body
            let body =
                layout_center_horizontal(body.inner(Margin::new(2, 2)), Constraint::Length(64));
            let shortcuts = match self.route {
                Route::Review => {
                    self.pages.review.on_render(body, buf);
                    self.pages.review.shortcuts()
                }
                Route::AddCard => {
                    self.pages.add_card.on_render(body, buf);
                    self.pages.add_card.shortcuts()
                }
                Route::EditCard(_) => {
                    self.pages.edit_card.on_render(body, buf);
                    self.pages.edit_card.shortcuts()
                }
            };

            // Footer
            let mut footer_line = Line::default();
            for &Shortcut { name, key } in shortcuts {
                footer_line.extend([
                    Span::styled(key, STYLE_LABEL),
                    Span::raw(" "),
                    Span::raw(name),
                    Span::raw("  "),
                ]);
            }
            footer_line.spans.pop();
            footer_line.alignment(Alignment::Center).render(footer, buf);
        })
    }
}
