mod common;

use common::*;
use gband_core::action::Steps;
use gband_core::layout::Proportion;
use gband_core::view::CenterFocusedColumn;
use gband_lua::{Binding, Border, BorderChars, CharSet, ConfigError, Sides};

fn error_naming<'a>(errors: &'a [ConfigError], name: &str) -> &'a ConfigError {
    errors
        .iter()
        .find(|error| error.message.contains(name))
        .unwrap_or_else(|| panic!("no error names {name}: {errors:?}"))
}

#[test]
fn option_read_and_set_through_gband_opt() {
    let scratch = Scratch::new("read-set");
    scratch.server(
        "gband.opt.default_column_width = 1/3\npresets = gband.opt.width_presets\nwidth = gband.opt.default_column_width",
    );
    let config = scratch.loaded_server();
    assert_eq!(config.options.layout.default_width, Proportion::ONE_THIRD);
    let presets: Vec<f64> = global(&config, "presets");
    assert_eq!(presets, [1.0 / 3.0, 1.0 / 2.0, 2.0 / 3.0]);
    assert_eq!(global::<f64>(&config, "width"), 1.0 / 3.0);
}

#[test]
fn built_in_reads() {
    let scratch = Scratch::new("reads");
    scratch.write(
        "gband.opt.prefix = 'ctrl+b'\ngband.opt.center_focused_column = 'on-overflow'\nprefix = gband.opt.prefix\npolicy = gband.opt.center_focused_column",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "prefix"), "ctrl+b");
    assert_eq!(global::<String>(&config, "policy"), "on-overflow");
    assert_eq!(config.options.prefix, key("ctrl+b"));
    assert_eq!(
        config.options.center_focused_column,
        CenterFocusedColumn::OnOverflow
    );
}

#[test]
fn invalid_value_falls_back_to_the_default() {
    let scratch = Scratch::new("invalid");
    let path = scratch.server(
        "\n\n\ngband.opt.default_column_width = 1/3\ngband.opt.default_column_width = 'wide'",
    );
    let config = scratch.loaded_server();
    assert_eq!(config.options.layout.default_width, Proportion::ONE_HALF);
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_error_at(error, &path, 5, "default_column_width");
}

#[test]
fn gband_set_and_gband_opt_share_the_options() {
    let scratch = Scratch::new("shared");
    scratch
        .server("gband.set { default_column_width = 1/3 }\nwidth = gband.opt.default_column_width");
    let config = scratch.loaded_server();
    assert_eq!(global::<f64>(&config, "width"), 1.0 / 3.0);
}

#[test]
fn plugin_files_see_the_init_files_options() {
    let scratch = Scratch::new("plugin-sees");
    scratch.write("gband.opt.prefix = 'ctrl+b'");
    scratch.client_plugin("check", "seen = gband.opt.prefix");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "seen"), "ctrl+b");
}

#[test]
fn declared_plugin_option() {
    let scratch = Scratch::new("declared");
    scratch.client_plugin("hello",
        "full = gband.opt.declare('greeting', { type = 'string', default = 'hello', desc = 'what to say' })",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "full"), "hello.greeting");
    let value: String = eval(&config, "return gband.opt['hello.greeting']");
    assert_eq!(value, "hello");
}

#[test]
fn set_before_declared() {
    let scratch = Scratch::new("before");
    scratch.write("gband.opt['hello.greeting'] = 'hi'");
    scratch.client_plugin(
        "hello",
        "gband.opt.declare('greeting', { type = 'string', default = 'hello' })",
    );
    let config = scratch.loaded();
    let value: String = eval(&config, "return gband.opt['hello.greeting']");
    assert_eq!(value, "hi");
    assert!(config.errors.is_empty(), "{:?}", config.errors);
}

#[test]
fn set_before_declared_with_a_wrong_type() {
    let scratch = Scratch::new("before-wrong");
    let path = scratch.write("\ngband.opt['hello.count'] = 'many'");
    scratch.client_plugin(
        "hello",
        "gband.opt.declare('count', { type = 'integer', default = 3 })",
    );
    let config = scratch.loaded();
    let value: i64 = eval(&config, "return gband.opt['hello.count']");
    assert_eq!(value, 3);
    assert_error_at(
        error_naming(&config.errors, "hello.count"),
        &path,
        2,
        "hello.count",
    );
}

