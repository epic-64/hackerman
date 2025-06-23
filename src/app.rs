use crossterm::event::{KeyCode, KeyModifiers};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use crate::App;

#[derive(EnumIter, Display, Clone)]
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

pub fn handle_input(
    app: &mut App,
    input: crossterm::event::KeyEvent,
) -> color_eyre::Result<()> {
    let number_of_games = Game::iter().count();

    match (input.modifiers, input.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => app.quit(),
        _ => {}
    }

    match app.input_mode.clone() {
        InputMode::GameSelection => match input.code {
            KeyCode::Up if app.selected_game_index > 0 => {
                app.selected_game_index -= 1;
            }
            KeyCode::Down if app.selected_game_index < number_of_games - 1 => {
                app.selected_game_index += 1;
            }
            KeyCode::Enter => {
                app.input_mode = InputMode::Game(Game::iter().nth(app.selected_game_index).unwrap());
            }
            _ => {}
        },
        InputMode::Game(game) => {
            match input.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::GameSelection;
                }
                _ => {}
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