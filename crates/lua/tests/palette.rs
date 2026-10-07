mod common;

use common::*;

#[test]
fn palette_set_by_the_init_file() {
    let scratch = Scratch::new("init");
    scratch.write("gband.palette.set({ bg = '#10A010', red = '#CC241D' })");
    let config = scratch.loaded();
    let palette = config.runtime.take_palette().unwrap();
    assert_eq!(palette.bg, Some((0x10, 0xa0, 0x10)));
    assert_eq!(palette.colors[1], Some((0xcc, 0x24, 0x1d)));
    assert_eq!(palette.fg, None);
    assert_eq!(
        eval::<String>(&config, "return gband.palette.get().bg"),
        "#10a010"
    );
}

#[test]
fn invalid_palette_field() {
    let scratch = Scratch::new("field");
    let dusk = scratch.user_file(
        "colors/dusk.lua",
        "local a = 1\nlocal b = 2\ngband.palette.set({ purple = '#800080' })",
    );
    scratch.write("gband.palette.set({ bg = '#101010' })\nok = gband.colorscheme('dusk')");
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "ok"));
    assert_eq!(config.errors.len(), 1, "{:?}", config.errors);
    assert_error_at(&config.errors[0], &dusk, 3, "purple");
    assert_eq!(
        eval::<String>(&config, "return gband.palette.get().bg"),
        "#101010"
    );
}

#[test]
fn invalid_palette_color() {
    let scratch = Scratch::new("color");
    let init = scratch.write("local a = 1\ngband.palette.set({ bg = 236 })");
    let error = scratch.load().err().expect("loading fails");
    assert_error_at(&error, &init, 2, "bg");
}

#[test]
fn palette_set_expects_a_table() {
    let scratch = Scratch::new("table");
    let init = scratch.write("gband.palette.set('#101010')");
    let error = scratch.load().err().expect("loading fails");
    assert_error_at(&error, &init, 1, "gband.palette.set");
}

#[test]
fn returned_table_is_a_copy() {
    let scratch = Scratch::new("copy");
    scratch.write("gband.palette.set({ fg = '#ffffff' })");
    let config = scratch.loaded();
    let fg: String = eval(
        &config,
        "local palette = gband.palette.get()\npalette.fg = '#000000'\npalette.bg = '#000000'\nreturn gband.palette.get().fg",
    );
    assert_eq!(fg, "#ffffff");
    let bg: Option<String> = eval(&config, "return gband.palette.get().bg");
    assert_eq!(bg, None);
}

#[test]
fn empty_palette_by_default() {
    let scratch = Scratch::new("empty");
    let config = scratch.loaded();
    let empty: bool = eval(&config, "return next(gband.palette.get()) == nil");
    assert!(empty);
    assert!(config.runtime.take_palette().unwrap().is_empty());
}

#[test]
fn palette_emptied_in_a_callback() {
    let scratch = Scratch::new("emptied");
    scratch.write(&format!("gband.palette.set({{ bg = '#101010' }})\n{JOB}"));
    let config = scratch.loaded();
    config.runtime.take_palette();
    clean(&run_job(&config, "gband.palette.set({})"));
    assert!(config.runtime.take_palette().unwrap().is_empty());
}
