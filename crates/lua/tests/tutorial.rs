mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::docs::mentions;
use common::*;
use gband_lua::{API_VERSION, Config};

const TOP_LEVEL: &str = r#"
local fields = {}
for key in next, gband do
  if type(key) == "string" and key ~= "core" then
    fields[#fields + 1] = key
  end
end
table.sort(fields)
return fields
"#;

struct Chapter {
    slug: String,
    path: PathBuf,
    text: String,
}

struct Block {
    info: String,
    text: String,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn slug(name: &str) -> Option<&str> {
    let stem = name.strip_suffix(".md")?;
    let bytes = stem.as_bytes();
    (bytes.len() > 3 && bytes[..2].iter().all(u8::is_ascii_digit) && bytes[2] == b'-')
        .then_some(stem)
}

fn chapters(docs: &Path) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = fs::read_dir(docs)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let slug = slug(path.file_name()?.to_str()?)?.to_owned();
            let text = fs::read_to_string(&path).unwrap();
            Some(Chapter { slug, path, text })
        })
        .collect();
    chapters.sort_by(|a, b| a.slug.cmp(&b.slug));
    chapters
}

fn directories(examples: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(examples)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn files(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return found;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

fn blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let Some(info) = line.strip_prefix("```") else {
            continue;
        };
        let body: Vec<&str> = lines
            .by_ref()
            .take_while(|line| !line.starts_with("```"))
            .collect();
        blocks.push(Block {
            info: info.trim().to_owned(),
            text: body.join("\n"),
        });
    }
    blocks
}

fn unmatched(docs: &Path, examples: &Path) -> Vec<String> {
    let chapters: Vec<String> = chapters(docs).into_iter().map(|c| c.slug).collect();
    let directories = directories(examples);
    let mut problems: Vec<String> = chapters
        .iter()
        .filter(|slug| !directories.contains(slug))
        .map(|slug| {
            format!(
                "{} has no directory {}",
                docs.join(format!("{slug}.md")).display(),
                examples.join(slug).display()
            )
        })
        .collect();
    problems.extend(
        directories
            .iter()
            .filter(|name| !chapters.contains(name))
            .map(|name| {
                format!(
                    "{} has no chapter {}",
                    examples.join(name).display(),
                    docs.join(format!("{name}.md")).display()
                )
            }),
    );
    problems
}

fn stale_lua(docs: &Path, examples: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    for chapter in chapters(docs) {
        let sources: Vec<String> = files(&examples.join(&chapter.slug))
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .collect();
        for block in blocks(&chapter.text) {
            if block.info.split_whitespace().next() != Some("lua") {
                continue;
            }
            if !sources.iter().any(|source| source.contains(&block.text)) {
                problems.push(format!(
                    "{}: no file of {} holds the lua block starting `{}`",
                    chapter.path.display(),
                    examples.join(&chapter.slug).display(),
                    block.text.lines().next().unwrap_or("")
                ));
            }
        }
    }
    problems
}

fn reference_rows(text: &str) -> String {
    text.lines()
        .skip(1)
        .take_while(|line| *line != "--")
        .map(|line| line.split_once('|').map_or(line, |(_, row)| row))
        .collect::<Vec<_>>()
        .join("\n")
}

fn stale_screens(docs: &Path, examples: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    for chapter in chapters(docs) {
        for block in blocks(&chapter.text) {
            let mut words = block.info.split_whitespace();
            if words.next() != Some("screen") {
                continue;
            }
            let Some(reference) = words.next() else {
                problems.push(format!(
                    "{}: a screen block names no reference",
                    chapter.path.display()
                ));
                continue;
            };
            let path = examples.join(&chapter.slug).join(reference);
            match fs::read_to_string(&path) {
                Err(error) => problems.push(format!(
                    "{}: the screen reference {} cannot be read: {error}",
                    chapter.path.display(),
                    path.display()
                )),
                Ok(text) if reference_rows(&text) != block.text => problems.push(format!(
                    "{}: the screen block differs from {}",
                    chapter.path.display(),
                    path.display()
                )),
                Ok(_) => {}
            }
        }
    }
    problems
}

