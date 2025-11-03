use crate::games::main_screen_widget::{MainScreenWidget, WidgetRef};
use crate::utils::{center, When};
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use rand::prelude::SliceRandom;
use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::Alignment::Center;
use ratatui::prelude::{Color, Line, Style, Stylize, Text, Widget};
use ratatui::text::Span;
use ratatui::widgets::BorderType::Double;
use ratatui::widgets::{Block, BorderType, Gauge, Paragraph};

const MAX_LIVES: u32 = 5; // maximum lives attainable via streak bonuses

impl WidgetRef for BinaryNumbersGame {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Create a scoreboard area on top and pass remaining area to puzzle
        let [scoreboard_area, puzzle_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .flex(Flex::Start)
        .horizontal_margin(1)
        .areas(area);

        // Render scoreboard
        Block::bordered()
            .title("Binary Numbers")
            .title_alignment(Center)
            .dark_gray()
            .render(scoreboard_area, buf);

        let hearts = self.lives_hearts();
        let info_line = Line::from(vec![
            Span::styled(format!("Score: {}  ", self.score), Style::default().fg(Color::Green)),
            Span::styled(format!("Streak: {}  ", self.streak), Style::default().fg(Color::Cyan)),
            Span::styled(format!("Rounds: {}  ", self.rounds), Style::default().fg(Color::Magenta)),
            Span::styled(format!("Lives: {} ({}/{})  ", hearts, self.lives, MAX_LIVES), Style::default().fg(Color::Red)),
            Span::styled(format!("Bits: {}", self.bits.to_int()), Style::default().fg(Color::Yellow)),
        ]);
        Paragraph::new(info_line.clone())
            .alignment(Center)
            .render(center(scoreboard_area, Constraint::Length(info_line.width() as u16)), buf);

        if self.game_over {
            // Render a game over screen instead of puzzle
            let block = Block::bordered()
                .title("Game Over")
                .title_alignment(Center)
                .border_type(Double)
                .title_style(Style::default().fg(Color::Red));
            block.render(puzzle_area, buf);
            let lines = vec![
                Line::from(Span::styled(format!("Final Score: {}", self.score), Style::default().fg(Color::Green))),
                Line::from(Span::styled(format!("Rounds Played: {}", self.rounds), Style::default().fg(Color::Magenta))),
                Line::from(Span::styled("Press Enter to restart or Esc to exit", Style::default().fg(Color::Yellow))),
            ];
            Paragraph::new(lines)
                .alignment(Center)
                .render(center(puzzle_area, Constraint::Length(40)), buf);
            return;
        }

        // Render puzzle in remaining area when not game over
        self.puzzle.render_ref(puzzle_area, buf);
    }
}

impl WidgetRef for BinaryNumbersPuzzle {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [middle] = Layout::horizontal([Constraint::Length(65)]).flex(Flex::Center).areas(area);

        let [current_number_area, suggestions_area, progress_bar_area, result_area] =
            Layout::vertical([
                Constraint::Length(5), // Current number area
                Constraint::Length(3), // Suggestion area
                Constraint::Length(5), // Progress Bar / Result area (increased from 3 for gauge space)
                Constraint::Length(5), // Result / instructions area
            ])
            .flex(Flex::Center)
            .horizontal_margin(1)
            .areas(middle);

        let binary_string = self.current_to_binary_string();
        let suggestions = self.suggestions();

        // draw current number
        let [inner] = Layout::horizontal([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(current_number_area);

        Block::bordered()
            .border_type(Double)
            .border_style(Style::default().dark_gray())
            .render(inner, buf);

        // Show binary string + optional hint
        let mut lines: Vec<Line> = vec![Line::raw(binary_string.clone())];
        if self.show_hint {
            lines.push(Line::from(vec![Span::styled(
                format!("= {}", self.current_number),
                Style::default().fg(Color::DarkGray),
            )]));
        }
        let para = Paragraph::new(lines).alignment(Center);
        let centered = center(inner, Constraint::Length(binary_string.len() as u16));
        // replaced width() -> len() since binary string is ASCII (0/1 + spaces)
        para.render(centered, buf);

        // create sub layout for suggestions
        let suggestions_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(6); suggestions.len()])
            .split(suggestions_area);

