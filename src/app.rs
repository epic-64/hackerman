use crate::games::binary_numbers;
use crate::games::game_widget::MainScreenWidget;
use crate::{App, MainMenu};
use binary_numbers::BinaryNumbersGame;
use crossterm::event::{KeyCode, KeyModifiers};
use strum_macros::{Display, EnumIter};
use crate::utils::{speak, TrimMargin};

#[derive(EnumIter, Display, Clone, PartialEq)]
pub enum MainMenuEntry {
    Home,
    BinaryNumbers,
    DinoJump,
}

impl MainMenuEntry {
    pub fn get_main_screen_widget(&self) -> Option<Box<dyn MainScreenWidget>> {
        match self {
            MainMenuEntry::BinaryNumbers => Some(Box::new(BinaryNumbersGame::new())),
            _ => None,
        }
    }
}

#[derive(Display, Clone, PartialEq)]
pub enum InputMode {
    GameSelection,
    Game(MainMenuEntry),
}

pub fn handle_input(app: &mut App, input: crossterm::event::KeyEvent) -> color_eyre::Result<()> {
    if input.modifiers == KeyModifiers::CONTROL && matches!(input.code, KeyCode::Char('c') | KeyCode::Char('C')) {
        app.quit();
    }
    
    if input.code == KeyCode::F(1) {
        speak("
            Hackerman. This is a terminal based environment with minigames.
            There is a main menu that you can navigate using the up and down arrow keys.
            Press Enter to select a game.
            Press Escape to return to the main menu.
            Press F2 to get an overview of where you currently are.
            Press F1 to read this help message again.
        ".nice());
    }

    if input.code == KeyCode::F(2) {
        match app.current_main_widget {
            None => {
                speak("You are in the main menu.".into());
                speak(format!(
                    "highlighted item: {}",
                    app.main_menu.get_selected_entry()
                        .map_or("No game selected".into(), |entry| entry.to_string()))
                );
            },
            Some(ref game) => speak(format!("{}", game.get_overview())),
        }
    }

    if input.code == KeyCode::Char(' ') {
        app.refresh_without_inputs = !app.refresh_without_inputs;
    }

    match &mut app.current_main_widget {
        None => handle_game_selection_input(app, input),
        Some(game) => game.handle_input(input),
    }
    Ok(())
}

fn handle_game_selection_input(app: &mut App, input: crossterm::event::KeyEvent) -> () {
    match input.code {
        KeyCode::Up => {
            app.main_menu.select_previous();
            speak(app.main_menu.get_selected_entry().map_or("No game selected".into(), |entry| entry.to_string()));
        },
        KeyCode::Down => {
            app.main_menu.select_next();
            speak(app.main_menu.get_selected_entry().map_or("No game selected".into(), |entry| entry.to_string()));
        }
        KeyCode::Enter => {
            app.current_main_widget = match app.main_menu.get_selected_entry() {
                Some(entry) => entry.get_main_screen_widget(),
                None => None,
            }
        }
        _ => {}
    }
}