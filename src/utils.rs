use std::collections::HashMap;
use ratatui::prelude::*;

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

pub struct AsciiPixel {
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
) -> Vec<AsciiPixel> {
    let art_lines: Vec<Vec<char>> = art.lines().map(|line| line.chars().collect()).collect();
    let color_lines: Vec<Vec<char>> = color_map_str.lines().map(|line| line.chars().collect()).collect();

    assert_eq!(art_lines.len(), color_lines.len(), "Art and color string must have same height");

    let mut pixels = Vec::new();

    for (y, (art_row, color_row)) in art_lines.iter().zip(color_lines.iter()).enumerate() {
        assert_eq!(art_row.len(), color_row.len(), "Mismatched line lengths");

        for (x, (&ch, &color_ch)) in art_row.iter().zip(color_row.iter()).enumerate() {
            let color = color_map.get(&color_ch).cloned().unwrap_or(default_color);
            pixels.push(AsciiPixel {
                ch,
                x: x as u16,
                y: y as u16,
                color,
            });
        }
    }

    pixels
}

pub struct AsciiArtWidget {
    pub pixels: Vec<AsciiPixel>,
}

impl AsciiArtWidget {
    pub fn new(pixels: Vec<AsciiPixel>) -> Self {
        Self { pixels }
    }

    pub fn from_art(
        art: String,
        color_map_str: String,
        color_map: &HashMap<char, Color>,
        default_color: Color,
    ) -> Self {
        let pixels = parse_ascii_art(art, color_map_str, color_map, default_color);
        Self { pixels }
    }
}

impl Widget for AsciiArtWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for pixel in self.pixels {
            let x = pixel.x + area.x;
            let y = pixel.y + area.y;

            // Only draw if inside the area
            if area.contains(Position::new(x, y)) {
                buf.cell_mut(Position::new(x, y))
                    .unwrap()
                    .set_char(pixel.ch)
                    .set_fg(pixel.color);
            }
        }
    }
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