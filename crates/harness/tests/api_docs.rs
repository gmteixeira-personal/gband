#[path = "../../lua/tests/common/docs.rs"]
mod docs;

use std::path::Path;

use docs::{Docs, WALK};

#[test]
fn every_test_field_documented() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lua = gband_harness::runner::test_side(manifest).unwrap();
    let paths: Vec<String> = lua
        .load(format!(
            "local walk = load({WALK:?})\n\
             return walk({{ {{ 'gband.core', rawget(gband, 'core') }}, \
             {{ 'require(\"gband.test\")', require('gband.test') }} }})"
        ))
        .eval()
        .unwrap();
    assert!(paths.contains(&"gband.core.register".to_owned()));
    assert!(paths.contains(&"gband.core.wrap".to_owned()));
    let missing = Docs::read(&manifest.join("../../docs/testing.md")).missing(&paths);
    assert!(
        missing.is_empty(),
        "docs/testing.md does not describe:\n{}",
        missing.join("\n")
    );
}
