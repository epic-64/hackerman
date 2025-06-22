use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

use ratatui::prelude::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    margin_size: u16,
    rects: u16,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: false,
            margin_size: 1,
            rects: 1,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .margin(self.margin_size)
            .split(frame.area());

        let title = Line::from("Ratatui Simple Template").bold().blue().centered();
        let top_area = layout[0];
        let top_content = Paragraph::new(text)
            .block(Block::bordered().title(title))
            .centered();

        frame.render_widget(top_content, top_area);

        let bottom_area = layout[1];
        let bottom_title = Line::from("Rectangles").bold().green().centered();
        let bottom_content = Paragraph::new("This area contains rectangles.")
            .block(Block::bordered().title(bottom_title))
            .centered();

        frame.render_widget(bottom_content, bottom_area);

        let constraints = (0 .. self.rects as usize)
            .map(|_| Constraint::Min(1))
            .collect::<Vec<_>>();

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .margin(1)
            .split(layout[1]);

        for i in 0..self.rects as usize {
            frame.render_widget(
                Paragraph::new("This is a rectangle.")
                    .block(Block::bordered().title("Rectangle"))
                    .centered(),
                inner_layout[i],
            );
        }
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
