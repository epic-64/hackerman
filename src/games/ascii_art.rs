use std::collections::HashMap;
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Widget};
use crate::games::main_screen_widget::{MainScreenWidget, WidgetRef};
use crate::utils::{parse_ascii_art, AsciiArtWidget, TrimMargin};

pub(crate) struct AsciiArtMain {
    exit_intended: bool,
}

impl AsciiArtMain {
    pub fn new() -> Self {
        Self { exit_intended: false }
    }
}

impl WidgetRef for AsciiArtMain {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
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
            ___ \/ ._\//_/__/  ,\_\//__\/.  \_//__
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

        AsciiArtWidget::from_art(art, foreground_colors, &color_map, Color::Gray).render(area, buf);
    }
}

impl MainScreenWidget for AsciiArtMain {
    fn run(&mut self) {}

    fn handle_input(&mut self, input: KeyEvent) -> () {}

    fn is_exit_intended(&self) -> bool { self.exit_intended }
}