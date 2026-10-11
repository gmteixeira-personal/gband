mod common;

use std::collections::BTreeSet;
use std::fs;

use common::docs::{gband_root, walk};
use common::*;
use gband_lua::Config;

const DEFINITIONS: &str = include_str!("../src/runtime/meta/gband.lua");

const PLUGINS: [&str; 5] = [
    "gband.sidebar",
    "gband.keylist",
    "gband.errors",
    "gband.prompt",
    "gband.desktop",
];

fn declaration(line: &str) -> Option<String> {
    let path = match line.strip_prefix("function ") {
        Some(rest) => rest.split_once('(')?.0,
        None => line.split_once(" =")?.0,
    };
    path.starts_with("gband.")
        .then(|| path.replace("[\"", ".").replace("\"]", ""))
}

fn declared() -> BTreeSet<String> {
    DEFINITIONS.lines().filter_map(declaration).collect()
}

fn gband_paths(config: &Config) -> Vec<String> {
    let lua = config.runtime.lua();
    walk(lua, vec![gband_root(lua)])
}

fn plugin_action(path: &str) -> bool {
    path.strip_prefix("gband.action.")
        .is_some_and(|name| name.contains('.'))
}

fn walked(name: &str, init: &str) -> BTreeSet<String> {
    let setup: String = PLUGINS
        .iter()
        .map(|plugin| format!("gband.plugin('{plugin}')\n"))
        .collect();
    let client = Scratch::new(&format!("{name}-client"));
    client.write(&format!("{setup}{init}"));
    let mut paths = gband_paths(&client.loaded());
    assert!(paths.contains(&"gband.action.keylist.open".to_owned()));
    let server = Scratch::new(&format!("{name}-server"));
    server.server("");
    paths.extend(gband_paths(&server.loaded_server()));
    paths
        .into_iter()
        .filter(|path| !plugin_action(path))
        .collect()
}

fn undeclared(walked: &BTreeSet<String>) -> Vec<String> {
    walked.difference(&declared()).cloned().collect()
}

#[test]
fn every_path_declared() {
    let walked = walked("types-declared", "");
    assert!(walked.contains("gband.core.present_window"));
    assert!(walked.contains("gband.sessions"));
    let missing = undeclared(&walked);
    assert!(
        missing.is_empty(),
        "crates/lua/src/runtime/meta/gband.lua does not declare:\n{}",
        missing.join("\n")
    );
}

#[test]
fn no_stale_declaration() {
    let walked = walked("types-stale", "");
    let stale: Vec<String> = declared().difference(&walked).cloned().collect();
    assert!(
        stale.is_empty(),
        "crates/lua/src/runtime/meta/gband.lua declares paths neither side holds:\n{}",
        stale.join("\n")
    );
}

#[test]
fn an_undeclared_field_is_reported() {
    let walked = walked(
        "types-undeclared",
        "gband.core.undeclared_probe = function() end",
    );
    let missing = undeclared(&walked);
    assert!(
        missing.contains(&"gband.core.undeclared_probe".to_owned()),
        "{missing:?}"
    );
}

#[test]
fn definitions_are_not_required() {
    let scratch = Scratch::new("types-require");
    gband_lua::prepare(&scratch.dir()).unwrap();
    scratch.write("require('meta.gband')");
    let error = scratch.load().err().expect("the require fails the load");
    assert!(error.message.contains("meta.gband"), "{error}");
}

#[test]
fn definitions_are_not_loaded() {
    let scratch = Scratch::new("types-loaded");
    gband_lua::prepare(&scratch.dir()).unwrap();
    let definitions = scratch
        .dir()
        .join("defaults")
        .join("meta")
        .join("gband.lua");
    let edited = format!(
        "{}\nerror(\"boom\")\n",
        fs::read_to_string(&definitions).unwrap()
    );
    fs::write(&definitions, edited).unwrap();
    let client = scratch.loaded();
    assert!(client.errors.is_empty(), "{:?}", client.errors);
    let server = scratch.loaded_server();
    assert!(server.errors.is_empty(), "{:?}", server.errors);
}
