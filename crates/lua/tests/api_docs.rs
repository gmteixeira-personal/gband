mod common;

use std::path::Path;

use common::docs::{Docs, WALK};
use common::*;
use gband_lua::Config;

const PRELUDE_MODULES: [&str; 7] = [
    "hl",
    "palette",
    "colorscheme",
    "bar",
    "win",
    "settings",
    "keystyle",
];

fn plugins_docs() -> Docs {
    Docs::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/plugins.md"))
}

fn paths(config: &Config, with_exports: bool) -> Vec<String> {
    let lua = config.runtime.lua();
    let roots = lua.create_table().unwrap();
    let root = |name: String, value: mlua::Value| {
        let entry = lua.create_table().unwrap();
        entry.push(name).unwrap();
        entry.push(value).unwrap();
        roots.push(entry).unwrap();
    };
    root("gband".to_owned(), lua.globals().get("gband").unwrap());
    if with_exports {
        let loaded: mlua::Table = lua
            .globals()
            .get::<mlua::Table>("package")
            .unwrap()
            .get("loaded")
            .unwrap();
        for module in PRELUDE_MODULES {
            let name = format!("gband.{module}");
            root(
                format!("require(\"{name}\")"),
                loaded.get(name.as_str()).unwrap(),
            );
        }
    }
    lua.load(WALK).call(roots).unwrap()
}

fn assert_documented(paths: &[String]) {
    let missing = plugins_docs().missing(paths);
    assert!(
        missing.is_empty(),
        "docs/plugins.md does not describe:\n{}",
        missing.join("\n")
    );
}

#[test]
fn every_client_field_documented() {
    let scratch = Scratch::new("docs-client");
    scratch.write("");
    let config = scratch.loaded();
    let collected = paths(&config, true);
    assert!(collected.contains(&"gband.core.present_window".to_owned()));
    assert!(collected.contains(&"require(\"gband.hl\").drawn".to_owned()));
    assert_documented(&collected);
}

#[test]
fn every_server_field_documented() {
    let scratch = Scratch::new("docs-server");
    scratch.server("");
    let config = scratch.loaded_server();
    assert_documented(&paths(&config, false));
}

#[test]
fn every_client_event_documented() {
    let scratch = Scratch::new("docs-events");
    scratch.write("");
    let config = scratch.loaded();
    let events: Vec<String> = eval(&config, "return gband.core.events");
    let docs = plugins_docs();
    let missing: Vec<&String> = events
        .iter()
        .filter(|event| !docs.documents(&format!("`{event}`")))
        .collect();
    assert!(missing.is_empty(), "undocumented events: {missing:?}");
}

#[test]
fn an_undocumented_field_is_reported() {
    let scratch = Scratch::new("docs-undocumented");
    scratch.write("gband.core.undocumented_probe = function() end");
    let config = scratch.loaded();
    let collected = paths(&config, true);
    let missing = plugins_docs().missing(&collected);
    assert!(
        missing.contains(&"gband.core.undocumented_probe".to_owned()),
        "{missing:?}"
    );
}
