use std::collections::HashMap;
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Widget};
use crate::games::main_screen_widget::{MainScreenWidget, WidgetRef};
use crate::utils::{AsciiArtWidget, AsciiCells, TrimMargin};

pub struct AsciiArtMain {
    exit_intended: bool,
    timer: f64,
}

impl AsciiArtMain {
    pub fn new() -> Self {
        Self { exit_intended: false, timer: 0.0 }
    }
}

impl WidgetRef for AsciiArtMain {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let time_per_frame = 0.25;

        let frame = get_frame_1();
        let centered = frame.get_centered_area(area);

        AsciiArtWidget::new(frame).render(centered, buf);
    }
}

impl MainScreenWidget for AsciiArtMain {
    fn run(&mut self, dt: f64) {
        self.timer += dt;

        if self.timer > 10.0 {
            self.timer -= 10.0;
        }
    }

    fn handle_input(&mut self, _input: KeyEvent) -> () {}

    fn is_exit_intended(&self) -> bool { self.exit_intended }
}

fn get_frame_1() -> AsciiCells {
    let art = r"
                          ,@@@@@@@,
                  ,,,.   ,@@@@@@/@@,  .oo8888o.
               ,&%%&%&&%,@@@@@/@@@@@@,:8888\88/8o
              ,%&\%&&%&&%,@@@\@@/@@@88\88888/88'
              %&&%&%&/%&&%@@\@@/ /@@@88888\88888'
              %&&%/ %&%%&&@@\ V /@@' `88\8 `/88'
              `&%\ ` /%&'    |.|        \ '|8'
                  |o|        | |         | |
                  |.|        | |         | |
            ___ \/ ._\//_/__/  ,\_\//__\/.  \_//__
        ".nice();

    let foreground_colors = r"
                          ,@@@@@@@,
                  ,,,.   ,@@@@@@/@@,  .oo8888o.
               ,&%%&%&&%,@@@@@/@Y@@@@,:8888\88/8o
              ,%&\%&&%&&%,@@@\@Y/@@@88\88888/88'
              %&&%&%&/%&&%@@\@Y/ /@@@88888\88888'
              %&&%/ %&%%&&@@\ V /@@' `88\8 `/88'
              `&%\ ` /%&'    |.|        \ '|8'
                  |W|        | |         | |
                  |.|        | |         | |
            ___ B/ ._\BG_B__/  G\_BGG__B/.  \_BG__
        ".nice();

    let color_map = HashMap::from([
        ('@', Color::LightGreen),
        ('&', Color::Green),
        ('%', Color::Green),
        ('8', Color::Green),
        ('o', Color::Green),
        ('G', Color::Green),
        ('B', Color::LightGreen),
        ('W', Color::White),
        ('Y', Color::Yellow),
    ]);

    let default_color = Color::DarkGray;

    AsciiCells::from(art, foreground_colors, &color_map, default_color)
}