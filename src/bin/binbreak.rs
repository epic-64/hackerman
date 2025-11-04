use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use hackerman::games::binary_numbers::{BinaryNumbersGame, Bits};
use hackerman::games::main_screen_widget::MainScreenWidget;
use std::time::Instant;
use std::{cmp, env, thread};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let bits = parse_bits_arg();
    let mut game = BinaryNumbersGame::new(bits);
    let mut terminal = ratatui::init();
    game_loop(&mut game, &mut terminal)?;
    ratatui::restore();
    Ok(())
}

fn parse_bits_arg() -> Bits {
    let arg = env::args().nth(1).unwrap_or_else(|| "8".to_string());
    match arg.as_str() {
        "4" => Bits::Four,
        "8" => Bits::Eight,
        "12" => Bits::Twelve,
        "16" => Bits::Sixteen,
        _ => Bits::Eight,
    }
}

fn game_loop(
    game: &mut BinaryNumbersGame,
    terminal: &mut ratatui::DefaultTerminal,
) -> color_eyre::Result<()> {
    let mut last_frame_time = Instant::now();
    let target_frame_duration = std::time::Duration::from_millis(33); // ~30 FPS

    while !game.is_exit_intended() {
        let now = Instant::now();
        let dt = now - last_frame_time;
        last_frame_time = now;

        // draw frame
        terminal.draw(|f| f.render_widget(&mut *game, f.area()))?;

        // advance game state
        game.run(dt.as_secs_f64());

        // handle input (non-blocking with poll)
        let poll_timeout = std::cmp::min(dt, target_frame_duration);
        if event::poll(poll_timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(game, key);
                }
            }
        }

        // cap frame rate
        let frame_duration = last_frame_time.elapsed();
        if frame_duration < target_frame_duration {
            thread::sleep(target_frame_duration - frame_duration);
        }
    }
    Ok(())
}

fn handle_key(game: &mut BinaryNumbersGame, key: KeyEvent) {
    match key.code {
        KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers == KeyModifiers::CONTROL => {
            game.handle_game_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        }
        _ => game.handle_game_input(key),
    }
}
