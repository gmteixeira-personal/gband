use gband_core::geometry::Size;
use gband_lua::{StatusLineOptions, StatusLinePosition};
use ratatui::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub position: StatusLinePosition,
    pub height: u16,
}

impl Placement {
    pub const OFF: Self = Self {
        position: StatusLinePosition::Off,
        height: 1,
    };

    pub fn new(options: &StatusLineOptions) -> Self {
        Self {
            position: options.position,
            height: options.height,
        }
    }

    pub fn split(self, terminal: Size) -> (Rect, Option<Rect>) {
        let whole = Rect::new(0, 0, terminal.cols, terminal.rows);
        if self.position == StatusLinePosition::Off || terminal.rows <= self.height {
            return (whole, None);
        }
        let rest = terminal.rows - self.height;
        match self.position {
            StatusLinePosition::Top => (
                Rect::new(0, self.height, terminal.cols, rest),
                Some(Rect::new(0, 0, terminal.cols, self.height)),
            ),
            _ => (
                Rect::new(0, 0, terminal.cols, rest),
                Some(Rect::new(0, rest, terminal.cols, self.height)),
            ),
        }
    }

    pub fn reported(self, terminal: Size) -> Size {
        let (ribbon, _) = self.split(terminal);
        Size::new(ribbon.width, ribbon.height)
    }
}

impl Default for Placement {
    fn default() -> Self {
        Self::new(&StatusLineOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placement(position: StatusLinePosition, height: u16) -> Placement {
        Placement { position, height }
    }

    #[test]
    fn default_placement() {
        let (ribbon, status) = Placement::default().split(Size::new(80, 24));
        assert_eq!(ribbon, Rect::new(0, 0, 80, 23));
        assert_eq!(status, Some(Rect::new(0, 23, 80, 1)));
    }

    #[test]
    fn top_placement() {
        let (ribbon, status) = placement(StatusLinePosition::Top, 1).split(Size::new(80, 24));
        assert_eq!(ribbon, Rect::new(0, 1, 80, 23));
        assert_eq!(status, Some(Rect::new(0, 0, 80, 1)));
    }

    #[test]
    fn off() {
        let (ribbon, status) = Placement::OFF.split(Size::new(80, 24));
        assert_eq!(ribbon, Rect::new(0, 0, 80, 24));
        assert_eq!(status, None);
    }

    #[test]
    fn taller_status_line() {
        let (ribbon, status) = placement(StatusLinePosition::Bottom, 3).split(Size::new(80, 24));
        assert_eq!(ribbon, Rect::new(0, 0, 80, 21));
        assert_eq!(status, Some(Rect::new(0, 21, 80, 3)));
    }

    #[test]
    fn terminal_too_short() {
        let (ribbon, status) = placement(StatusLinePosition::Bottom, 2).split(Size::new(80, 2));
        assert_eq!(ribbon, Rect::new(0, 0, 80, 2));
        assert_eq!(status, None);
        let (_, status) = placement(StatusLinePosition::Bottom, 2).split(Size::new(80, 3));
        assert_eq!(status, Some(Rect::new(0, 1, 80, 2)));
    }

    #[test]
    fn reported_size() {
        assert_eq!(
            Placement::default().reported(Size::new(80, 24)),
            Size::new(80, 23)
        );
        assert_eq!(
            Placement::OFF.reported(Size::new(80, 24)),
            Size::new(80, 24)
        );
    }
}
