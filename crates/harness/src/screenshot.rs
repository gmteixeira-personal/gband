use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use vt100::{Color, Screen};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Colour>,
    pub bg: Option<Colour>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colour {
    Palette(u8),
    Rgb(u8, u8, u8),
}

impl Colour {
    fn of(color: Color) -> Option<Self> {
        match color {
            Color::Default => None,
            Color::Idx(index) => Some(Colour::Palette(index)),
            Color::Rgb(r, g, b) => Some(Colour::Rgb(r, g, b)),
        }
    }
}

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Colour::Palette(index) => write!(f, "{index}"),
            Colour::Rgb(r, g, b) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
        }
    }
}

impl Style {
    pub fn of(cell: &vt100::Cell) -> Self {
        Self {
            fg: Colour::of(cell.fgcolor()),
            bg: Colour::of(cell.bgcolor()),
            bold: cell.bold(),
            dim: cell.dim(),
            italic: cell.italic(),
            underline: cell.underline(),
            inverse: cell.inverse(),
        }
    }

    fn describe(&self) -> String {
        let mut parts = Vec::new();
        if let Some(fg) = self.fg {
            parts.push(format!("fg={fg}"));
        }
        if let Some(bg) = self.bg {
            parts.push(format!("bg={bg}"));
        }
        for (set, name) in [
            (self.bold, "bold"),
            (self.dim, "dim"),
            (self.italic, "italic"),
            (self.underline, "underline"),
            (self.inverse, "inverse"),
        ] {
            if set {
                parts.push(name.to_owned());
            }
        }
        parts.join(" ")
    }
}

pub fn row_text(screen: &Screen, row: u16) -> String {
    let (_, cols) = screen.size();
    let mut text = String::new();
    for col in 0..cols {
        let Some(cell) = screen.cell(row, col) else {
            break;
        };
        if cell.is_wide_continuation() {
            continue;
        }
        if cell.has_contents() {
            text.push_str(cell.contents());
        } else {
            text.push(' ');
        }
    }
    text.trim_end().to_owned()
}

fn row_styles(screen: &Screen, row: u16) -> Vec<Style> {
    let (_, cols) = screen.size();
    let mut styles: Vec<Style> = Vec::with_capacity(usize::from(cols));
    for col in 0..cols {
        let style = match screen.cell(row, col) {
            Some(cell) if cell.is_wide_continuation() => styles.last().copied().unwrap_or_default(),
            Some(cell) => Style::of(cell),
            None => Style::default(),
        };
        styles.push(style);
    }
    styles
}

fn runs(row: u16, styles: &[Style]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut col = 0;
    while col < styles.len() {
        let style = styles[col];
        let mut last = col;
        while last + 1 < styles.len() && styles[last + 1] == style {
            last += 1;
        }
        if style != Style::default() {
            let span = if last == col {
                format!("{row}:{col}")
            } else {
                format!("{row}:{col}-{last}")
            };
            lines.push(format!("{span} {}", style.describe()));
        }
        col = last + 1;
    }
    lines
}

pub fn render(screen: &Screen, styles: bool) -> String {
    let (rows, cols) = screen.size();
    let (row, col) = screen.cursor_position();
    let shown = if screen.hide_cursor() {
        "hidden"
    } else {
        "shown"
    };
    let mut text = format!("size {cols}x{rows} cursor {row}:{col} {shown}\n");
    let width = rows.saturating_sub(1).to_string().len();
    for row in 0..rows {
        let _ = writeln!(text, "{row:>width$}|{}", row_text(screen, row));
    }
    if styles {
        text.push_str("--\n");
        for row in 0..rows {
            for line in runs(row, &row_styles(screen, row)) {
                text.push_str(&line);
                text.push('\n');
            }
        }
    }
    text
}

pub fn slug(text: &str) -> String {
    let mut slug = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_owned()
}

pub fn reference_path(test_file: &Path, case: &str, name: Option<&str>) -> PathBuf {
    let directory = test_file.parent().unwrap_or(Path::new("."));
    let stem = test_file
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut file = slug(case);
    if let Some(name) = name {
        file.push_str("--");
        file.push_str(&slug(name));
    }
    file.push_str(".txt");
    directory.join("screenshots").join(stem).join(file)
}

pub fn pending_path(reference: &Path) -> PathBuf {
    let mut name = reference.as_os_str().to_owned();
    name.push(".new");
    PathBuf::from(name)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mismatch {
    Duplicate(PathBuf),
    Missing(PathBuf),
    Differs { reference: PathBuf, diff: String },
    Io(String),
}

impl std::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mismatch::Duplicate(path) => write!(
                f,
                "the screenshot reference {} was already compared in this run",
                path.display()
            ),
            Mismatch::Missing(path) => write!(
                f,
                "no screenshot reference {}; the screenshot is in {}",
                path.display(),
                pending_path(path).display()
            ),
            Mismatch::Differs { reference, .. } => write!(
                f,
                "the screenshot differs from {}; the screenshot is in {}",
                reference.display(),
                pending_path(reference).display()
            ),
            Mismatch::Io(reason) => f.write_str(reason),
        }
    }
}