fn api_fields(text: &str) -> Vec<i64> {
    text.match_indices("api = ")
        .filter(|(at, _)| {
            !text[..*at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        })
        .filter_map(|(at, needle)| {
            let digits: String = text[at + needle.len()..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse().ok()
        })
        .collect()
}

fn wrong_api(examples: &Path) -> (usize, Vec<String>) {
    let mut count = 0;
    let mut problems = Vec::new();
    for path in files(examples) {
        if path.extension().is_none_or(|extension| extension != "lua") {
            continue;
        }
        for api in api_fields(&fs::read_to_string(&path).unwrap()) {
            count += 1;
            if api != API_VERSION {
                problems.push(format!(
                    "{} sets api = {api}, and gband.api_version is {API_VERSION}",
                    path.display()
                ));
            }
        }
    }
    (count, problems)
}

fn top_level(config: &Config) -> Vec<String> {
    eval(config, TOP_LEVEL)
}

fn fields(name: &str) -> Vec<String> {
    let client = Scratch::new(&format!("{name}-client"));
    client.write("");
    let server = Scratch::new(&format!("{name}-server"));
    server.server("");
    let mut fields = top_level(&client.loaded());
    fields.extend(top_level(&server.loaded_server()));
    fields.sort();
    fields.dedup();
    fields
}

fn untaught(fields: &[String], text: &str) -> Vec<String> {
    fields
        .iter()
        .map(|field| format!("gband.{field}"))
        .filter(|path| !mentions(text, path))
        .collect()
}

fn joined(docs: &Path) -> String {
    chapters(docs)
        .into_iter()
        .map(|chapter| chapter.text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn docs() -> PathBuf {
    root().join("docs/tutorial")
}

fn examples() -> PathBuf {
    root().join("examples/tutorial")
}

fn assert_none(problems: Vec<String>) {
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

fn scratch_tree(name: &str) -> (gband_scratch::Scratch, PathBuf, PathBuf) {
    let scratch = gband_scratch::Scratch::new("lua", &format!("tutorial-{name}"));
    let docs = scratch.join("docs");
    let examples = scratch.join("examples");
    fs::create_dir_all(&docs).unwrap();
    fs::create_dir_all(&examples).unwrap();
    (scratch, docs, examples)
}

#[test]
fn every_chapter_has_a_directory() {
    let chapters = chapters(&docs());
    assert!(chapters.first().is_some_and(|c| c.slug == "00-setup"));
    assert!(chapters.last().is_some_and(|c| c.slug == "11-testing"));
    assert_eq!(chapters.len(), 12);
    assert_none(unmatched(&docs(), &examples()));
}

#[test]
fn every_lua_block_is_in_its_directory() {
    assert_none(stale_lua(&docs(), &examples()));
}

#[test]
fn every_screen_block_equals_its_reference() {
    assert_none(stale_screens(&docs(), &examples()));
}

#[test]
fn every_api_field_is_the_api_version() {
    let (count, problems) = wrong_api(&examples());
    assert!(count > 0, "no Lua file under examples/tutorial/ sets api");
    assert_none(problems);
}

#[test]
fn every_namespace_is_taught() {
    let fields = fields("taught");
    for field in ["keymap", "window_state", "sessions"] {
        assert!(fields.contains(&field.to_owned()), "{fields:?}");
    }
    assert!(!fields.contains(&"core".to_owned()));
    assert_none(untaught(&fields, &joined(&docs())));
}

#[test]
fn a_chapter_without_a_directory_is_reported() {
    let (_scratch, docs, examples) = scratch_tree("unmatched");
    write(&docs.join("12-more.md"), "# More\n");
    write(&docs.join("README.md"), "# Index\n");
    fs::create_dir_all(examples.join("13-orphan")).unwrap();
    let problems = unmatched(&docs, &examples);
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].contains("docs/12-more.md"), "{problems:?}");
    assert!(problems[1].contains("13-orphan"), "{problems:?}");
}

#[test]
fn a_stale_lua_block_is_reported() {
    let (_scratch, docs, examples) = scratch_tree("stale-lua");
    write(
        &docs.join("04-events.md"),
        "```lua\nlocal kept = 1\n```\n\n```lua\ngband.on(\"Gone\", print)\nlocal x = 2\n```\n\n```sh\nnot checked\n```\n",
    );
    write(
        &examples.join("04-events/user/init.lua"),
        "local kept = 1\nlocal x = 2\n",
    );
    let problems = stale_lua(&docs, &examples);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("docs/04-events.md"), "{problems:?}");
    assert!(
        problems[0].contains("`gband.on(\"Gone\", print)`"),
        "{problems:?}"
    );
}

#[test]
fn a_changed_screen_is_reported() {
    let (_scratch, docs, examples) = scratch_tree("stale-screen");
    let reference = "tests/screenshots/bars_spec/two.txt";
    write(
        &examples.join("08-bars").join(reference),
        "size 10x2 cursor 0:0 shown\n0|I abc\n1|\n--\n0:0 bold\n",
    );
    write(
        &docs.join("08-bars.md"),
        &format!(
            "```screen {reference}\nI abc\n\n```\n\n```screen {reference}\nI abd\n\n```\n\n```screen tests/missing.txt\nI\n```\n"
        ),
    );
    let problems = stale_screens(&docs, &examples);
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].contains("differs from"), "{problems:?}");
    assert!(problems[0].contains(reference), "{problems:?}");
    assert!(problems[1].contains("tests/missing.txt"), "{problems:?}");
    assert!(
        problems.iter().all(|p| p.contains("docs/08-bars.md")),
        "{problems:?}"
    );
}

#[test]
fn a_raised_api_version_is_reported() {
    let (_scratch, _docs, examples) = scratch_tree("api");
    let stale = examples.join("09-plugin/plugins/marks/lua/marks/init.lua");
    write(
        &stale,
        &format!(
            "local M = {{ name = \"marks\", api = {} }}\n",
            API_VERSION + 1
        ),
    );
    write(
        &examples.join("10-server/plugins/marks/lua/marks/init.lua"),
        &format!("local M = {{ api = {API_VERSION} }}\nprint(gband.api_version)\n"),
    );
    let (count, problems) = wrong_api(&examples);
    assert_eq!(count, 2);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].contains("09-plugin/plugins/marks/lua/marks/init.lua"),
        "{problems:?}"
    );
}

#[test]
fn an_untaught_namespace_is_reported() {
    let text = joined(&docs()).replace("gband.sessions", "");
    let missing = untaught(&fields("untaught"), &text);
    assert_eq!(missing, ["gband.sessions"]);
}
