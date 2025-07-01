use crate::games::main_screen_widget::{MainScreenWidget, WidgetRef};
use crate::utils::{parse_ascii_art, AsciiArtWidget, AsciiCells, TrimMargin};
use crossterm::event::KeyEvent;
use ratatui::layout::Flex::Center;
use ratatui::prelude::*;
use std::collections::HashMap;

pub struct SettingsMain {
    exit_intended: bool,
}

impl SettingsMain {
    pub fn new() -> Self {
        Self { exit_intended: false }
    }
}

impl MainScreenWidget for SettingsMain {
    fn run(&mut self, _dt: f64) {}

    fn handle_input(&mut self, _input: KeyEvent) -> () {}

    fn is_exit_intended(&self) -> bool { self.exit_intended }
}

impl WidgetRef for SettingsMain {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [top, bottom] = Layout::vertical([Constraint::Length(6), Constraint::Fill(20)])
            .areas(area);

        //Block::default().borders(Borders::ALL).render(top, buf);
        //Block::default().borders(Borders::ALL).render(bottom, buf);

        render_big_text(top, buf);
    }
}

fn render_big_text(area: Rect, buf: &mut Buffer) {
    let art = "
        ███████╗███████╗████████╗████████╗██╗███╗   ██╗ ██████╗ ███████╗
        ██╔════╝██╔════╝╚══██╔══╝╚══██╔══╝██║████╗  ██║██╔════╝ ██╔════╝
        ███████╗█████╗     ██║      ██║   ██║██╔██╗ ██║██║  ███╗███████╗
        ╚════██║██╔══╝     ██║      ██║   ██║██║╚██╗██║██║   ██║╚════██║
        ███████║███████╗   ██║      ██║   ██║██║ ╚████║╚██████╔╝███████║
        ╚══════╝╚══════╝   ╚═╝      ╚═╝   ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
    ".nice();

    let colors = "
        ████R██╗███████╗█████B██╗████████╗██╗███╗   ██╗ ██C███╗ ███████╗
        ██╔════╝██╔════╝╚══██╔══╝╚══██╔══╝██║████╗  ██║██╔════╝ ██╔════╝
        ███████╗█G███╗     ██║      ██║   █P║██╔██╗ ██║██║  ███╗██R████╗
        ╚════██║██╔══╝     ██║      Y█║   ██║██║╚██╗██║██║   ██║╚════██║
        ███████║███████╗   ██║      ██║   ██║██║ ╚█G██║╚██████╔╝███████║
        ╚══════╝╚══════╝   ╚═╝      ╚═╝   ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
    ".nice();

    let color_map = HashMap::from([
        ('█', Color::White),
        ('R', Color::LightRed),
        ('G', Color::LightGreen),
        ('B', Color::LightBlue),
        ('Y', Color::LightYellow),
        ('P', Color::LightMagenta),
        ('C', Color::LightCyan),
        ('W', Color::White),
        (' ', Color::Reset),
    ]);

    let default_color = Color::LightBlue;

    let cells = parse_ascii_art(art.to_string(), colors.to_string(), &color_map, default_color);
    let cells = AsciiCells::new(cells);
    let width = cells.get_width();
    let ascii_widget = AsciiArtWidget::new(cells);

    let [centered] = Layout::horizontal([Constraint::Length(width)]).flex(Center).areas(area);
    ascii_widget.render(centered, buf);
}