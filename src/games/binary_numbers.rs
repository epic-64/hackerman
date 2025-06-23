use crossterm::event::{KeyCode, KeyEvent};
use rand::prelude::SliceRandom;
use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::{Alignment, Color, Style, Stylize, Widget};
use ratatui::text::ToSpan;
use ratatui::widgets::{Block, Gauge, Paragraph};
use crate::games::game_widget::{WidgetGame, WidgetRef};

impl WidgetRef for BinaryNumbersGame {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.puzzle.render_ref(area, buf);
    }
}

impl WidgetRef for BinaryNumbersPuzzle {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [title_area, current_number_area, suggestions_area, progress_bar_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(3)
            ])
            .margin(1)
            .areas(area);

        Paragraph::new("Binary Numbers Puzzle")
            .alignment(Alignment::Center)
            .render(title_area, buf);

        let binary_string = self.current_to_binary_string();
        let suggestions = self.suggestions();

        Paragraph::new(format!("\n{}", binary_string))
            .block(Block::bordered().title("Binary Number").title_alignment(Alignment::Center))
            .alignment(Alignment::Center)
            .render(current_number_area.inner(Margin::new(12, 0)), buf);

        // create sub layout for suggestions
        let suggestions_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(6); suggestions.len()])
            .split(suggestions_area);

        for (i, suggestion) in suggestions.iter().enumerate() {
            let background_color = if self.selected_suggestion == Some(*suggestion) {
                Style::default().bg(Color::Yellow)
            } else {
                Style::default()
            };

            Paragraph::new(format!("{}", suggestion))
                .block(Block::bordered())
                .style(background_color)
                .alignment(Alignment::Center)
                .render(suggestions_layout[i], buf);
        }

        // render a progress bar
        Gauge::default()
            .gauge_style(Style::new().white().on_black().italic())
            .ratio(self.frames_left as f64 / self.frames_total as f64)
            .label(format!("Time left: {} seconds", self.frames_left / 60).to_span().style(Style::default().fg(Color::Yellow)))
            .render(progress_bar_area, buf);

        // render the guess result if available
        if let Some(result) = &self.guess_result {
            let result_text = match result {
                GuessResult::Correct => "Correct Guess!",
                GuessResult::Incorrect => "Incorrect Guess!",
                GuessResult::Timeout => "Time's Up!",
            };

            Paragraph::new(result_text)
                .block(Block::bordered().title("Guess Result").title_alignment(Alignment::Center))
                .alignment(Alignment::Center)
                .render(area.inner(Margin { horizontal: 0, vertical: 1 }), buf);
        }
    }
}

pub struct BinaryNumbersGame {
    puzzle: BinaryNumbersPuzzle,
    exit_intended: bool,
}

impl WidgetGame for BinaryNumbersGame {
    fn run(&mut self) {
        self.puzzle.run();
    }

    fn handle_input(&mut self, input: KeyEvent) -> () {
        self.handle_game_input(input);
    }

    fn is_exit_intended(&self) -> bool {
        self.exit_intended
    }
}

impl BinaryNumbersGame {
    pub fn new() -> Self {
        Self { puzzle: BinaryNumbersPuzzle::new(), exit_intended: false }
    }
}

impl BinaryNumbersGame {
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
                } else {
                    // if no suggestion is selected, select the first one
                    self.puzzle.selected_suggestion = Some(self.puzzle.suggestions[0]);
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
            KeyCode::Enter => self.puzzle = BinaryNumbersPuzzle::new(),
            KeyCode::Esc => self.exit_intended = true,
            _ => {}
        }
    }

    pub fn handle_game_input(&mut self, input: KeyEvent) {
        match self.puzzle.guess_result {
            None => self.handle_no_result_yet(input),
            Some(_) => self.handle_result_available(input),
        }
    }
}

enum GuessResult {
    Correct,
    Incorrect,
    Timeout,
}

pub struct BinaryNumbersPuzzle {
    current_number: u32,
    suggestions: Vec<u32>,
    selected_suggestion: Option<u32>,
    frames_total: u32,
    frames_left: u32,
    guess_result: Option<GuessResult>,
}

impl BinaryNumbersPuzzle {
    pub fn new() -> Self {
        let mut rng = rand::rng();

        let mut suggestions = Vec::new();
        while suggestions.len() < 4 {
            let num = rng.random_range(0..=255);
            if !suggestions.contains(&num) {
                suggestions.push(num);
            }
        }

        let current_number = suggestions[0];
        suggestions.shuffle(&mut rng);

        let seconds_to_guess = 5;
        let frames_total = seconds_to_guess * 60; // assuming 60 frames per second
        let frames_left = frames_total;
        let selected_suggestion = None;
        let guess_result = None;

        Self {
            current_number,
            suggestions,
            frames_left,
            frames_total,
            selected_suggestion,
            guess_result
        }
    }

    pub fn current_number(&self) -> u32 {
        self.current_number
    }

    pub fn suggestions(&self) -> &[u32] {
        &self.suggestions
    }

    pub fn is_correct_guess(&self, guess: u32) -> bool {
        guess == self.current_number
    }

    pub fn current_to_binary_string(&self) -> String {
        format!("{:08b}", self.current_number)
    }

    pub fn run(&mut self) {
        if self.guess_result.is_some() {
            // If a guess has been made, we don't need to run the game logic anymore.
            return;
        }

        self.frames_left = self.frames_left.saturating_sub(1);

        if self.frames_left == 0 {
            self.guess_result = Some(GuessResult::Timeout);
        }
    }
}