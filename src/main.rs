mod utils;
mod app;
mod games;

use std::time::Instant;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::{prelude, text::Line, widgets::{Block, Paragraph}, DefaultTerminal};

use crate::app::{handle_input, GameName, InputMode};
use crate::utils::{ToDuration, TrimMargin};
use ratatui::prelude::*;
use ratatui::widgets::{Borders, HighlightSpacing, List, ListState};
use strum::IntoEnumIterator;
use crate::games::game_widget::WidgetGame;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

struct Games {
    games: Vec<GameName>,
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
    current_game: Option<Box<dyn WidgetGame>>,
    input_mode: InputMode,
    games_state: Games,
    refresh_without_inputs: bool,
    frame_times: Vec<Instant>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            frame_counter: 0,
            input_mode: InputMode::GameSelection,
            games_state: Games {
                games: GameName::iter().collect(),
                list_state: ListState::default().with_selected(Some(0)),
            },
            refresh_without_inputs: false,
            frame_times: Vec::new(),
            current_game: None,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;

        while self.running {
            if self.frame_times.len() > 10 {
                self.frame_times.remove(0);
            }

            self.frame_times.push(Instant::now());

            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            if let Some(game) = &mut self.current_game {
                game.run();
            }

            self.frame_counter += 1;

            if self.refresh_without_inputs {
                // real time mode: poll for events every 16 milliseconds, do not block otherwise
                if event::poll(16.milliseconds())? {
                    self.handle_crossterm_events()?;
                }
            } else {
                // performance mode: block thread until an input event occurs
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
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_press(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_press(&mut self, key: KeyEvent) -> () {
        handle_input(self, key).unwrap_or_else(|e| eprintln!("Error handling input: {}", e));
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn get_fps(&self) -> f64 {
        let average_frame_time = if self.frame_times.len() > 1 {
            let duration = self.frame_times.last().unwrap().duration_since(self.frame_times.first().unwrap().clone());
            duration.as_secs_f64() / (self.frame_times.len() as f64 - 1.0)
        } else {
            0.0
        };

        if average_frame_time > 0.0 {
            1.0 / average_frame_time
        } else {
            0.0
        }
    }

    pub fn render_games_list(&mut self, area: Rect, buf: &mut Buffer) {
        let highlight_color = if self.input_mode == InputMode::GameSelection {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let border_style = Style::default().fg(highlight_color);

        let games = self.games_state.games.iter()
            .map(|game| Line::from(game.to_string()))
            .collect::<Vec<_>>();

        let games_list = List::new(games.clone())
            .block(Block::default().borders(Borders::ALL).border_style(border_style).title("Games List").title_alignment(Alignment::Center))
            .highlight_style(Style::default().fg(highlight_color).bold())
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::WhenSelected)
            .repeat_highlight_symbol(true);

        prelude::StatefulWidget::render(
            games_list, area, buf, &mut self.games_state.list_state
        );
    }

    pub fn render_game_details(&mut self, area: Rect, buf: &mut Buffer) {
        let border_style = if matches!(self.input_mode, InputMode::Game(_)) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let selected_game_name = self.games_state.list_state.selected()
            .and_then(|index| self.games_state.games.get(index));

        let details_content = match selected_game_name {
            Some(game) => Paragraph::new(game.to_string()),
            None => Paragraph::new("No game selected."),
        };

        let game_title = selected_game_name
            .map(|game| game.to_string())
            .unwrap_or_else(|| "Game Details".to_string());

        Block::bordered()
            .border_style(border_style)
            .title(game_title)
            .title_alignment(Alignment::Center)
            .render(area, buf);

        let details_inner = Layout::default()
            .horizontal_margin(2)
            .vertical_margin(1)
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1)])
            .split(area)[0];

        details_content.render(details_inner, buf);
    }

    pub fn render_top_area(&self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .title("Debug Info")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
            .render(area, buf);

        let debug_content = Paragraph::new(format!(
            "Loop Mode: {}, Input Mode: {}, Frames: {}, FPS: {:.2}",
            if self.refresh_without_inputs { "Real Time" } else { "Performance" },
            self.input_mode.to_string(),
            self.frame_counter,
            self.get_fps()
        ));

        let debug_inner = Layout::default()
            .horizontal_margin(2)
            .vertical_margin(1)
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1)])
            .split(area)[0];
        debug_content.render(debug_inner, buf);
    }

    pub fn render_bottom_area(&self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .title("Controls")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
            .render(area, buf);

        let controls_content = Paragraph::new(
            "Use arrow keys to navigate the games list. ENTER to select game. CTRL+C to exit."
        );

        let controls_inner = Layout::default()
            .horizontal_margin(2)
            .vertical_margin(1)
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1)])
            .split(area)[0];
        controls_content.render(controls_inner, buf);
    }

    pub fn render_main_area(&mut self, main_area: Rect, buf: &mut Buffer) {
        let [left_area, right_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(28), Constraint::Min(24),])
            .areas(main_area);

        self.render_games_list(left_area, buf);

        match &self.current_game {
            Some(game) => game.render_ref(right_area, buf),
            None => self.render_game_details(right_area, buf),
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top_area, main_area, bottom_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .areas(area);

        self.render_top_area(top_area, buf);
        self.render_main_area(main_area, buf);
        self.render_bottom_area(bottom_area, buf);
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