#[derive(Default)]
pub struct References {
    seen: HashSet<PathBuf>,
}

impl References {
    pub fn compare(
        &mut self,
        reference: &Path,
        screenshot: &str,
        update: bool,
    ) -> Result<(), Mismatch> {
        if !self.seen.insert(reference.to_path_buf()) {
            return Err(Mismatch::Duplicate(reference.to_path_buf()));
        }
        let pending = pending_path(reference);
        let io = |error: std::io::Error, path: &Path| {
            Mismatch::Io(format!("cannot write {}: {error}", path.display()))
        };
        let remove_pending = || match fs::remove_file(&pending) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(io(error, &pending)),
            _ => Ok(()),
        };
        if update {
            write(reference, screenshot).map_err(|error| io(error, reference))?;
            return remove_pending();
        }
        let existing = fs::read_to_string(reference).ok();
        if existing.as_deref() == Some(screenshot) {
            return remove_pending();
        }
        write(&pending, screenshot).map_err(|error| io(error, &pending))?;
        Err(match existing {
            None => Mismatch::Missing(reference.to_path_buf()),
            Some(existing) => Mismatch::Differs {
                reference: reference.to_path_buf(),
                diff: diff(&existing, screenshot),
            },
        })
    }
}

fn write(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

#[derive(Default)]
struct Parsed<'a> {
    header: &'a str,
    rows: BTreeMap<u32, Vec<&'a str>>,
    styles: BTreeMap<u32, Vec<&'a str>>,
}

impl<'a> Parsed<'a> {
    fn lines(&self, row: u32) -> Vec<&'a str> {
        [&self.rows, &self.styles]
            .into_iter()
            .filter_map(|map| map.get(&row))
            .flatten()
            .copied()
            .collect()
    }
}

fn row_number(line: &str, separator: char) -> Option<u32> {
    line.split_once(separator)?.0.trim_start().parse().ok()
}

fn parse(text: &str) -> Parsed<'_> {
    let mut parsed = Parsed::default();
    let mut lines = text.lines();
    parsed.header = lines.next().unwrap_or_default();
    let mut in_styles = false;
    for line in lines {
        if !in_styles && line == "--" {
            in_styles = true;
            continue;
        }
        let (map, separator) = if in_styles {
            (&mut parsed.styles, ':')
        } else {
            (&mut parsed.rows, '|')
        };
        let row = row_number(line, separator).unwrap_or(u32::MAX);
        map.entry(row).or_default().push(line);
    }
    parsed
}

