mod utils;
mod app;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal,
};

use ratatui::prelude::*;
use ratatui::widgets::{Borders, List, Wrap};
use strum::IntoEnumIterator;
use crate::app::Game;
use crate::utils::{ToDuration, TrimMargin};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    running: bool,
    margin_size: u16,
    rects: u16,
    frame_counter: u64,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let top_bottom = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);

        let debug_area = top_bottom[0];

        Block::bordered()
            .title("Debug Info")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
            .render(debug_area, buf);

        Block::bordered()
            .border_style(Style::default().fg(Color::Magenta))
            .render(area, buf);

        let debug_content = Paragraph::new(format!(
            "Hackerman Suite of Minigames, Frames: {}", self.frame_counter
        ));
        let debug_inner = Layout::default()
            .horizontal_margin(2)
            .vertical_margin(1)
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1)])
            .split(top_bottom[0])[0];
        debug_content.render(debug_inner, buf);

        let main_area = top_bottom[1];

        let left_right = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(28), Constraint::Min(24),])
            .margin(1)
            .split(main_area);

        let left_area = left_right[0];

        // list of games
        Block::bordered()
            .title("Ratatui Simple Template")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
            .render(left_area, buf);

        let games = Game::iter().map(|game| game.to_string()).collect::<Vec<String>>();

        ratatui::prelude::Widget::render(
            List::new(games) // Game List
                .block(Block::default().borders(Borders::ALL).title("Games"))
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(">> ")
                .repeat_highlight_symbol(true)
            , left_area
            , buf
        );
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            margin_size: 0,
            rects: 1,
            frame_counter: 0,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;

        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

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
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q')) => self.quit(),
            (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Up) => self.margin_size += 1,
            (_, KeyCode::Down) if self.margin_size > 0 => self.margin_size -= 1,
            (_, KeyCode::Left) if self.rects > 1 => self.rects -= 1,
            (_, KeyCode::Right) if self.rects < 10 => self.rects += 1,
            _ => {}
        }
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
    fn app_new() {
        let app = App::new();
        assert!(app.running);
        assert_eq!(app.margin_size, 0);
        assert_eq!(app.rects, 1);
    }

    #[test]
    fn app_quit() {
        let mut app = App::new();
        app.quit();
        assert!(!app.running);
    }

    #[test]
    fn app_handle_key_event() {
        let mut app = App::new();
        assert!(app.running);
        assert_eq!(app.margin_size, 0);
        assert_eq!(app.rects, 1);
        
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        app.on_key_event(key_event);
        assert!(!app.running);

        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        app.on_key_event(key_event);
        assert_eq!(app.margin_size, 1);

        let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        app.on_key_event(key_event);
        assert_eq!(app.margin_size, 0);

        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::empty());
        app.rects = 2;
        app.on_key_event(key_event);
        assert_eq!(app.rects, 1);

        let key_event = KeyEvent::new(KeyCode::Right, KeyModifiers::empty());
        app.on_key_event(key_event);
        assert_eq!(app.rects, 2);
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