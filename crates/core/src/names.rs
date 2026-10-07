use std::collections::HashMap;

use crate::layout::{Band, WindowId};

pub fn shown_names(band: &Band, names: &HashMap<WindowId, String>) -> HashMap<WindowId, String> {
    let named: Vec<(WindowId, &str)> = band
        .windows()
        .filter_map(|window| Some((window, names.get(&window)?.as_str())))
        .collect();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for (_, name) in &named {
        *counts.entry(name).or_default() += 1;
    }
    let mut numbers: HashMap<&str, usize> = HashMap::new();
    named
        .into_iter()
        .map(|(window, name)| {
            if counts[name] == 1 {
                return (window, name.to_owned());
            }
            let number = numbers.entry(name).or_default();
            *number += 1;
            (window, format!("{name} #{number}"))
        })
        .collect()
}
