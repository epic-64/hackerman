mod utils;
mod app;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal,
};

use ratatui::prelude::*;
use ratatui::widgets::{Borders, HighlightSpacing, List, ListState, Wrap};
use strum::IntoEnumIterator;
use crate::app::{handle_input, Game, InputMode};
use crate::utils::{ToDuration, TrimMargin};

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

pub struct App {
    running: bool,
    frame_counter: u64,
    selected_game_index: usize,
    input_mode: InputMode,
    games: Games,
}

impl App {
    pub fn render_games_list(&mut self, area: Rect, buf: &mut Buffer) {
        let games = self.games.games.iter()
            .map(|game| Line::from(game.to_string()))
            .collect::<Vec<_>>();

        let games_list = List::new(games.clone())
            .block(Block::default().borders(Borders::ALL).title("Games"))
            .highlight_style(Style::default().fg(Color::Yellow).bold())
            .highlight_symbol(">> ")
            .highlight_spacing(HighlightSpacing::WhenSelected)
            .repeat_highlight_symbol(true);

        ratatui::prelude::StatefulWidget::render(
            games_list, area, buf, &mut self.games.list_state
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
        let [debug_area, main_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .areas(area);

        self.render_top_area(debug_area, buf);
        // draw box for the whole game
        Block::bordered().border_style(Style::default().fg(Color::Magenta)).render(area, buf);

        let [left_area, right_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(28), Constraint::Min(24),])
            .margin(1)
            .areas(main_area);

        // list of games
        Block::bordered()
            .title("Ratatui Simple Template")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
            .render(left_area, buf);

        self.render_games_list(left_area, buf);
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            frame_counter: 0,
            selected_game_index: 0,
            input_mode: InputMode::GameSelection,
            games: Games {
                games: Game::iter().collect(),
                list_state: ListState::default(),
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
    use pretty_assertions::assert_eq;
    use crate::utils::TrimMargin;

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

    #[test]
    fn app_draw_initial_state() {
        let app = App::new();

        let mut buf = Buffer::empty(Rect::new(0, 0, 70, 12));
        app.render(buf.area, &mut buf);

        let expected: String = "
            ┌──────────────────────Ratatui Simple Template───────────────────────┐
            │                           Hello, Ratatui!                          │
            │                                                                    │
            │         Created using https://github.com/ratatui/templates         │
            │            Press `Esc`, `Ctrl-C` or `q` to stop running.           │
            └────────────────────────────────────────────────────────────────────┘
            ┌─────────────────────────────Rectangles─────────────────────────────┐
            │┌Rectangle─────────────────────────────────────────────────────────┐│
            ││                       This is a rectangle.                       ││
            ││                                                                  ││
            │└──────────────────────────────────────────────────────────────────┘│
            └────────────────────────────────────────────────────────────────────┘
        ".nice();
        
        let readable_buf = buffer_to_string(&buf);

        assert_eq!(expected, readable_buf);
    }

    #[test]
    fn app_draw_add_rectangle() {
        let mut app = App::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 70, 12));

        // press Right to add a rectangle
        app.on_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));

        // populate the buffer with the current state of the app
        app.render(buf.area, &mut buf);

        let expected: String = "
            ┌──────────────────────Ratatui Simple Template───────────────────────┐
            │                           Hello, Ratatui!                          │
            │                                                                    │
            │         Created using https://github.com/ratatui/templates         │
            │            Press `Esc`, `Ctrl-C` or `q` to stop running.           │
            └────────────────────────────────────────────────────────────────────┘
            ┌─────────────────────────────Rectangles─────────────────────────────┐
            │┌Rectangle───────────────────────┐┌Rectangle───────────────────────┐│
            ││      This is a rectangle.      ││      This is a rectangle.      ││
            ││                                ││                                ││
            │└────────────────────────────────┘└────────────────────────────────┘│
            └────────────────────────────────────────────────────────────────────┘
        ".nice();
        
        let readable_buf = buffer_to_string(&buf);

        assert_eq!(expected, readable_buf);
    }
}