        for (i, suggestion) in suggestions.iter().enumerate() {
            let item_is_selected = self.selected_suggestion == Some(*suggestion);
            let show_correct_number = self.guess_result.is_some();
            let is_correct_number = self.is_correct_guess(*suggestion);
            let area = suggestions_layout[i];

            let border_type = if item_is_selected { BorderType::Double } else { BorderType::Plain };

            let border_color = if item_is_selected {
                match self.guess_result {
                    Some(GuessResult::Correct) => Color::Green,
                    Some(GuessResult::Incorrect) => Color::Red,
                    Some(GuessResult::Timeout) => Color::Yellow,
                    None => Color::LightCyan,
                }
            } else {
                Color::DarkGray
            };

            Block::bordered().border_type(border_type).fg(border_color).render(area, buf);

            let suggestion_str = format!("{suggestion}");
            let centered = center(area, Constraint::Length(suggestion_str.len() as u16));
            Paragraph::new(format!("{}", suggestion_str))
                .white()
                .when(show_correct_number && is_correct_number, |p| p.light_green().underlined())
                .alignment(Center)
                .render(centered, buf);
        }

        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(progress_bar_area);

        Block::bordered()
            .dark_gray()
            .title("Status")
            .title_alignment(Center)
            .title_style(Style::default().white())
            .render(left, buf);

        // display the result if available
        if let Some(result) = &self.guess_result {
            let result_text = match result {
                GuessResult::Correct => ":) Correct Guess!",
                GuessResult::Incorrect => ":( Incorrect Guess!",
                GuessResult::Timeout => ":( Time's Up!",
            };

            let color = match result {
                GuessResult::Correct => Color::Green,
                GuessResult::Incorrect => Color::Red,
                GuessResult::Timeout => Color::Yellow,
            };

            let text = vec![Line::from(result_text.fg(color))];

            Paragraph::new(text)
                .alignment(Center)
                .style(Style::default().fg(color))
                .render(center(left, Constraint::Length(20)), buf);
        }

        // Dynamic gauge color based on remaining ratio
        let ratio = self.time_left / self.time_total;
        let gauge_color = if ratio > 0.6 {
            Color::Green
        } else if ratio > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };

        // Replace previous split layout: keep everything inside a single bordered block and remove percent label
        let time_block = Block::bordered()
            .dark_gray()
            .title("Time Remaining")
            .title_style(Style::default().white())
            .title_alignment(Center);
        let inner_time = time_block.inner(right);
        time_block.render(right, buf);

        // Vertical layout inside the time block interior: gauge line + text line
        let [gauge_line, time_line] = Layout::vertical([
            Constraint::Length(1), // gauge occupies one row
            Constraint::Length(1), // time text occupies one row
        ])
        .areas(inner_time);

        Gauge::default()
            .gauge_style(Style::default().fg(gauge_color))
            .ratio(ratio)
            .label("") // empty label to suppress default percent display
            .render(gauge_line, buf);

        Paragraph::new(Line::from(Span::styled(
            format!("{:.2} seconds left", self.time_left),
            Style::default().fg(gauge_color),
        )))
        .alignment(Center)
        .render(time_line, buf);

        Block::bordered().dark_gray().render(result_area, buf);

        let mut instruction_spans: Vec<Span> = vec![
            Span::styled("<", Style::default().fg(Color::White)),
            Span::styled("Left/Right", Style::default().fg(Color::LightCyan)),
            Span::styled("> select  ", Style::default().fg(Color::White)),
            Span::styled("<", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::LightCyan)),
            Span::styled("> confirm  ", Style::default().fg(Color::White)),
            Span::styled("<", Style::default().fg(Color::White)),
            Span::styled("H", Style::default().fg(Color::LightCyan)),
            Span::styled("> hint  ", Style::default().fg(Color::White)),
            Span::styled("<", Style::default().fg(Color::White)),
            Span::styled("S", Style::default().fg(Color::LightCyan)),
            Span::styled("> skip  ", Style::default().fg(Color::White)),
            Span::styled("<", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::LightCyan)),
            Span::styled("> exit", Style::default().fg(Color::White)),
        ];

        if self.guess_result.is_some() {
            instruction_spans.extend(vec![
                Span::styled("  <", Style::default().fg(Color::White)),
                Span::styled("Enter", Style::default().fg(Color::LightCyan)),
                Span::styled("> play again", Style::default().fg(Color::White)),
            ]);
        }

        let text = vec![Line::from(instruction_spans)];
        Paragraph::new(text)
            .alignment(Center)
            .render(center(result_area, Constraint::Length(65)), buf);
    }
}

