use crate::games::binary_numbers::Bits;
use crate::games::main_screen_widget::MainScreenWidget;
use crate::games::settings::SettingsMain;
use crate::games::weather_main::WeatherMain;
use crate::games::{ascii_art, binary_numbers};
use crate::utils::{ToDuration, When};
use ascii_art::AsciiArtMain;
use binary_numbers::BinaryNumbersGame;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment::Center;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListState, Paragraph};
use ratatui::{prelude, DefaultTerminal};
use std::time::Instant;
use std::{cmp, thread};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(EnumIter, Display, Clone, PartialEq)]
pub enum MainMenuEntry {
    Settings,
    Weather,
    AsciiArt,
    BinaryNumbers,
    DinoJump,
    Exit,
}

impl MenuEntry for MainMenuEntry {
    fn name(&self) -> &str {
        match self {
            MainMenuEntry::Settings => "Settings",
            MainMenuEntry::Weather => "Weather",
            MainMenuEntry::AsciiArt => "Ascii Art",
            MainMenuEntry::BinaryNumbers => "Binary Numbers",
            MainMenuEntry::DinoJump => "Dino Jump",
            MainMenuEntry::Exit => "Exit",
        }
    }
}

impl MainMenuEntry {
    pub fn get_main_screen_widget(&self) -> Option<Box<dyn MainScreenWidget>> {
        match self {
            MainMenuEntry::Settings => Some(Box::new(SettingsMain::new())),
            MainMenuEntry::Weather => Some(Box::new(WeatherMain::new())),
            MainMenuEntry::AsciiArt => Some(Box::new(AsciiArtMain::new())),
            MainMenuEntry::BinaryNumbers => Some(Box::new(BinaryNumbersGame::new(Bits::Eight))),
            MainMenuEntry::DinoJump => None, // Dino Jump is not implemented yet
            MainMenuEntry::Exit => None, // Exit does not return a widget
        }
    }
}

pub fn handle_input(app: &mut App, input: KeyEvent) -> color_eyre::Result<()> {
    match input.code {
        KeyCode::Char('c') | KeyCode::Char('C') if input.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }
        KeyCode::Char(' ') => app.refresh_without_inputs = !app.refresh_without_inputs,
        KeyCode::Esc => app.current_main_widget = None,
        KeyCode::F(2) => match app.current_main_widget {
            None => {}
            Some(ref game) => {}
        },
        KeyCode::F(4) => {
            // Debug mode toggle
            app.debug_mode = !app.debug_mode;
        }
        _ => {}
    }

    match &mut app.current_main_widget {
        None => handle_main_menu_inputs(app, input),
        Some(game) => game.handle_input(input),
    }
    Ok(())
}

fn handle_main_menu_inputs(app: &mut App, input: KeyEvent) -> () {
    app.main_menu.handle_navigation(input);

    match input.code {
        KeyCode::Enter => {
            if app.main_menu.get_selected_entry() == Some(&MainMenuEntry::Exit) {
                app.quit();
                return;
            }

            app.current_main_widget = match app.main_menu.get_selected_entry() {
                Some(entry) => entry.get_main_screen_widget(),
                None => None,
            }
        }
        _ => {}
    }
}

#[derive(Clone)]
pub enum MenuOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
struct StatefulMenu<T> {
    orientation: MenuOrientation,
    items: Vec<T>,
    state: ListState,
}

pub trait MenuEntry {
    fn name(&self) -> &str;
}

impl<T: MenuEntry> StatefulMenu<T> {
    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }

    fn get_selected_entry(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    fn handle_navigation(&mut self, input: KeyEvent) -> () {
        match self.orientation {
            MenuOrientation::Horizontal => match input.code {
                KeyCode::Left => self.select_previous(),
                KeyCode::Right => self.select_next(),
                _ => {}
            }
            MenuOrientation::Vertical => match input.code {
                KeyCode::Up => self.select_previous(),
                KeyCode::Down => self.select_next(),
                _ => {}
            }
        }
    }

    fn get_lines(&self) -> Vec<Line> {
        self.items.iter().map(|item| Line::from(item.name())).collect()
    }
}

pub struct App {
    running: bool,
    debug_mode: bool,
    frame_counter: u64,
    current_main_widget: Option<Box<dyn MainScreenWidget>>,
    main_menu: StatefulMenu<MainMenuEntry>,
    refresh_without_inputs: bool,
    frame_times: Vec<Instant>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            debug_mode: true,
            frame_counter: 0,
            main_menu: StatefulMenu {
                orientation: MenuOrientation::Vertical,
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
        let target_frame_duration = 16.milliseconds(); // Target frame duration for 30 FPS

        while self.running {
            let now = Instant::now();
            let dt = now - last_frame_time;
            last_frame_time = now;

            if self.frame_times.len() > 10 {
                self.frame_times.remove(0);
            }

            self.frame_times.push(Instant::now());

            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            if let Some(widget) = &mut self.current_main_widget {
                widget.run(dt.as_secs_f64());

                if widget.is_exit_intended() {
                    self.current_main_widget = None;
                }
            }

            self.frame_counter += 1;

            if self.refresh_without_inputs {
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

    pub fn render_main_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let highlight_color = Color::LightCyan;

        let menu_is_active = self.current_main_widget.is_none();

        let binding = self.main_menu.clone();
        let menu_lines = binding.get_lines();

        let games_list = List::new(menu_lines)
            .block(Block::default().borders(Borders::ALL)
                .title("Main Menu").title_alignment(Center)
            )
            .highlight_style(Style::default().fg(highlight_color).bold())
            .highlight_symbol("> ")
            .when(!menu_is_active, |list| list.dim())
            .highlight_spacing(HighlightSpacing::WhenSelected)
            .repeat_highlight_symbol(true);

        prelude::StatefulWidget::render(games_list, area, buf, &mut self.main_menu.state);
    }

    pub fn render_game_details(&mut self, area: Rect, buf: &mut Buffer) {
        let selected_game_name = self.main_menu.get_selected_entry();

        let details_content = match selected_game_name {
            Some(game) => Paragraph::new(game.to_string()),
            None => Paragraph::new("No game selected."),
        };

        details_content.render(area, buf);
    }

    pub fn render_main_widget(&mut self, area: Rect, buf: &mut Buffer) {
        let is_active = self.current_main_widget.is_some();

        Block::bordered()
            .when(!is_active, |block| block.dim())
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
        if !self.debug_mode {
            return;
        }

        let content = format!(
            "Loop Mode: {}, FPS: {:.0}",
            if self.refresh_without_inputs { "Real Time" } else { "Performance" },
            self.get_fps()
        );

        Paragraph::new(content)
            .block(Block::bordered().border_style(Style::default().dark_gray()).title("Debug"))
            .render(area, buf);
    }

    pub fn render_bottom_area(&self, area: Rect, buf: &mut Buffer) {
        if !self.debug_mode {
            return;
        }

        Paragraph::new("<F1> Overview | <F2> Settings | <F4> Debug | <Space> Pause, <Ctrl+C> Quit")
            .block(Block::bordered().border_style(Style::default().dark_gray()).title("Controls"))
            .render(area, buf);
    }

    pub fn render_middle_area(&mut self, main_area: Rect, buf: &mut Buffer) {
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(28), Constraint::Min(24),])
            .areas(main_area);

        self.render_main_menu(left, buf);
        self.render_main_widget(right, buf);
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
        self.render_middle_area(main_area, buf);
        self.render_bottom_area(bottom_area, buf);
    }
}