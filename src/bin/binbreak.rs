use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use hackerman::games::binary_numbers::{BinaryNumbersGame, Bits};
use hackerman::games::main_screen_widget::MainScreenWidget;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListState, Paragraph};
use std::time::Instant;
use std::{env, thread};

// Start menu state
struct StartMenuState {
    items: Vec<(String, Bits)>,
    list_state: ListState,
}

impl StartMenuState {
    fn new() -> Self {
        let items = vec![
            ("easy (4 bits)".to_string(), Bits::Four),
            ("normal (8 bits)".to_string(), Bits::Eight),
            ("master (12 bits)".to_string(), Bits::Twelve),
            ("insane (16 bits)".to_string(), Bits::Sixteen),
        ];
        Self { items, list_state: ListState::default().with_selected(Some(1)) } // default to normal
    }
    fn selected_index(&self) -> usize { self.list_state.selected().unwrap_or(0) }
    fn selected_bits(&self) -> Bits { self.items[self.selected_index()].1.clone() }
    fn select_next(&mut self) { self.list_state.select_next(); }
    fn select_previous(&mut self) { self.list_state.select_previous(); }
}

enum AppState {
    Start(StartMenuState),
    Playing(BinaryNumbersGame),
    Exit,
}

fn handle_start_input(state: &mut StartMenuState, key: KeyEvent) -> Option<AppState> {
    match key.code {
        KeyCode::Up => state.select_previous(),
        KeyCode::Down => state.select_next(),
        KeyCode::Enter => {
            let bits = state.selected_bits();
            return Some(AppState::Playing(BinaryNumbersGame::new(bits)));
        }
        KeyCode::Esc => return Some(AppState::Exit),
        KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers == KeyModifiers::CONTROL => {
            return Some(AppState::Exit)
        }
        _ => {}
    }
    None
}

fn render_start_screen(state: &mut StartMenuState, area: Rect, buf: &mut Buffer) {
    let [title_area, list_area, info_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(4),
    ]).areas(area);

    Paragraph::new("BinBreak - Select Mode")
        .alignment(Alignment::Center)
        .block(Block::bordered().title("Start"))
        .render(title_area, buf);

    let items: Vec<Line> = state.items.iter().map(|(label, _)| Line::from(label.as_str())).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Modes"))
        .highlight_style(Style::default().fg(Color::LightCyan).bold())
        .highlight_symbol("> ")
        .repeat_highlight_symbol(true);
    ratatui::widgets::StatefulWidget::render(list, list_area, buf, &mut state.list_state);

    let instructions = "Use Up/Down to select, Enter to start, Esc to exit";
    Paragraph::new(instructions)
        .alignment(Alignment::Center)
        .block(Block::bordered().title("Controls"))
        .render(info_area, buf);
}

fn run_app(terminal: &mut ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut app_state = AppState::Start(StartMenuState::new());
    let mut last_frame_time = Instant::now();
    let target_frame_duration = std::time::Duration::from_millis(33); // ~30 FPS

    while !matches!(app_state, AppState::Exit) {
        let now = Instant::now();
        let dt = now - last_frame_time;
        last_frame_time = now;

        terminal.draw(|f| {
            match &mut app_state {
                AppState::Start(menu) => render_start_screen(menu, f.area(), f.buffer_mut()),
                AppState::Playing(game) => { f.render_widget(&mut *game, f.area()); },
                AppState::Exit => {},
            }
        })?;

        // Advance game if playing
        if let AppState::Playing(game) = &mut app_state {
            game.run(dt.as_secs_f64());
            if game.is_exit_intended() {
                // Return to start screen instead of exiting entirely
                app_state = AppState::Start(StartMenuState::new());
                continue;
            }
        }

        // handle input
        let poll_timeout = std::cmp::min(dt, target_frame_duration);
        if event::poll(poll_timeout)? {
            if let Event::Key(key) = event::read()? { if key.kind == KeyEventKind::Press {
                app_state = match app_state {
                    AppState::Start(mut menu) => handle_start_input(&mut menu, key).unwrap_or(AppState::Start(menu)),
                    AppState::Playing(mut game) => { handle_game_key(&mut game, key); AppState::Playing(game) },
                    AppState::Exit => AppState::Exit,
                };
            }}
        }

        // cap frame rate
        let frame_duration = last_frame_time.elapsed();
        if frame_duration < target_frame_duration { thread::sleep(target_frame_duration - frame_duration); }
    }
    Ok(())
}

fn handle_game_key(game: &mut BinaryNumbersGame, key: KeyEvent) {
    match key.code {
        KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers == KeyModifiers::CONTROL => {
            game.handle_game_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        }
        _ => game.handle_game_input(key),
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // Optional: allow bypass start screen via CLI arg bits
    if let Some(arg) = env::args().nth(1) { if ["4","8","12","16"].contains(&arg.as_str()) { let bits = match arg.as_str() {"4"=>Bits::Four,"8"=>Bits::Eight,"12"=>Bits::Twelve,"16"=>Bits::Sixteen,_=>Bits::Eight}; let mut game = BinaryNumbersGame::new(bits); let mut terminal = ratatui::init(); run_direct_game(&mut terminal, &mut game)?; ratatui::restore(); return Ok(()); } }
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

// Fallback direct game loop if bits supplied via CLI
fn run_direct_game(terminal: &mut ratatui::DefaultTerminal, game: &mut BinaryNumbersGame) -> color_eyre::Result<()> {
    let mut last_frame_time = Instant::now();
    let target_frame_duration = std::time::Duration::from_millis(33);
    while !game.is_exit_intended() {
        let now = Instant::now();
        let dt = now - last_frame_time;
        last_frame_time = now;
        terminal.draw(|f| f.render_widget(&mut *game, f.area()))?;
        game.run(dt.as_secs_f64());
        let poll_timeout = std::cmp::min(dt, target_frame_duration);
        if event::poll(poll_timeout)? { if let Event::Key(key) = event::read()? { if key.kind == KeyEventKind::Press { handle_game_key(game, key); } } }
        let frame_duration = last_frame_time.elapsed();
        if frame_duration < target_frame_duration { thread::sleep(target_frame_duration - frame_duration); }
    }
    Ok(())
}
