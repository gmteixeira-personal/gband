use ratatui::style::{Color, Modifier, Style};

const COLORTERM: &str = "COLORTERM";
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];
const CUBE_START: u8 = 16;
const GRAY_START: u8 = 232;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSupport {
    TrueColor,
    #[default]
    Indexed,
}

impl ColorSupport {
    pub fn detect() -> Self {
        Self::from_colorterm(std::env::var(COLORTERM).ok().as_deref())
    }

    pub fn from_colorterm(value: Option<&str>) -> Self {
        match value {
            Some("truecolor" | "24bit") => ColorSupport::TrueColor,
            _ => ColorSupport::Indexed,
        }
    }

    pub fn color(self, color: gband_lua::Color) -> Color {
        match (self, color) {
            (_, gband_lua::Color::Index(index)) => Color::Indexed(index),
            (ColorSupport::TrueColor, gband_lua::Color::Rgb(r, g, b)) => Color::Rgb(r, g, b),
            (ColorSupport::Indexed, gband_lua::Color::Rgb(r, g, b)) => {
                Color::Indexed(nearest_index(r, g, b))
            }
        }
    }

    pub fn style(self, style: &gband_lua::Style) -> Style {
        let mut drawn = Style::new();
        if let Some(fg) = style.fg {
            drawn = drawn.fg(self.color(fg));
        }
        if let Some(bg) = style.bg {
            drawn = drawn.bg(self.color(bg));
        }
        for (set, modifier) in [
            (style.bold, Modifier::BOLD),
            (style.italic, Modifier::ITALIC),
            (style.underline, Modifier::UNDERLINED),
            (style.reverse, Modifier::REVERSED),
            (style.dim, Modifier::DIM),
        ] {
            if set {
                drawn = drawn.add_modifier(modifier);
            }
        }
        drawn
    }
}

fn palette(index: u8) -> (u8, u8, u8) {
    if index >= GRAY_START {
        let level = 8 + 10 * (index - GRAY_START);
        return (level, level, level);
    }
    let cube = index - CUBE_START;
    (
        CUBE[usize::from(cube / 36)],
        CUBE[usize::from(cube / 6 % 6)],
        CUBE[usize::from(cube % 6)],
    )
}

pub fn nearest_index(r: u8, g: u8, b: u8) -> u8 {
    let distance = |index: u8| {
        let (pr, pg, pb) = palette(index);
        let channel = |a: u8, b: u8| (i32::from(a) - i32::from(b)).pow(2);
        channel(r, pr) + channel(g, pg) + channel(b, pb)
    };
    (CUBE_START..=u8::MAX)
        .min_by_key(|&index| (distance(index), index))
        .unwrap_or(CUBE_START)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truecolor_detection() {
        assert_eq!(
            ColorSupport::from_colorterm(Some("truecolor")),
            ColorSupport::TrueColor
        );
        assert_eq!(
            ColorSupport::from_colorterm(Some("24bit")),
            ColorSupport::TrueColor
        );
        assert_eq!(
            ColorSupport::from_colorterm(Some("yes")),
            ColorSupport::Indexed
        );
        assert_eq!(ColorSupport::from_colorterm(None), ColorSupport::Indexed);
    }

    #[test]
    fn truecolor_terminal() {
        let color = ColorSupport::TrueColor.color(gband_lua::Color::Rgb(0xff, 0x88, 0x00));
        assert_eq!(color, Color::Rgb(0xff, 0x88, 0x00));
    }

    #[test]
    fn downgrade_without_truecolor() {
        let color = ColorSupport::Indexed.color(gband_lua::Color::Rgb(0xff, 0x88, 0x00));
        assert_eq!(color, Color::Indexed(208));
    }

    #[test]
    fn gray_downgrade() {
        assert_eq!(nearest_index(0x30, 0x30, 0x30), 236);
    }

    #[test]
    fn index_unchanged() {
        for support in [ColorSupport::TrueColor, ColorSupport::Indexed] {
            assert_eq!(support.color(gband_lua::Color::Index(4)), Color::Indexed(4));
            assert_eq!(support.color(gband_lua::Color::Index(9)), Color::Indexed(9));
        }
    }

    #[test]
    fn tie_goes_to_the_lower_index() {
        assert_eq!(palette(16), (0, 0, 0));
        assert_eq!(palette(232), (8, 8, 8));
        assert_eq!(nearest_index(4, 4, 4), 16);
        assert_eq!(nearest_index(0, 0, 0), 16);
    }

    #[test]
    fn exact_palette_colors_map_to_themselves() {
        for index in CUBE_START..=u8::MAX {
            let (r, g, b) = palette(index);
            let found = nearest_index(r, g, b);
            assert_eq!(palette(found), (r, g, b), "{index}");
        }
    }

    #[test]
    fn style_fields() {
        let style = gband_lua::Style {
            fg: Some(gband_lua::Color::Index(3)),
            bg: None,
            bold: true,
            italic: false,
            underline: true,
            reverse: true,
            dim: true,
        };
        let drawn = ColorSupport::Indexed.style(&style);
        assert_eq!(drawn.fg, Some(Color::Indexed(3)));
        assert_eq!(drawn.bg, None);
        assert_eq!(
            drawn.add_modifier,
            Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED | Modifier::DIM
        );
        assert_eq!(
            ColorSupport::Indexed.style(&gband_lua::Style::default()),
            Style::new()
        );
    }
}
