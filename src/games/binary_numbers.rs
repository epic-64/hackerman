use color_eyre::owo_colors::OwoColorize;
use crate::games::main_screen_widget::{MainScreenWidget, WidgetRef};
use crossterm::event::{KeyCode, KeyEvent};
use rand::prelude::SliceRandom;
use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::{Alignment, Color, Line, Style, Stylize, Widget};
use ratatui::prelude::Alignment::Center;
use ratatui::text::{Span, ToSpan};
use ratatui::widgets::{Block, Gauge, LineGauge, Paragraph};

impl WidgetRef for BinaryNumbersGame {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.puzzle.render_ref(area, buf);
    }
}

impl WidgetRef for BinaryNumbersPuzzle {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [_left, middle, _right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(0), Constraint::Length(65), Constraint::Fill(0)])
            .areas(area);

        let [_s1, current_number_area, suggestions_area, _s2, progress_bar_area, result_area, _s3] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(0), // Spacer
                Constraint::Length(5), // Current number area
                Constraint::Length(3), // Suggestions area
                Constraint::Length(2), // Spacer
                Constraint::Length(3), // Progress bar area
                Constraint::Length(6), // Result area
                Constraint::Fill(0), // Spacer
            ])
            .horizontal_margin(1)
            .areas(middle);

        let binary_string = self.current_to_binary_string();
        let suggestions = self.suggestions();

        Paragraph::new(format!("\n{}", binary_string))
            .block(Block::bordered())
            .alignment(Center)
            .render(current_number_area, buf);

        // create sub layout for suggestions
        let suggestions_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(6); suggestions.len()])
            .split(suggestions_area);

        for (i, suggestion) in suggestions.iter().enumerate() {
            let background_color = if self.selected_suggestion == Some(*suggestion) {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let show_correct_number = self.guess_result.is_some();
            let is_correct_number = self.is_correct_guess(*suggestion);

            let foreground_color = if show_correct_number && is_correct_number {
                Color::LightGreen
            } else {
                Color::White
            };

            Paragraph::new(format!("{}", suggestion))
                .block(Block::bordered())
                .style(background_color)
                .fg(foreground_color)
                .alignment(Center)
                .render(suggestions_layout[i], buf);
        }

        // render a progress bar
        Gauge::default()
            .style(Style::reset().fg(Color::White))
            .gauge_style(Style::default().green().on_dark_gray())
            .ratio(self.time_left / self.time_total)
            .label(format!("{:.2} seconds", self.time_left)
                .to_span().style(Style::default().white()))
            .block(Block::bordered().title("Time Remaining").title_alignment(Center))
            .render(progress_bar_area, buf);

        Block::bordered().title("Result").title_alignment(Center).render(result_area, buf);

        // render the guess result if available
        if let Some(result) = &self.guess_result {
            let result_text = match result {
                GuessResult::Correct => "Correct Guess!",
                GuessResult::Incorrect => "Incorrect Guess!",
                GuessResult::Timeout => "Time's Up!",
            };

            let color = match result {
                GuessResult::Correct => Color::Green,
                GuessResult::Incorrect => Color::Red,
                GuessResult::Timeout => Color::Yellow,
            };

            let text = vec![
                Line::from(result_text.fg(color)),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::White)),
                    Span::styled("Enter", Style::default().fg(Color::LightCyan)),
                    Span::styled(" to play again or ", Style::default().fg(Color::White)),
                    Span::styled("Esc", Style::default().fg(Color::LightCyan)),
                    Span::styled(" to exit.", Style::default().fg(Color::White)),
                ]),
            ];

            Paragraph::new(text)
                .alignment(Center)
                .style(Style::default().fg(color))
                .render(result_area.inner(Margin::new(1, 2)), buf);
        }
    }
}

pub struct BinaryNumbersGame {
    puzzle: BinaryNumbersPuzzle,
    bits: Bits,
    exit_intended: bool,
}

impl MainScreenWidget for BinaryNumbersGame {
    fn run(&mut self, dt: f64) {
        self.puzzle.run(dt);
    }

    fn handle_input(&mut self, input: KeyEvent) -> () {
        self.handle_game_input(input);
    }

    fn is_exit_intended(&self) -> bool {
        self.exit_intended
    }
}

impl BinaryNumbersGame {
    pub fn new(bits: Bits) -> Self {
        Self {
            bits: bits.clone(),
            puzzle: Self::init_puzzle(bits),
            exit_intended: false
        }
    }

    pub fn init_puzzle(bits: Bits) -> BinaryNumbersPuzzle {
        BinaryNumbersPuzzle::new(bits)
    }
}

impl BinaryNumbersGame {
    pub fn handle_game_input(&mut self, input: KeyEvent) {
        if input.code == KeyCode::Esc {
            self.exit_intended = true
        };
        
        match self.puzzle.guess_result {
            None => self.handle_no_result_yet(input),
            Some(_) => self.handle_result_available(input),
        }
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
                }
            }
            _ => {
                // ignore other inputs
            }
        }
    }

    fn handle_result_available(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Enter => self.puzzle = Self::init_puzzle(self.bits.clone()),
            KeyCode::Esc => self.exit_intended = true,
            _ => {}
        }
    }
}

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
}

impl BinaryNumbersPuzzle {
    pub fn new(bits: Bits) -> Self {
        let mut rng = rand::rng();

        let mut suggestions = Vec::new();
        while suggestions.len() < bits.suggestion_count() {
            let num = rng.random_range(0..=bits.upper_bound());
            if !suggestions.contains(&num) {
                suggestions.push(num);
            }
        }

        let current_number = suggestions[0];
        suggestions.shuffle(&mut rng);

        let time_total = 5.0;
        let time_left = time_total;
        let selected_suggestion = Some(suggestions[0]);
        let guess_result = None;

        Self {
            bits,
            current_number,
            suggestions,
            time_total,
            time_left,
            selected_suggestion,
            guess_result
        }
    }

    pub fn suggestions(&self) -> &[u32] {
        &self.suggestions
    }

    pub fn is_correct_guess(&self, guess: u32) -> bool {
        guess == self.current_number
    }

    pub fn current_to_binary_string(&self) -> String {
        // let binary_string = format!(
        //     "{:0width$b}",
        //     self.current_number,
        //     width = self.bits.to_int() as usize
        // );
        //
        // binary_string.chars()
        //     .collect::<Vec<_>>()
        //     .chunks(4)
        //     .map(|chunk| chunk.iter().collect::<String>())
        //     .collect::<Vec<_>>()
        //     .join(" ")

        format!("{:08b}", self.current_number)
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