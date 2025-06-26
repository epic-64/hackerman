use std::collections::HashMap;
use ratatui::layout::Alignment::Center;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

pub trait ToDuration {
    /// Convert a number to a [`std::time::Duration`].
    fn milliseconds(&self) -> std::time::Duration;
}

impl ToDuration for u64 {
    /// Convert a number to a [`std::time::Duration`].
    fn milliseconds(&self) -> std::time::Duration {
        std::time::Duration::from_millis(*self)
    }
}

pub trait TrimMargin {
    #![allow(dead_code)]
    fn nice(&self) -> String;
}

impl TrimMargin for str {
    /// Remove the surrounding whitespace from a multi-line string.
    /// Opinionated: it removes the first and last line, and trims the leading whitespace based on minimum indentation.
    /// (empty lines are ignored)
    fn nice(&self) -> String {
        let lines: Vec<&str> = self.lines().collect();

        let content_lines = &lines[1..lines.len().saturating_sub(1)];

        let indent = content_lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.chars().take_while(|&c| c == ' ').count())
            .min()
            .unwrap_or(0);

        let trimmed = |line: &&str| {
            if line.len() >= indent {
                line.chars().skip(indent).collect::<String>()
            } else {
                (*line).to_string()
            }
        };

        content_lines.iter().map(trimmed).collect::<Vec<_>>().join("\n")
    }
}

pub struct AsciiCell {
    pub ch: char,
    pub x: u16,
    pub y: u16,
    pub color: Color,
}

pub fn parse_ascii_art(
    art: String,
    color_map_str: String,
    color_map: &HashMap<char, Color>,
    default_color: Color,
) -> Vec<AsciiCell> {
    let art_lines: Vec<Vec<char>> = art.lines().map(|line| line.chars().collect()).collect();
    let color_lines: Vec<Vec<char>> = color_map_str.lines().map(|line| line.chars().collect()).collect();

    assert_eq!(art_lines.len(), color_lines.len(), "Art and color string must have same height");

    let mut pixels = Vec::new();

    for (y, (art_row, color_row)) in art_lines.iter().zip(color_lines.iter()).enumerate() {
        assert_eq!(art_row.len(), color_row.len(), "Mismatched line lengths");

        for (x, (&ch, &color_ch)) in art_row.iter().zip(color_row.iter()).enumerate() {
            let color = color_map.get(&color_ch).cloned().unwrap_or(default_color);
            pixels.push(AsciiCell {
                ch,
                x: x as u16,
                y: y as u16,
                color,
            });
        }
    }

    pixels
}

pub struct AsciiCells {
    pub cells: Vec<AsciiCell>,
}

impl AsciiCells {
    pub fn new(cells: Vec<AsciiCell>) -> Self {
        Self { cells }
    }

    pub fn from(
        art: String,
        color_map_str: String,
        color_map: &HashMap<char, Color>,
        default_color: Color,
    ) -> Self {
        Self { cells: parse_ascii_art(art, color_map_str, color_map, default_color) }
    }

    pub fn get_width(&self) -> u16 {
        self.cells.iter().map(|cell| cell.x).max().unwrap_or(0) + 1
    }

    pub fn get_height(&self) -> u16 {
        self.cells.iter().map(|cell| cell.y).max().unwrap_or(0) + 1
    }

    pub fn get_centered_area(&self, area: Rect) -> Rect {
        let width = self.get_width();
        let height = self.get_height();
        let x_offset = (area.width.saturating_sub(width)) / 2;
        let y_offset = (area.height.saturating_sub(height)) / 2;

        Rect::new(area.x + x_offset, area.y + y_offset, width, height)
    }
}

pub struct AsciiArtWidget {
    collection: AsciiCells,
}

impl AsciiArtWidget {
    pub fn new(collection: AsciiCells) -> Self {
        Self { collection }
    }
}

impl Widget for AsciiArtWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for pixel in self.collection.cells {
            let position = Position::new(pixel.x + area.x, pixel.y + area.y);

            if area.contains(position) {
                buf.cell_mut(position)
                    .expect("Failed to get cell at position")
                    .set_char(pixel.ch)
                    .set_fg(pixel.color);
            }
        }
    }
}

fn buffer_to_string(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn vertically_center(area: Rect) -> Rect {
    let [_, center, _] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Min(1),
            Constraint::Fill(1),
        ])
        .areas(area);
    center
}

// fn render_contents: impl FnOnce(Rect, &mut Buffer)
pub fn render_centered_block(
    area: Rect,
    buf: &mut Buffer,
    render_box: impl FnOnce(Rect, &mut Buffer),
    render_contents: impl FnOnce(Rect, &mut Buffer)
) {
    render_box(area, buf);
    render_contents(vertically_center(area), buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annoying_default_behavior() {
        let input = "
            asdf
        ";

        let expected = "\n            asdf\n        ";

        assert_eq!(expected, input);
    }

    #[test]
    fn test_nice_basic() {
        let input = "
            This is a test string.
            It has multiple lines.
            Some lines are indented.
        ";

        let expected = vec![
            "This is a test string.",
            "It has multiple lines.",
            "Some lines are indented."
        ].join("\n");

        let result = input.nice();

        assert_eq!(expected, result);
    }

    #[test]
    fn test_nice_indentation() {
        let input = "
            This is a test string.
              It has multiple lines.
                Some lines are indented.
        ";

        let expected = vec![
            "This is a test string.",
            "  It has multiple lines.",
            "    Some lines are indented."
        ].join("\n");

        let result = input.nice();

        assert_eq!(expected, result);
    }

    #[test]
    fn test_preserve_empty_lines() {
        let input = "
            This is a test string.

            It has multiple lines.

            Some lines are indented.
        ";

        let expected = vec![
            "This is a test string.",
            "",
            "It has multiple lines.",
            "",
            "Some lines are indented."
        ].join("\n");

        let result = input.nice();

        assert_eq!(expected, result);
    }
}