#[test]
fn never_declared() {
    let scratch = Scratch::new("never");
    let path = scratch.write("\n\n\n\n\n\n\n\ngband.opt['absent.flag'] = true");
    let config = scratch.loaded();
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_error_at(error, &path, 9, "absent.flag");
}

#[test]
fn value_outside_the_allowed_list() {
    let scratch = Scratch::new("values");
    scratch.write(
        "gband.opt.declare('mode', { type = 'string', values = { 'a', 'b' }, default = 'a' })\ngband.opt.mode = 'c'\nmode = gband.opt.mode",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "mode"), "a");
    error_naming(&config.errors, "mode");
}

#[test]
fn typed_values() {
    let scratch = Scratch::new("typed");
    scratch.write(
        "gband.opt.declare('flag', { type = 'boolean', default = false })
gband.opt.declare('count', { type = 'integer', default = 1 })
gband.opt.declare('ratio', { type = 'number', default = 1 })
gband.opt.flag = true
gband.opt.count = 4.0
gband.opt.ratio = 0.5
values = { gband.opt.flag, math.type(gband.opt.count), gband.opt.count, gband.opt.ratio }",
    );
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let values: String = eval(
        &config,
        "return table.concat({ tostring(values[1]), values[2], values[3], values[4] }, ' ')",
    );
    assert_eq!(values, "true integer 4 0.5");
    let rejected = Scratch::new("typed-rejected");
    rejected.write(
        "gband.opt.declare('count', { type = 'integer', default = 1 })\ngband.opt.count = 1.5",
    );
    let config = rejected.loaded();
    error_naming(&config.errors, "count");
}

#[test]
fn invalid_declarations() {
    for (index, call) in [
        "gband.opt.declare('x', { type = 'table', default = {} })",
        "gband.opt.declare('x', { type = 'string', default = 5 })",
        "gband.opt.declare('x', { type = 'string' })",
        "gband.opt.declare('x', { type = 'string', default = 'c', values = { 'a', 'b' } })",
        "gband.opt.declare('x', { type = 'string', default = 'a', values = { 1 } })",
        "gband.opt.declare('x', { type = 'string', default = 'a', desc = 5 })",
        "gband.opt.declare('prefix', { type = 'string', default = 'a' })",
        "gband.opt.declare('x', { type = 'string', default = 'a' }) gband.opt.declare('x', { type = 'string', default = 'a' })",
        "gband.opt.declare('', { type = 'string', default = 'a' })",
        "gband.opt.declare('x', 'string')",
    ]
    .into_iter()
    .enumerate()
    {
        let scratch = Scratch::new(&format!("bad-declare-{index}"));
        let path = scratch.write(&format!("\n{call}"));
        let error = scratch.load().err().unwrap_or_else(|| panic!("{call} loaded"));
        assert_error_at(&error, &path, 2, "");
    }
}

#[test]
fn list_holds_built_in_and_declared_options() {
    let scratch = Scratch::new("list");
    scratch.write(
        "gband.opt.declare('greeting', { type = 'string', default = 'hello', desc = 'what to say' })\ngband.opt.greeting = 'hi'",
    );
    let config = scratch.loaded();
    let names: Vec<String> = eval(
        &config,
        "local names = {} for _, o in ipairs(gband.opt.list()) do names[#names + 1] = o.name end return names",
    );
    assert_eq!(
        names,
        [
            "center_focused_column",
            "floating_border_chars",
            "floating_border_sides",
            "greeting",
            "height_step",
            "loop_bands",
            "notify_style",
            "prefix",
            "tile_border_chars",
            "tile_border_sides",
            "width_step",
        ]
    );
    let greeting: Vec<String> = eval(
        &config,
        "for _, o in ipairs(gband.opt.list()) do if o.name == 'greeting' then return { o.type, o.default, o.value, o.desc } end end",
    );
    assert_eq!(greeting, ["string", "hello", "hi", "what to say"]);
    let prefix: Vec<String> = eval(
        &config,
        "for _, o in ipairs(gband.opt.list()) do if o.name == 'prefix' then return { o.type, o.default, o.value, o.desc } end end",
    );
    assert_eq!(prefix[1], "ctrl+space");
    assert!(!prefix[3].is_empty());
    let server = Scratch::new("list-server");
    let config = server.loaded_server();
    let names: Vec<String> = eval(
        &config,
        "local names = {} for _, o in ipairs(gband.opt.list()) do names[#names + 1] = o.name end return names",
    );
    assert_eq!(names, ["default_column_width", "width_presets"]);
}