pub fn diff(reference: &str, screenshot: &str) -> String {
    let old = parse(reference);
    let new = parse(screenshot);
    let mut text = String::new();
    if old.header != new.header {
        let _ = writeln!(text, "-{}", old.header);
        let _ = writeln!(text, "+{}", new.header);
    }
    let rows: std::collections::BTreeSet<u32> = [&old.rows, &old.styles, &new.rows, &new.styles]
        .into_iter()
        .flat_map(|map| map.keys().copied())
        .collect();
    for row in rows {
        let (before, after) = (old.lines(row), new.lines(row));
        if before == after {
            continue;
        }
        for line in before {
            let _ = writeln!(text, "-{line}");
        }
        for line in after {
            let _ = writeln!(text, "+{line}");
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use gband_scratch::Scratch;

    fn screen(cols: u16, rows: u16, output: &str) -> vt100::Parser {
        let mut parser = vt100::Parser::new(rows, cols, 0);
        parser.process(output.as_bytes());
        parser
    }

    #[test]
    fn plain_screen_has_no_style_lines() {
        let parser = screen(10, 3, "hi\r\n  there");
        assert_eq!(
            render(parser.screen(), true),
            "size 10x3 cursor 1:7 shown\n0|hi\n1|  there\n2|\n--\n"
        );
    }

    #[test]
    fn text_only_omits_the_separator() {
        let parser = screen(10, 2, "\x1b[1mhi\x1b[?25l");
        assert_eq!(
            render(parser.screen(), false),
            "size 10x2 cursor 0:2 hidden\n0|hi\n1|\n"
        );
    }

    #[test]
    fn row_numbers_are_aligned() {
        let parser = screen(4, 11, "");
        let text = render(parser.screen(), false);
        assert!(text.contains("\n 0|\n"), "{text}");
        assert!(text.contains("\n10|\n"), "{text}");
    }

    #[test]
    fn run_of_one_colour() {
        let mut output = "\x1b[6;1H\x1b[1mbold \x1b[0m".to_owned();
        output.push_str(&" ".repeat(20));
        output.push_str("\x1b[38;2;122;162;247;48;2;26;27;38mwindow 1\x1b[0m\x1b[7mx\x1b[0m");
        let parser = screen(40, 6, &output);
        let text = render(parser.screen(), true);
        assert!(text.contains("\n5:25-32 fg=#7aa2f7 bg=#1a1b26\n"), "{text}");
        assert!(text.contains("\n5:0-4 bold\n"), "{text}");
        assert!(text.contains("\n5:33 inverse\n"), "{text}");
    }

    #[test]
    fn palette_colours_and_attribute_order() {
        let parser = screen(10, 1, "\x1b[31;44;7;4;3;1mab\x1b[0;2;38;5;200mc");
        let text = render(parser.screen(), true);
        assert!(
            text.ends_with("--\n0:0-1 fg=1 bg=4 bold italic underline inverse\n0:2 fg=200 dim\n"),
            "{text}"
        );
    }

    #[test]
    fn wide_characters_appear_once() {
        let parser = screen(10, 1, "a\u{4e16}b\x1b[32m\u{754c}");
        let text = render(parser.screen(), true);
        assert!(text.contains("\n0|a\u{4e16}b\u{754c}\n"), "{text}");
        assert!(text.ends_with("--\n0:4-5 fg=2\n"), "{text}");
    }

    #[test]
    fn named_screenshot_path() {
        assert_eq!(
            reference_path(
                Path::new("tests/window_spec.lua"),
                "Shows the focused window",
                Some("two windows")
            ),
            Path::new("tests/screenshots/window_spec/shows-the-focused-window--two-windows.txt")
        );
        assert_eq!(
            reference_path(Path::new("t/a_spec.lua"), "  Ünicode -- case! ", None),
            Path::new("t/screenshots/a_spec/nicode-case.txt")
        );
    }

    #[test]
    fn first_run_writes_the_pending_file() {
        let root = Scratch::new("screenshot", "first");
        let reference = root.join("screenshots/a_spec/case.txt");
        let error = References::default()
            .compare(&reference, "shot\n", false)
            .unwrap_err();
        assert_eq!(error, Mismatch::Missing(reference.clone()));
        assert!(error.to_string().contains("case.txt"), "{error}");
        assert_eq!(
            fs::read_to_string(pending_path(&reference)).unwrap(),
            "shot\n"
        );
        assert!(!reference.exists());
    }

    #[test]
    fn update_accepts_and_removes_the_pending_file() {
        let root = Scratch::new("screenshot", "update");
        let reference = root.join("screenshots/a_spec/case.txt");
        let _ = References::default().compare(&reference, "shot\n", false);
        References::default()
            .compare(&reference, "shot\n", true)
            .unwrap();
        assert_eq!(fs::read_to_string(&reference).unwrap(), "shot\n");
        assert!(!pending_path(&reference).exists());
        References::default()
            .compare(&reference, "shot\n", false)
            .unwrap();
    }

    #[test]
    fn a_matching_run_removes_a_stale_pending_file() {
        let root = Scratch::new("screenshot", "stale");
        let reference = root.join("case.txt");
        write(&reference, "same\n").unwrap();
        write(&pending_path(&reference), "old\n").unwrap();
        References::default()
            .compare(&reference, "same\n", false)
            .unwrap();
        assert!(!pending_path(&reference).exists());
    }

    #[test]
    fn duplicate_reference_is_an_error() {
        let root = Scratch::new("screenshot", "duplicate");
        let reference = root.join("case.txt");
        let mut references = References::default();
        references.compare(&reference, "a\n", true).unwrap();
        assert_eq!(
            references.compare(&reference, "a\n", true),
            Err(Mismatch::Duplicate(reference.clone()))
        );
    }

    #[test]
    fn diff_names_only_the_changed_row() {
        let before =
            "size 10x3 cursor 0:0 shown\n0|one\n1|two\n2|three\n--\n0:0-2 bold\n2:0 fg=1\n";
        let after = "size 10x3 cursor 0:0 shown\n0|one\n1|TWO\n2|three\n--\n0:0-2 bold\n2:0 fg=1\n";
        assert_eq!(diff(before, after), "-1|two\n+1|TWO\n");
        let restyled =
            "size 10x3 cursor 0:0 shown\n0|one\n1|two\n2|three\n--\n0:0-2 bold\n2:0 fg=2\n";
        assert_eq!(
            diff(before, restyled),
            "-2|three\n-2:0 fg=1\n+2|three\n+2:0 fg=2\n"
        );
    }

    #[test]
    fn diff_names_a_changed_header() {
        let before = "size 10x1 cursor 0:0 shown\n0|a\n";
        let after = "size 10x1 cursor 0:1 shown\n0|a\n";
        assert_eq!(
            diff(before, after),
            "-size 10x1 cursor 0:0 shown\n+size 10x1 cursor 0:1 shown\n"
        );
    }
}
