use crate::games::binary_numbers;
use crate::games::main_screen_widget::MainScreenWidget;
use crate::App;
use binary_numbers::BinaryNumbersGame;
use crossterm::event::{KeyCode, KeyModifiers};
use strum_macros::{Display, EnumIter};

#[derive(EnumIter, Display, Clone, PartialEq)]
pub enum MainMenuEntry {
    AsciiArt,
    BinaryNumbers,
    DinoJump,
}

impl MainMenuEntry {
    pub fn get_main_screen_widget(&self) -> Option<Box<dyn MainScreenWidget>> {
        match self {
            MainMenuEntry::AsciiArt => Some(Box::new(crate::games::ascii_art::AsciiArtMain::new())),
            MainMenuEntry::BinaryNumbers => Some(Box::new(BinaryNumbersGame::new())),
            _ => None,
        }
    }
}

pub fn handle_input(app: &mut App, input: crossterm::event::KeyEvent) -> color_eyre::Result<()> {
    if input.modifiers == KeyModifiers::CONTROL && matches!(input.code, KeyCode::Char('c') | KeyCode::Char('C')) {
        app.quit();
    }

    if input.code == KeyCode::F(2) {
        match app.current_main_widget {
            None => {}
            Some(ref game) => {}
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
        },
        KeyCode::Down => {
            app.main_menu.select_next();
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