#[test]
fn options_are_set_only_while_loading() {
    let scratch = Scratch::new("late");
    let path = scratch.write(
        "gband.opt.declare('x', { type = 'string', default = 'a' })
gband.bind('alt+a', function()
  gband.opt.x = 'b'
end)
gband.bind('alt+b', function()
  gband.opt.declare('y', { type = 'string', default = 'a' })
end)",
    );
    let config = scratch.loaded();
    let root = &config.keymap["root"];
    for (index, line) in [(0, 3), (1, 6)] {
        let Binding::Callback(callback) = root[index].1 else {
            panic!("{root:?}");
        };
        let outcome = config.runtime.call(callback);
        assert_error_at(
            &outcome.errors[0],
            &path,
            line,
            "while the configuration loads",
        );
    }
}

#[test]
fn plugin_option_errors_name_the_plugin() {
    let scratch = Scratch::new("plugin-error");
    let file = scratch.client_plugin(
        "hello",
        "\ngband.opt.statusline_height = 'tall'\nafter = true",
    );
    let config = scratch.loaded();
    let error = error_naming(&config.errors, "statusline_height");
    assert_eq!(
        error.plugin.as_deref(),
        Some("hello"),
        "{:?}",
        config.errors
    );
    assert_error_at(error, &file, 2, "statusline_height");
    assert!(global::<bool>(&config, "after"));
}

#[test]
fn invalid_status_line_height() {
    let scratch = Scratch::new("statusline-height");
    let path = scratch.write("\ngband.opt.statusline_height = 2\nafter = true");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "after"));
    let error = error_naming(&config.errors, "statusline_height");
    assert_error_at(error, &path, 2, "statusline_height");
    assert!(error.message.contains("gband.statusline"), "{error}");
}

#[test]
fn removed_status_line_options_through_gband_set_load_anyway() {
    let scratch = Scratch::new("statusline-set");
    let path = scratch.write(
        "gband.set { statusline_position = 'top', loop_bands = false }\ngband.set { statusline_separator = ' | ' }\nafter = true",
    );
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "after"));
    assert!(!config.options.loop_bands);
    assert_error_at(
        error_naming(&config.errors, "statusline_position"),
        &path,
        1,
        "statusline_position",
    );
    assert_error_at(
        error_naming(&config.errors, "statusline_separator"),
        &path,
        2,
        "statusline_separator",
    );
    assert_eq!(config.errors.len(), 2, "{:?}", config.errors);
}

#[test]
fn removed_status_line_options_read_nil() {
    let scratch = Scratch::new("statusline-read");
    scratch.write("missing = gband.opt.statusline_position == nil");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "missing"));
}

#[test]
fn server_option_assigned_in_the_client_file() {
    let scratch = Scratch::new("foreign-assign");
    let path = scratch
        .write("\ngband.opt.default_column_width = 1/3\nwidth = gband.opt.default_column_width");
    let config = scratch.loaded();
    let error = error_naming(&config.errors, "default_column_width");
    assert_error_at(error, &path, 2, "server");
    assert!(global::<Option<f64>>(&config, "width").is_none());
}

#[test]
fn notify_style() {
    let scratch = Scratch::new("notify-style");
    scratch.write("before = gband.opt.notify_style\ngband.opt.notify_style = 'osc777'");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "before"), "osc9");
    assert_eq!(config.options.notify_style, gband_lua::NotifyStyle::Osc777);
    let rejected = Scratch::new("notify-style-rejected");
    rejected.write("gband.opt.notify_style = 'loud'");
    let config = rejected.loaded();
    assert_eq!(config.options.notify_style, gband_lua::NotifyStyle::Osc9);
    error_naming(&config.errors, "notify_style");
}

#[test]
fn looping_bands_by_default() {
    let scratch = Scratch::new("loop-bands-default");
    scratch.write("looping = gband.opt.loop_bands");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "looping"));
    assert!(config.options.loop_bands);
}

#[test]
fn looping_bands_turned_off() {
    let scratch = Scratch::new("loop-bands-off");
    scratch.write("\n\ngband.set { loop_bands = false }\nlooping = gband.opt.loop_bands");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert!(!global::<bool>(&config, "looping"));
    assert!(!config.options.loop_bands);
}