pub struct BinaryNumbersGame {
    puzzle: BinaryNumbersPuzzle,
    bits: Bits,
    exit_intended: bool,
    score: u32,
    streak: u32,
    rounds: u32,
    puzzle_resolved: bool, // prevents double finalization
    lives: u32,            // NEW: lives remaining
    game_over: bool,       // NEW: game over state
}

impl MainScreenWidget for BinaryNumbersGame {
    fn run(&mut self, dt: f64) {
        if self.game_over { return; }
        self.puzzle.run(dt);
        if self.puzzle.guess_result.is_some() && !self.puzzle_resolved {
            self.finalize_round();
        }
    }

    fn handle_input(&mut self, input: KeyEvent) -> () { self.handle_game_input(input); }
    fn is_exit_intended(&self) -> bool { self.exit_intended }
}

impl BinaryNumbersGame {
    pub fn new(bits: Bits) -> Self {
        Self {
            bits: bits.clone(),
            puzzle: Self::init_puzzle(bits.clone(), 0),
            exit_intended: false,
            score: 0,
            streak: 0,
            rounds: 0,
            puzzle_resolved: false,
            lives: 3, // start with 3 lives
            game_over: false,
        }
    }

    pub fn init_puzzle(bits: Bits, streak: u32) -> BinaryNumbersPuzzle {
        BinaryNumbersPuzzle::new(bits, streak)
    }
}

impl BinaryNumbersGame {
    pub fn lives_hearts(&self) -> String {
        let full = "♥".repeat(self.lives as usize);
        let empty = "·".repeat((MAX_LIVES - self.lives) as usize);
        format!("{}{}", full, empty)
    }

    fn finalize_round(&mut self) {
        if let Some(result) = self.puzzle.guess_result {
            self.rounds += 1;
            match result {
                GuessResult::Correct => {
                    self.streak += 1;
                    self.score += 10 + (self.streak * 2);
                    // Award extra life every 5 streaks (up to MAX_LIVES)
                    if self.streak % 5 == 0 && self.lives < MAX_LIVES {
                        self.lives += 1;
                    }
                }
                GuessResult::Incorrect | GuessResult::Timeout => {
                    self.streak = 0;
                    if self.lives > 0 { self.lives -= 1; }
                }
            }
            if self.lives == 0 { self.game_over = true; }
            self.puzzle_resolved = true;
        }
    }

    pub fn handle_game_input(&mut self, input: KeyEvent) {
        if input.code == KeyCode::Esc { self.exit_intended = true; return; };
        if self.game_over { self.handle_game_over_input(input); return; }
        match self.puzzle.guess_result {
            None => self.handle_no_result_yet(input),
            Some(_) => self.handle_result_available(input),
        }
    }

