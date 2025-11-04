use std::collections::HashMap;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use hackerman::games::binary_numbers::{BinaryNumbersGame, Bits};
use hackerman::games::main_screen_widget::MainScreenWidget;
use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState};
use std::time::Instant;
use std::thread;
use nice_trim::NiceTrim;
use ratatui::layout::Flex::Center;
use hackerman::utils::{AsciiArtWidget, AsciiCells};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

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
        Self {
            items,
            list_state: ListState::default().with_selected(Some(1)),
        } // default to normal
    }
    fn selected_index(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }
    fn selected_bits(&self) -> Bits {
        self.items[self.selected_index()].1.clone()
    }
    fn select_next(&mut self) {
        self.list_state.select_next();
    }
    fn select_previous(&mut self) {
        self.list_state.select_previous();
    }
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
            return Some(AppState::Exit);
        }
        _ => {}
    }
    None
}

fn render_start_screen(state: &mut StartMenuState, area: Rect, buf: &mut Buffer) {
    // Height reserved for ASCII art header
    const ASCII_HEIGHT: u16 = 10; // approximate lines used by render_big_text

    // Render ASCII art at the top spanning full width slice (same as before)
    let ascii_area = Rect::new(area.x, area.y, area.width, ASCII_HEIGHT.min(area.height));
    render_big_text(ascii_area, buf);

    // Compute list metrics
    let epic_labels: Vec<String> = state
        .items
        .iter()
        .map(|(label, _)| format!("» {} «", label.to_uppercase()))
        .collect();

    let list_height: u16 = epic_labels.len() as u16; // one line per item
    let list_width: u16 = epic_labels
        .iter()
        .map(|s| s.len() as u16)
        .max()
        .unwrap_or(0)
        .saturating_add(4); // padding + potential highlight symbol

    // Center vertically in remaining space below ASCII art
    let remaining_height = area.height.saturating_sub(ASCII_HEIGHT);
    let offset_y = (remaining_height.saturating_sub(list_height)) / 2; // vertical centering
    let list_y = area.y + ASCII_HEIGHT + offset_y;

    // Center horizontally
    let offset_x = (area.width.saturating_sub(list_width)) / 2;
    let list_x = area.x + offset_x;

    // Ensure we don't overflow
    let list_area = Rect::new(list_x, list_y, list_width.min(area.width), list_height.min(remaining_height));

    // Build styled items with a color palette for an "epic" feel
    let palette = [
        Color::Magenta,
        Color::LightMagenta,
        Color::LightBlue,
        Color::LightCyan,
        Color::Yellow,
    ];
    let items: Vec<ListItem> = epic_labels
        .into_iter()
        .enumerate()
        .map(|(i, text)| {
            let color = palette[i % palette.len()];
            ListItem::new(Line::from(Span::styled(
                text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
        .highlight_symbol("▶ ");

    ratatui::widgets::StatefulWidget::render(list, list_area, buf, &mut state.list_state);
}

fn run_app(terminal: &mut ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut app_state = AppState::Start(StartMenuState::new());
    let mut last_frame_time = Instant::now();
    let target_frame_duration = std::time::Duration::from_millis(33); // ~30 FPS

    while !matches!(app_state, AppState::Exit) {
        let now = Instant::now();
        let dt = now - last_frame_time;
        last_frame_time = now;

        terminal.draw(|f| match &mut app_state {
            AppState::Start(menu) => render_start_screen(menu, f.area(), f.buffer_mut()),
            AppState::Playing(game) => {
                f.render_widget(&mut *game, f.area());
            }
            AppState::Exit => {}
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
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app_state = match app_state {
                        AppState::Start(mut menu) => {
                            handle_start_input(&mut menu, key).unwrap_or(AppState::Start(menu))
                        }
                        AppState::Playing(mut game) => {
                            handle_game_key(&mut game, key);
                            AppState::Playing(game)
                        }
                        AppState::Exit => AppState::Exit,
                    };
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

fn handle_game_key(game: &mut BinaryNumbersGame, key: KeyEvent) {
    match key.code {
        KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers == KeyModifiers::CONTROL => {
            game.handle_game_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        }
        _ => game.handle_game_input(key),
    }
}

fn render_big_text(area: Rect, buf: &mut Buffer) {
    let art = r#"
 ,,        ,,        ,,
*MM        db   mm  *MM                                `7MM
 MM             MM   MM                                  MM
 MM,dMMb.`7MM mmMMmm MM,dMMb.`7Mb,od8 .gP"Ya   ,6"Yb.    MM  ,MP'
 MM    `Mb MM   MM   MM    `Mb MM' "',M'   Yb 8)   MM    MM ;Y
 MM     M8 MM   MM   MM     M8 MM    8M""""""  ,pm9MM    MM;Mm
 MM.   ,M9 MM   MM   MM.   ,M9 MM    YM.    , 8M   MM    MM `Mb.
 P^YbmdP'.JMML. `MbmoP^YbmdP'.JMML.   `Mbmmd' `Moo9^Yo..JMML. YA.
   142    151   164     142   162       145     141       153
    "#.nice();

    let colors = r#"
 ,,        ,,        ,,
*MM        db   mm  *MM                                `7MM
 MM             MM   MM                                  MM
 MM,dMMb.`7MM mmMMmm MM,dMMb.`7Mb,od8 .gP"Ya   ,6"Yb.    MM  ,MP'
 MM    `Mb MM   MM   MM    `Mb MM' "',M'   Yb 8)   MM    MM ;Y
 MM     M8 MM   MM   MM     M8 MM    8M""""""  ,pm9MM    MM;Mm
 MM.   ,M9 MM   MM   MM.   ,M9 MM    YM.    , 8M   MM    MM `Mb.
 P^YbmdP'.JMML. `MbmoP^YbmdP'.JMML.   `Mbmmd' `Moo9^Yo..JMML. YA.
   142    151   164     142   162       145     141       153
    "#.nice();

    let color_map = HashMap::from([
        ('█', Color::White),
        ('R', Color::Red),
        ('r', Color::LightRed),
        ('G', Color::LightGreen),
        ('g', Color::Green),
        ('B', Color::LightBlue),
        ('b', Color::Blue),
        ('Y', Color::LightYellow),
        //('y', Color::LightYellow),
        ('P', Color::LightMagenta),
        ('p', Color::Magenta),
        ('C', Color::LightCyan),
        ('W', Color::White),
        (' ', Color::Reset),
    ]);

    let default_color = Color::LightBlue;
    let cells = AsciiCells::from(art.to_string(), colors.to_string(), &color_map, default_color);
    let width = cells.get_width();
    let ascii_widget = AsciiArtWidget::new(cells);

    let [centered] = Layout::horizontal([Constraint::Length(width)]).flex(Center).areas(area);
    ascii_widget.render(centered, buf);
}