#[test]
fn looping_bands_of_the_wrong_type() {
    let scratch = Scratch::new("loop-bands-wrong-type");
    let path = scratch.write("gband.opt.loop_bands = false\n\ngband.opt.loop_bands = 'yes'");
    let config = scratch.loaded();
    assert!(config.options.loop_bands);
    let error = error_naming(&config.errors, "loop_bands");
    assert_error_at(error, &path, 3, "loop_bands");
}

#[test]
fn steps_default_to_a_tenth() {
    let scratch = Scratch::new("steps-default");
    scratch.write("width = gband.opt.width_step\nheight = gband.opt.height_step");
    let config = scratch.loaded();
    assert_eq!(global::<f64>(&config, "width"), 0.1);
    assert_eq!(global::<f64>(&config, "height"), 0.1);
    assert_eq!(config.options.steps, Steps::default());
}

#[test]
fn steps_are_set() {
    let scratch = Scratch::new("steps-set");
    scratch.write("gband.set { width_step = 1/20 }\ngband.opt.height_step = 0.25\nwidth = gband.opt.width_step");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(config.options.steps.width, Proportion::new(1, 20));
    assert_eq!(config.options.steps.height, Proportion::new(1, 4));
    assert_eq!(global::<f64>(&config, "width"), 0.05);
}

#[test]
fn invalid_step() {
    let scratch = Scratch::new("invalid-step");
    let path = scratch
        .write("gband.opt.width_step = 1/4\ngband.opt.width_step = 0\ngband.opt.height_step = 2");
    let config = scratch.loaded();
    assert_eq!(config.options.steps, Steps::default());
    let error = error_naming(&config.errors, "width_step");
    assert_error_at(error, &path, 2, "width_step");
    let error = error_naming(&config.errors, "height_step");
    assert_error_at(error, &path, 3, "height_step");
}

#[test]
fn border_options_default_to_every_plain_side() {
    let scratch = Scratch::new("borders-default");
    scratch.write("sides = gband.opt.tile_border_sides\nchars = gband.opt.floating_border_chars");
    let config = scratch.loaded();
    let sides: Vec<String> = global(&config, "sides");
    assert_eq!(sides, ["top", "right", "bottom", "left"]);
    assert_eq!(global::<String>(&config, "chars"), "plain");
    assert_eq!(config.options.tile_border, Border::default());
    assert_eq!(config.options.floating_border, Border::default());
}

#[test]
fn sides_are_held_in_order() {
    let scratch = Scratch::new("sides-order");
    scratch.write("gband.opt.tile_border_sides = { 'left', 'top', 'left' }\nsides = gband.opt.tile_border_sides\ngband.opt.floating_border_sides = {}\nnone = #gband.opt.floating_border_sides");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let sides: Vec<String> = global(&config, "sides");
    assert_eq!(sides, ["top", "left"]);
    assert_eq!(global::<i64>(&config, "none"), 0);
    assert_eq!(config.options.floating_border.sides, Sides::NONE);
}

#[test]
fn custom_characters_read_back() {
    let scratch = Scratch::new("custom-chars");
    scratch.write("gband.opt.tile_border_chars = { '+', '-', '+', '|', '+', '-', '+', '|' }\nchars = gband.opt.tile_border_chars\ngband.opt.floating_border_chars = 'double'\nnamed = gband.opt.floating_border_chars");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let chars: Vec<String> = global(&config, "chars");
    assert_eq!(chars, ["+", "-", "+", "|", "+", "-", "+", "|"]);
    assert_eq!(global::<String>(&config, "named"), "double");
    assert_eq!(
        config.options.tile_border.chars.glyphs(),
        ["+", "-", "+", "|", "+", "-", "+", "|"]
    );
    assert_eq!(
        config.options.floating_border.chars,
        BorderChars::Named(CharSet::Double)
    );
}

#[test]
fn wide_character_rejected() {
    let scratch = Scratch::new("wide-chars");
    let path = scratch.write("gband.opt.tile_border_chars = 'rounded'\ngband.opt.tile_border_chars = { '日', '-', '+', '|', '+', '-', '+', '|' }\ngband.opt.floating_border_sides = { 'middle' }");
    let config = scratch.loaded();
    assert_eq!(config.options.tile_border, Border::default());
    let error = error_naming(&config.errors, "tile_border_chars");
    assert_error_at(error, &path, 2, "tile_border_chars");
    let error = error_naming(&config.errors, "floating_border_sides");
    assert_error_at(error, &path, 3, "middle");
}
