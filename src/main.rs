mod utils;
mod app;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::{prelude, text::Line, widgets::{Block, Paragraph}, DefaultTerminal};

use crate::app::{handle_input, Game, InputMode};
use crate::utils::{ToDuration, TrimMargin};
use ratatui::prelude::*;
use ratatui::widgets::{HighlightSpacing, List, ListState};
use strum::IntoEnumIterator;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

struct Games {
    games: Vec<Game>,
    list_state: ListState,
}

impl Games {
    fn select_previous(&mut self) {
        self.list_state.select_previous();
    }

    fn select_next(&mut self) {
        self.list_state.select_next();
    }
}

pub struct App {
    running: bool,
    frame_counter: u64,
    input_mode: InputMode,
    games_state: Games,
}

impl App {
    pub fn render_games_list(&mut self, area: Rect, buf: &mut Buffer) {
        // Block::bordered().border_style(Style::default().fg(Color::Magenta)).render(area, buf);

        let games = self.games_state.games.iter()
            .map(|game| Line::from(game.to_string()))
            .collect::<Vec<_>>();

        let games_list = List::new(games.clone())
            .block(Block::bordered().title("Games"))
            .highlight_style(Style::default().fg(Color::Green).bold())
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::WhenSelected)
            .repeat_highlight_symbol(true);

        prelude::StatefulWidget::render(
            games_list, area, buf, &mut self.games_state.list_state
        );
    }

    pub fn render_top_area(&self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .title("Debug Info")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
            .render(area, buf);

        Block::bordered()
            .border_style(Style::default().fg(Color::Magenta))
            .render(area, buf);

        let debug_content = Paragraph::new(format!(
            "Input Mode: {}, Frames: {}", self.input_mode.to_string(), self.frame_counter,
        ));

        let debug_inner = Layout::default()
            .horizontal_margin(2)
            .vertical_margin(1)
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1)])
            .split(area)[0];
        debug_content.render(debug_inner, buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw box for the whole game
        Block::bordered().border_style(Style::default().fg(Color::Magenta)).render(area, buf);

        let [debug_area, main_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .areas(area);

        self.render_top_area(debug_area, buf);

        let [left_area, right_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(28), Constraint::Min(24),])
            .margin(1)
            .areas(main_area);

        self.render_games_list(left_area, buf);
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            frame_counter: 0,
            input_mode: InputMode::GameSelection,
            games_state: Games {
                games: Game::iter().collect(),
                list_state: ListState::default().with_selected(Some(0)),
            },
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;

        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            self.frame_counter += 1;

            if event::poll(16.milliseconds())? {
                self.handle_crossterm_events()?;
            }
        }

        Ok(())
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) -> () {
        handle_input(self, key).unwrap_or_else(|e| eprintln!("Error handling input: {}", e));
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer_to_string(buf: &Buffer) -> String {
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}