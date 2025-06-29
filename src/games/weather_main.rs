use crate::games::main_screen_widget::{MainScreenWidget, WidgetRef};
use crossterm::event::KeyEvent;
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub struct WeatherMain {
    exit_intended: bool,
}

impl WeatherMain {
    pub fn new() -> Self {
        Self { exit_intended: false }
    }
}

impl MainScreenWidget for WeatherMain {
    fn run(&mut self, _dt: f64) {}

    fn handle_input(&mut self, _input: KeyEvent) -> () {}

    fn is_exit_intended(&self) -> bool { self.exit_intended }
}

impl WidgetRef for WeatherMain {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let width = [Constraint::Length(40)];
        let height = [Constraint::Length(3)];

        // create centered area with a specific width and height
        let [middle] = Layout::vertical(height).flex(Flex::Center).areas(area);
        let [center] = Layout::horizontal(width).flex(Flex::Center).areas(middle);

        // create left and right areas in the center
        let widths = [Constraint::Fill(10), Constraint::Length(20)];
        let [left, right] = Layout::horizontal(widths).areas(center);

        let left_content = Paragraph::new(Text::from(vec![
            Line::from("Current Temp:"),
            Line::from("Feels Like:"),
            Line::from("Weather Summary:"),
        ])).left_aligned();

        let right_content = Paragraph::new(Text::from(vec![
            Line::from("20°C"),
            Line::from("18°C"),
            Line::from("Moderately Cloudy"),
        ])).centered();

        left_content.render(left, buf);
        right_content.render(right, buf);
    }
}