    fn handle_game_over_input(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Enter => { self.reset_game_state(); }
            KeyCode::Esc => { self.exit_intended = true; }
            _ => {}
        }
    }

    fn reset_game_state(&mut self) {
        self.score = 0;
        self.streak = 0;
        self.rounds = 0;
        self.lives = 3;
        self.game_over = false;
        self.puzzle_resolved = false;
        self.puzzle = Self::init_puzzle(self.bits.clone(), 0);
    }

    fn handle_no_result_yet(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Right => {
                // select the next suggestion
                if let Some(selected) = self.puzzle.selected_suggestion {
                    let current_index = self.puzzle.suggestions.iter().position(|&x| x == selected);
                    if let Some(index) = current_index {
                        let next_index = (index + 1) % self.puzzle.suggestions.len();
                        self.puzzle.selected_suggestion = Some(self.puzzle.suggestions[next_index]);
                    }
                } else {
                    // if no suggestion is selected, select the first one
                    self.puzzle.selected_suggestion = Some(self.puzzle.suggestions[0]);
                }
            }
            KeyCode::Left => {
                // select the previous suggestion
                if let Some(selected) = self.puzzle.selected_suggestion {
                    let current_index = self.puzzle.suggestions.iter().position(|&x| x == selected);
                    if let Some(index) = current_index {
                        let prev_index = if index == 0 {
                            self.puzzle.suggestions.len() - 1
                        } else {
                            index - 1
                        };
                        self.puzzle.selected_suggestion = Some(self.puzzle.suggestions[prev_index]);
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = self.puzzle.selected_suggestion {
                    if self.puzzle.is_correct_guess(selected) {
                        self.puzzle.guess_result = Some(GuessResult::Correct);
                    } else {
                        self.puzzle.guess_result = Some(GuessResult::Incorrect);
                    }
                    self.finalize_round();
                }
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.puzzle.show_hint = !self.puzzle.show_hint;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Skip puzzle counts as timeout
                self.puzzle.guess_result = Some(GuessResult::Timeout);
                self.finalize_round();
            }
            _ => {}
        }
    }

    fn handle_result_available(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Enter => {
                // Start a new puzzle, difficulty scaling with current streak
                self.puzzle = Self::init_puzzle(self.bits.clone(), self.streak);
                self.puzzle_resolved = false;
            }
            KeyCode::Esc => self.exit_intended = true,
            KeyCode::Char('h') | KeyCode::Char('H') => {
                // Allow hint toggle even after result
                self.puzzle.show_hint = !self.puzzle.show_hint;
            }
            _ => {}
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum GuessResult {
    Correct,
    Incorrect,
    Timeout,
}

#[derive(Clone)]
pub enum Bits { Four, Eight, Twelve, Sixteen, }

impl Bits {
    pub fn to_int(&self) -> u32 {
        match self {
            Bits::Four => 4,
            Bits::Eight => 8,
            Bits::Twelve => 12,
            Bits::Sixteen => 16,
        }
    }

    pub fn upper_bound(&self) -> u32 {
        u32::pow(2, self.to_int()) - 1
    }

    pub fn suggestion_count(&self) -> usize {
        match self {
            Bits::Four => 3,
            Bits::Eight => 4,
            Bits::Twelve => 5,
            Bits::Sixteen => 6,
        }
    }
}

pub struct BinaryNumbersPuzzle {
    bits: Bits,
    current_number: u32,
    suggestions: Vec<u32>,
    selected_suggestion: Option<u32>,
    time_total: f64,
    time_left: f64,
    guess_result: Option<GuessResult>,
    show_hint: bool,
}

impl BinaryNumbersPuzzle {
    pub fn new(bits: Bits, streak: u32) -> Self {
        let mut rng = rand::rng();

        let mut suggestions = Vec::new();
        while suggestions.len() < bits.suggestion_count() {
            let num = rng.random_range(0..=bits.upper_bound());
            if !suggestions.contains(&num) {
                suggestions.push(num);
            }
        }

        // Choose current number from suggestions to ensure it's present
        let current_index = rng.random_range(0..suggestions.len());
        let current_number = suggestions[current_index];
        suggestions.shuffle(&mut rng);

        // Base time by bits + difficulty scaling (shorter as streak increases)
        let base_time = match bits {
            Bits::Four => 8.0,
            Bits::Eight => 12.0,
            Bits::Twelve => 16.0,
            Bits::Sixteen => 20.0,
        };
        let penalty = (streak as f64) * 0.5; // 0.5s less per streak
        let time_total = (base_time - penalty).max(5.0);
        let time_left = time_total;
        let selected_suggestion = Some(suggestions[0]);
        let guess_result = None;
        let show_hint = false;

        Self {
            bits,
            current_number,
            suggestions,
            time_total,
            time_left,
            selected_suggestion,
            guess_result,
            show_hint,
        }
    }

    pub fn suggestions(&self) -> &[u32] { &self.suggestions }
    pub fn is_correct_guess(&self, guess: u32) -> bool { guess == self.current_number }

    pub fn current_to_binary_string(&self) -> String {
        let width = self.bits.to_int() as usize;
        let raw = format!("{:0width$b}", self.current_number, width = width);
        raw.chars()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn run(&mut self, dt: f64) {
        if self.guess_result.is_some() {
            // If a guess has been made, we don't need to run the game logic anymore.
            return;
        }

        self.time_left = (self.time_left - dt).max(0.0);

        if self.time_left <= 0.0 {
            self.guess_result = Some(GuessResult::Timeout);
        }
    }
}