use crossterm::event::{KeyCode, KeyModifiers};
use strum_macros::{Display, EnumIter};
use crate::App;

#[derive(EnumIter, Display, Clone, PartialEq)]
pub enum Game {
    Home,
    Counter,
    DinoJump,
}

#[derive(Display, Clone)]
pub enum InputMode {
    GameSelection,
    Game(Game),
}

pub fn handle_input(app: &mut App, input: crossterm::event::KeyEvent) -> color_eyre::Result<()> {
    match (input.modifiers, input.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => app.quit(),
        _ => {}
    }

    match app.input_mode.clone() {
        InputMode::GameSelection => match input.code {
            KeyCode::Up => app.games_state.select_previous(),
            KeyCode::Down => app.games_state.select_next(),
            KeyCode::Enter => {
                app.input_mode = InputMode::Game(
                    app.games_state
                        .games
                        .get(app.games_state.list_state.selected().unwrap_or(0))
                        .cloned()
                        .unwrap_or(Game::Home),
                );
            }
            _ => {}
        },
        InputMode::Game(game) => {
            if input.code == KeyCode::Esc {
                app.input_mode = InputMode::GameSelection;
            }

            match game {
                Game::Home => {
                    // Handle Home game inputs
                }
                Game::Counter => {
                    // Handle Counter game inputs
                }
                Game::DinoJump => {
                    // Handle Dino Jump game inputs
                }
            }
        },
    }
    Ok(())
}