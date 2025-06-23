use crossterm::event::{KeyCode, KeyModifiers};
use strum_macros::{Display, EnumIter};
use binary_numbers::BinaryNumbersGame;
use crate::App;
use crate::games::binary_numbers;
use crate::games::game_widget::WidgetGame;

#[derive(EnumIter, Display, Clone, PartialEq)]
pub enum GameName {
    Home,
    BinaryNumbers,
    DinoJump,
}

impl GameName {
    pub fn get_widget_game(&self) -> Option<Box<dyn WidgetGame>> {
        match self {
            GameName::BinaryNumbers => Some(Box::new(BinaryNumbersGame::new())),
            _ => None,
        }
    }
}

#[derive(Display, Clone, PartialEq)]
pub enum InputMode {
    GameSelection,
    Game(GameName),
}

pub fn handle_input(app: &mut App, input: crossterm::event::KeyEvent) -> color_eyre::Result<()> {
    match (input.modifiers, input.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => app.quit(),
        _ => {}
    }

    if input.code == KeyCode::Char(' ') {
        app.refresh_without_inputs = !app.refresh_without_inputs;
    }

    match &mut app.current_game {
        None => handle_game_selection_input(app, input),
        Some(game) => game.handle_input(input),
    }
    Ok(())
}

fn handle_game_selection_input(app: &mut App, input: crossterm::event::KeyEvent) {
    match input.code {
        KeyCode::Up => app.games_state.select_previous(),
        KeyCode::Down => app.games_state.select_next(),
        KeyCode::Enter => {
            let selected_game_index = app.games_state.list_state.selected().unwrap_or(0);
            let selected_game = app.games_state.games.get(selected_game_index).cloned().unwrap_or(GameName::Home);

            if let Some(widget_game) = selected_game.get_widget_game() {
                app.current_game = Some(widget_game);
            } else {
                app.current_game = None;
            }

            app.input_mode = InputMode::Game(
                app.games_state
                    .games
                    .get(app.games_state.list_state.selected().unwrap_or(0))
                    .cloned()
                    .unwrap_or(GameName::Home),
            );
        }
        _ => {}
    }
}