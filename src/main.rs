mod utils;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal,
};

use ratatui::prelude::*;
use crate::utils::TrimMargin;

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
        let text = format!("

            Hello, Ratatui!

            Created using https://github.com/ratatui/templates
            Press `Esc`, `Ctrl-C` or `q` to stop running.

            {} frames rendered.
        ", self.frame_counter).nice();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .margin(self.margin_size)
            .split(area);

        let title = Line::from("Ratatui Simple Template").bold().blue().centered();
        let top_area = layout[0];
        let top_content = Paragraph::new(text)
            .block(Block::bordered().title(title))
            .centered();

        top_content.render(top_area, buf);

        let bottom_area = layout[1];
        let bottom_title = Line::from("Rectangles").bold().green().centered();
        let bottom_content = Paragraph::new("This area contains rectangles.")
            .block(Block::bordered().title(bottom_title))
            .centered();

        bottom_content.render(bottom_area, buf);

        let constraints = (0 .. self.rects as usize)
            .map(|_| Constraint::Min(1))
            .collect::<Vec<_>>();

        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .margin(1)
            .split(layout[1]);

        for i in 0..self.rects as usize {
            Paragraph::new("This is a rectangle.")
                .block(Block::bordered().title("Rectangle"))
                .centered()
                .render(bottom_layout[i], buf);
        }
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
            self.handle_crossterm_events()?;
            self.frame_counter += 1;
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

    /// Set running to false to quit the application.
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