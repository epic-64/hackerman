use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

pub trait WidgetRef {
    fn render_ref(&self, area: Rect, buf: &mut Buffer);
}

pub trait WidgetGame: WidgetRef {
    fn run(&mut self) -> ();
    fn handle_input(&mut self, input: KeyEvent) -> ();
    fn is_exit_intended(&self) -> bool;
}