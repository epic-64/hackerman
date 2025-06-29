use crate::games::main_screen_widget::{MainScreenWidget, WidgetRef};
use crate::utils::{AsciiArtWidget, AsciiCells, TrimMargin};
use crossterm::event::KeyEvent;
use layout::Direction;
use ratatui::prelude::*;
use std::collections::HashMap;
use ratatui::layout::Flex;
use ratatui::widgets::{Block, Borders, Paragraph};
use tui_big_text::{BigText, PixelSize};

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
        let width = 50;
        let height = 3;
        let right_width = 20;
        let left_width_min = 10;

        use Constraint::*;
        let [middle] = Layout::vertical([Length(height)]).flex(Flex::Center).areas(area);
        let [center] = Layout::horizontal([Length(width)]).flex(Flex::Center).areas(middle);
        let [left, right] = Layout::horizontal([Fill(left_width_min), Length(right_width)]).areas(center);

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

fn render_art(area: Rect, buf: &mut Buffer) {
    let art = r"
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⠀⠀⢀⣴⣾⣦⣀⣀⣠⣿⣿⣷⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⠀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⠀⠀⢀⣾⣿⡿⠋⠁⠈⠙⢿⣿⣷⣶⣶⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⢸⣿⣿⣿⣿⡇⠀⠀⠀⠀⢸⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⠘⠛⠛⠻⣿⣷⣤⣀⣀⣴⣿⣿⠏⢀⣀⠀⠀⠀⠀⣾⣿⣿⡇⠀⠀⠀⠀⣀⠀
            ⠀⠀⠀⠀⠀⣾⣿⣿⡿⠿⢿⣿⣿⣷⣿⣿⣧⠀⣀⣀⣿⣿⣿⣇⣀⡀⠀⣼⣿⠀
            ⠀⠀⠀⠀⠸⠿⣿⡿⠀⠀⠀⠻⠿⠋⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠁⢀⣴⣤⣀⢀⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠺⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠿⠿⠿⣿⣿⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠈⢻⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠘⠛⠛⠻⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⣠⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⠛⠛⠛⠛⠛⠛⠛⠛⠂⠀⠀⠀⠀⠒⠛⠛⠛⠛⠀
        ".nice();

    let foreground_colors = r"
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⠀⠀⢀⣴⣾⣦⣀⣀⣠⣿⣿⣷⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⠀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⠀⠀⢀⣾⣿⡿⠋⠁⠈⠙⢿⣿⣷⣶⣶⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⢸⣿⣿⣿⣿⡇⠀⠀⠀⠀⢸⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
            ⠀⠘⠛⠛⠻⣿⣷⣤⣀⣀⣴⣿⣿⠏⢀⣀⠀⠀⠀⠀⣾⣿⣿⡇⠀⠀⠀⠀⣀⠀
            ⠀⠀⠀⠀⠀⣾⣿⣿⡿⠿⢿⣿⣿⣷⣿⣿⣧⠀⣀⣀⣿⣿⣿⣇⣀⡀⠀⣼⣿⠀
            ⠀⠀⠀⠀⠸⠿⣿⡿⠀⠀⠀⠻⠿⠋⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠁⢀⣴⣤⣀⢀⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠺⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠿⠿⠿⣿⣿⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠈⢻⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠘⠛⠛⠻⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⣠⣿⣿⣿⠀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⠛⠛⠛⠛⠛⠛⠛⠛⠂⠀⠀⠀⠀⠒⠛⠛⠛⠛⠀
        ".nice();

    let color_map = HashMap::from([]);

    let cells = AsciiCells::from(art, foreground_colors, &color_map, Color::Blue);

    AsciiArtWidget::new(cells).render(area, buf);
}

fn render_big_text(area: Rect, buf: &mut Buffer) {
    let big_text = BigText::builder()
        .pixel_size(PixelSize::Sextant)
        .style(Style::new().white())
        .lines(vec![
            "Settings".into(),
            "~~~~~~~".into(),
        ])
        .build();

    big_text.render(area, buf);
}