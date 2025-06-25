mod utils;
mod app;
mod games;

use std::{cmp, thread};
use crate::app::{handle_input, MainMenuEntry};
use crate::games::main_screen_widget::MainScreenWidget;
use crate::utils::{ToDuration, TrimMargin};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Borders, HighlightSpacing, List, ListState};
use ratatui::{prelude, text::Line, widgets::{Block, Paragraph}, DefaultTerminal};
use std::time::Instant;
use strum::IntoEnumIterator;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

struct MainMenu {
    items: Vec<MainMenuEntry>,
    state: ListState,
}

impl MainMenu {
    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }

    fn get_selected_entry(&self) -> Option<&MainMenuEntry> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
}

pub struct App {
    running: bool,
    frame_counter: u64,
    current_main_widget: Option<Box<dyn MainScreenWidget>>,
    main_menu: MainMenu,
    refresh_without_inputs: bool,
    frame_times: Vec<Instant>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            frame_counter: 0,
            main_menu: MainMenu {
                items: MainMenuEntry::iter().collect(),
                state: ListState::default().with_selected(Some(0)),
            },
            refresh_without_inputs: true,
            frame_times: Vec::new(),
            current_main_widget: None,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let mut last_frame_time = Instant::now(); // Initialize previous time
        let target_frame_duration = 33.milliseconds(); // Target frame duration for 30 FPS

        while self.running {
            let now = Instant::now();
            let dt = now - last_frame_time;
            last_frame_time = now;

            if self.frame_times.len() > 10 {
                self.frame_times.remove(0);
            }

            self.frame_times.push(Instant::now());

            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            if let Some(game) = &mut self.current_main_widget {
                game.run();

                if game.is_exit_intended() {
                    self.current_main_widget = None;
                }
            }

            self.frame_counter += 1;

            if self.refresh_without_inputs {
                // Use dt-based timeout, capped to avoid very large values
                let poll_timeout = cmp::min(dt, target_frame_duration);
                if event::poll(poll_timeout)? {
                    self.handle_crossterm_events()?;
                }
            } else {
                // performance mode: block thread until an input event occurs
                self.handle_crossterm_events()?;
            }

            // Optional: sleep to avoid running too fast
            let frame_duration = last_frame_time.elapsed();
            if frame_duration < target_frame_duration {
                thread::sleep(target_frame_duration - frame_duration);
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
        let highlight_color = if self.current_main_widget.is_none() {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let border_style = Style::default().fg(highlight_color);

        let games = self.main_menu.items.iter()
            .map(|game| Line::from(game.to_string()))
            .collect::<Vec<_>>();

        let games_list = List::new(games.clone())
            .block(Block::default().borders(Borders::ALL).border_style(border_style)
                .title("Main Menu").title_alignment(Alignment::Center))
            .highlight_style(Style::default().fg(highlight_color).bold())
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::WhenSelected)
            .repeat_highlight_symbol(true);

        prelude::StatefulWidget::render(
            games_list, area, buf, &mut self.main_menu.state
        );
    }

    pub fn render_game_details(&mut self, area: Rect, buf: &mut Buffer) {
        let selected_game_name = self.main_menu.state.selected()
            .and_then(|index| self.main_menu.items.get(index));

        let details_content = match selected_game_name {
            Some(game) => Paragraph::new(game.to_string()),
            None => Paragraph::new("No game selected."),
        };

        details_content.render(area, buf);
    }

    pub fn render_game_box(&mut self, area: Rect, buf: &mut Buffer) {
        let border_style = if matches!(self.current_main_widget, Some(_)) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let selected_game_name = self.main_menu.state.selected()
            .and_then(|index| self.main_menu.items.get(index));

        let game_title = selected_game_name
            .map(|game| game.to_string())
            .unwrap_or_else(|| "Game Details".to_string());

        Block::bordered()
            .border_style(border_style)
            .title(game_title)
            .title_alignment(Alignment::Center)
            .render(area, buf);

        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        match &self.current_main_widget {
            Some(main_widget) => main_widget.render_ref(inner_area, buf),
            None => self.render_game_details(inner_area, buf),
        }
    }

    pub fn render_top_area(&self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .title("Debug Info")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
            .render(area, buf);

        let debug_content = Paragraph::new(format!(
            "Loop Mode: {}, Selected Game: {} Frames: {}, FPS: {:.2}",
            if self.refresh_without_inputs { "Real Time" } else { "Performance" },
            self.main_menu.state.selected()
                .and_then(|i| self.main_menu.items.get(i))
                .map_or("None".to_string(), |game| game.to_string()),
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
        self.render_game_box(right_area, buf);
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