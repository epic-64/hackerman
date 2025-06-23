use crossterm::event::{KeyCode, KeyEvent};
use rand::prelude::SliceRandom;
use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Alignment, Color, Style, Widget};
use ratatui::widgets::{Block, Paragraph};
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
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3)
            ])
            .areas(area);
        
        // render the title
        Paragraph::new("Binary Numbers Puzzle")
            .block(Block::bordered().title("Game Title").title_alignment(Alignment::Center))
            .alignment(Alignment::Center)
            .render(title_area, buf);

        let binary_string = self.current_to_binary_string();
        let suggestions = self.suggestions();

        Paragraph::new(format!("{}", binary_string))
            .block(Block::bordered().title("Binary Number").title_alignment(Alignment::Center))
            .alignment(Alignment::Center)
            .render(current_number_area, buf);

        // create sub layout for suggestions
        let suggestions_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(6); suggestions.len()])
            .split(suggestions_area);

        for (i, suggestion) in suggestions.iter().enumerate() {
            let background_color = if self.selected_suggestion == Some(*suggestion) {
                Style::default().bg(Color::Yellow)
            } else {
                Style::default()
            };

            Paragraph::new(format!("{:08b}", suggestion))
                .block(Block::bordered().title(format!("Suggestion {}", i + 1)).title_alignment(Alignment::Center))
                .style(background_color)
                .alignment(Alignment::Center)
                .render(suggestions_layout[i], buf);
        }

        // render a progress bar
        let progress_bar = Paragraph::new(format!("Frames left: {}", self.frames_left))
            .block(Block::bordered().title("Progress").title_alignment(Alignment::Center))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Green));
        progress_bar.render(progress_bar_area, buf);
    }
}

pub struct BinaryNumbersGame {
    puzzle: BinaryNumbersPuzzle,
}

impl WidgetGame for BinaryNumbersGame {
    fn run(&mut self) { self.puzzle.run(); }

    fn handle_input(&mut self, input: KeyEvent) -> () {
        self.handle_input(input);
    }
}

impl BinaryNumbersGame {
    pub fn new() -> Self {
        Self { puzzle: BinaryNumbersPuzzle::new() }
    }
}

impl BinaryNumbersGame {
    pub fn handle_input(&mut self, input: KeyEvent) {
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
    }}

enum GuessResult {
    Correct,
    Incorrect,
    Timeout,
}

pub struct BinaryNumbersPuzzle {
    current_number: u32,
    suggestions: Vec<u32>,
    selected_suggestion: Option<u32>,
    frames_left: u32,
    guess_result: Option<GuessResult>,
}

impl BinaryNumbersPuzzle {
    pub fn new() -> Self {
        let mut rng = rand::rng();

        // get 4 different random numbers between 0 and 255
        let mut suggestions = Vec::new();
        while suggestions.len() < 4 {
            let num = rng.random_range(0..=255);
            if !suggestions.contains(&num) {
                suggestions.push(num);
            }
        }

        // use the first number as the current number
        let current_number = suggestions[0];

        // shuffle the suggestions
        suggestions.shuffle(&mut rng);

        let seconds_to_guess = 60;
        let frames_left = seconds_to_guess * 60; // assuming 60 frames per second
        let selected_suggestion = None;
        let guess_result = None;

        Self { current_number, suggestions, frames_left, selected_suggestion, guess_result }
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
        self.frames_left = self.frames_left.saturating_sub(1);

        if self.frames_left == 0 {
            self.guess_result = Some(GuessResult::Timeout);
        } else if let Some(selected) = self.selected_suggestion {
            if self.is_correct_guess(selected) {
                self.guess_result = Some(GuessResult::Correct);
            } else {
                self.guess_result = Some(GuessResult::Incorrect);
            }
        }
    }
}