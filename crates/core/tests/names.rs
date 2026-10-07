use std::collections::HashMap;

use gband_core::layout::{Band, BandId, Column, FloatingWindow, Proportion, WindowId};
use gband_core::names::shown_names;

fn band(columns: &[&[u32]], floating: &[u32]) -> Band {
    Band {
        id: BandId(1),
        columns: columns
            .iter()
            .map(|windows| {
                let mut column = Column::new(WindowId(windows[0]), Proportion::ONE_HALF);
                for &window in &windows[1..] {
                    column.windows.push(WindowId(window));
                    column.heights.push(column.heights[0]);
                }
                column
            })
            .collect(),
        floating: floating
            .iter()
            .map(|&window| FloatingWindow {
                window: WindowId(window),
                col: 0,
                row: 0,
                width: Proportion::ONE_HALF,
                full_width: false,
                rows: 10,
            })
            .collect(),
    }
}

fn names(entries: &[(u32, &str)]) -> HashMap<WindowId, String> {
    entries
        .iter()
        .map(|&(window, name)| (WindowId(window), name.to_owned()))
        .collect()
}

fn shown(band: &Band, entries: &[(u32, &str)]) -> Vec<(u32, String)> {
    let mut shown: Vec<(u32, String)> = shown_names(band, &names(entries))
        .into_iter()
        .map(|(window, name)| (window.0, name))
        .collect();
    shown.sort();
    shown
}

fn expected(entries: &[(u32, &str)]) -> Vec<(u32, String)> {
    entries
        .iter()
        .map(|&(window, name)| (window, name.to_owned()))
        .collect()
}

#[test]
fn two_shells() {
    assert_eq!(
        shown(&band(&[&[1], &[2]], &[]), &[(1, "bash"), (2, "bash")]),
        expected(&[(1, "bash #1"), (2, "bash #2")])
    );
}

#[test]
fn number_follows_the_layout() {
    assert_eq!(
        shown(&band(&[&[2], &[1]], &[]), &[(1, "bash"), (2, "bash")]),
        expected(&[(1, "bash #2"), (2, "bash #1")])
    );
}

#[test]
fn windows_of_a_column_numbered_top_to_bottom() {
    assert_eq!(
        shown(
            &band(&[&[3, 1], &[2]], &[]),
            &[(1, "sh"), (2, "sh"), (3, "sh")]
        ),
        expected(&[(1, "sh #2"), (2, "sh #3"), (3, "sh #1")])
    );
}

#[test]
fn floating_windows_numbered_last() {
    assert_eq!(
        shown(&band(&[&[2]], &[1]), &[(1, "bash"), (2, "bash")]),
        expected(&[(1, "bash #2"), (2, "bash #1")])
    );
}

#[test]
fn unique_name_has_no_number() {
    assert_eq!(
        shown(
            &band(&[&[1], &[2], &[3]], &[]),
            &[(1, "bash"), (2, "vim"), (3, "bash")]
        ),
        expected(&[(1, "bash #1"), (2, "vim"), (3, "bash #2")])
    );
}

#[test]
fn other_bands_do_not_count() {
    let entries = [(1, "bash"), (2, "bash")];
    assert_eq!(
        shown(&band(&[&[1]], &[]), &entries),
        expected(&[(1, "bash")])
    );
    assert_eq!(
        shown(&band(&[&[2]], &[]), &entries),
        expected(&[(2, "bash")])
    );
}

#[test]
fn manual_names_numbered_too() {
    assert_eq!(
        shown(&band(&[&[1], &[2]], &[]), &[(1, "logs"), (2, "logs")]),
        expected(&[(1, "logs #1"), (2, "logs #2")])
    );
}

#[test]
fn names_compared_as_exact_text() {
    assert_eq!(
        shown(&band(&[&[1], &[2]], &[]), &[(1, "Bash"), (2, "bash")]),
        expected(&[(1, "Bash"), (2, "bash")])
    );
}

#[test]
fn unnamed_windows_are_skipped() {
    assert_eq!(
        shown(&band(&[&[1], &[2], &[3]], &[]), &[(1, "sh"), (3, "sh")]),
        expected(&[(1, "sh #1"), (3, "sh #2")])
    );
}
