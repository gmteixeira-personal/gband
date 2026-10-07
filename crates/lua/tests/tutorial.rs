mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use common::docs::{WALK, mentions};
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

const CORE: &str = r#"return { { "gband.core", gband.core } }"#;

const UNCOVERED: &str = "defaults/lua/gband/theme/catppuccin.lua";

const COVERED_COLORSCHEME: &str = "defaults/colors/gruvbox.lua";

const CLOSERS: [&str; 5] = ["end", "}", ")", "else", "until"];

struct Tutorial {
    docs: PathBuf,
    examples: PathBuf,
    every_chapter: bool,
}

impl Tutorial {
    fn scripting() -> Self {
        Self {
            docs: root().join("docs/tutorial"),
            examples: root().join("examples/tutorial"),
            every_chapter: true,
        }
    }

    fn internals() -> Self {
        Self {
            docs: root().join("docs/internals"),
            examples: root().join("examples/internals"),
            every_chapter: false,
        }
    }

    fn reader_code(&self, block: &Block) -> bool {
        let mut words = block.info.split_whitespace();
        words.next() == Some("lua") && (self.every_chapter || words.next().is_none())
    }

    fn needs_directory(&self, chapter: &Chapter) -> bool {
        self.every_chapter
            || blocks(&chapter.text)
                .iter()
                .any(|block| self.reader_code(block) || block.info.starts_with("screen"))
    }
}

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
    let Ok(entries) = fs::read_dir(examples) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
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

fn unmatched(tutorial: &Tutorial) -> Vec<String> {
    let Tutorial { docs, examples, .. } = tutorial;
    let chapters = chapters(docs);
    let directories = directories(examples);
    let mut problems: Vec<String> = chapters
        .iter()
        .filter(|chapter| tutorial.needs_directory(chapter))
        .filter(|chapter| !directories.contains(&chapter.slug))
        .map(|chapter| {
            format!(
                "{} has no directory {}",
                chapter.path.display(),
                examples.join(&chapter.slug).display()
            )
        })
        .collect();
    problems.extend(
        directories
            .iter()
            .filter(|name| !chapters.iter().any(|chapter| chapter.slug == **name))
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

fn stale_lua(tutorial: &Tutorial) -> Vec<String> {
    let mut problems = Vec::new();
    for chapter in chapters(&tutorial.docs) {
        let directory = tutorial.examples.join(&chapter.slug);
        let sources: Vec<String> = files(&directory)
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .collect();
        for block in blocks(&chapter.text) {
            if !tutorial.reader_code(&block) {
                continue;
            }
            if !sources.iter().any(|source| source.contains(&block.text)) {
                problems.push(format!(
                    "{}: no file of {} holds the lua block starting `{}`",
                    chapter.path.display(),
                    directory.display(),
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

fn stale_screens(tutorial: &Tutorial) -> Vec<String> {
    let mut problems = Vec::new();
    for chapter in chapters(&tutorial.docs) {
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
            let path = tutorial.examples.join(&chapter.slug).join(reference);
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

fn bundled(name: &str) -> BTreeMap<String, String> {
    let scratch = gband_scratch::Scratch::new("lua", &format!("tutorial-bundled-{name}"));
    let dir = scratch.join("config");
    gband_lua::prepare(&dir).unwrap();
    files(&dir.join("defaults"))
        .into_iter()
        .map(|path| {
            let relative = path.strip_prefix(&dir).unwrap().to_string_lossy();
            (relative.into_owned(), fs::read_to_string(&path).unwrap())
        })
        .collect()
}

struct Quote {
    chapter: PathBuf,
    path: String,
    start: usize,
    len: usize,
}

fn positions(file: &[&str], quote: &[&str]) -> Vec<usize> {
    if quote.is_empty() || quote.len() > file.len() {
        return Vec::new();
    }
    (0..=file.len() - quote.len())
        .filter(|&at| file[at..at + quote.len()] == *quote)
        .collect()
}

fn quotes(docs: &Path, bundled: &BTreeMap<String, String>) -> (Vec<Quote>, Vec<String>) {
    let mut found = Vec::new();
    let mut problems = Vec::new();
    for chapter in chapters(docs) {
        for block in blocks(&chapter.text) {
            let mut words = block.info.split_whitespace();
            let (Some("lua"), Some(path)) = (words.next(), words.next()) else {
                continue;
            };
            let first = block.text.lines().next().unwrap_or("");
            let Some(text) = bundled.get(path) else {
                problems.push(format!(
                    "{}: the quote starting `{first}` names {path}, which gband does not write",
                    chapter.path.display()
                ));
                continue;
            };
            let file: Vec<&str> = text.lines().collect();
            let quote: Vec<&str> = block.text.lines().collect();
            match positions(&file, &quote)[..] {
                [start] => found.push(Quote {
                    chapter: chapter.path.clone(),
                    path: path.to_owned(),
                    start,
                    len: quote.len(),
                }),
                [] => problems.push(format!(
                    "{}: the quote of {path} starting `{first}` matches no run of its lines",
                    chapter.path.display()
                )),
                _ => problems.push(format!(
                    "{}: the quote of {path} starting `{first}` matches more than one run of its lines",
                    chapter.path.display()
                )),
            }
        }
    }
    (found, problems)
}

fn covered(path: &str) -> bool {
    path != UNCOVERED && (!path.starts_with("defaults/colors/") || path == COVERED_COLORSCHEME)
}

fn statement<'a>(lines: &[&'a str], index: usize) -> &'a str {
    lines[..=index]
        .iter()
        .rev()
        .find(|line| {
            line.starts_with(|c: char| !c.is_whitespace())
                && !CLOSERS.iter().any(|closer| line.starts_with(closer))
        })
        .copied()
        .unwrap_or(lines[index])
}

fn uncovered(bundled: &BTreeMap<String, String>, quotes: &[Quote]) -> Vec<String> {
    let mut holders: BTreeMap<(&str, usize), Vec<&Path>> = BTreeMap::new();
    for quote in quotes {
        for line in quote.start..quote.start + quote.len {
            holders
                .entry((quote.path.as_str(), line))
                .or_default()
                .push(&quote.chapter);
        }
    }
    let mut problems = Vec::new();
    for ((path, index), chapters) in &holders {
        if let [first, second, ..] = chapters[..] {
            problems.push(format!(
                "{path}:{} is quoted in both {} and {}",
                index + 1,
                first.display(),
                second.display()
            ));
        }
    }
    for (path, text) in bundled.iter().filter(|(path, _)| covered(path)) {
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if line.trim().is_empty() || holders.contains_key(&(path.as_str(), index)) {
                continue;
            }
            problems.push(format!(
                "{path}:{} `{line}` is quoted in no chapter, in the statement starting `{}`",
                index + 1,
                statement(&lines, index)
            ));
        }
    }
    problems
}

fn prose(text: &str) -> String {
    let mut fenced = false;
    text.lines()
        .filter(|line| {
            if line.starts_with("```") {
                fenced = !fenced;
                return false;
            }
            !fenced
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn unnamed(docs: &Path, bundled: &BTreeMap<String, String>) -> Vec<String> {
    let text = prose(&joined(docs));
    bundled
        .keys()
        .filter(|path| covered(path) && !mentions(&text, path))
        .map(|path| format!("{path} is named in no chapter"))
        .collect()
}

fn primitives(name: &str) -> Vec<String> {
    let client = Scratch::new(&format!("{name}-core"));
    client.write("");
    let config = client.loaded();
    let lua = config.runtime.lua();
    let roots: mlua::Table = lua.load(CORE).eval().unwrap();
    lua.load(WALK).call(roots).unwrap()
}

fn untaught_primitives(primitives: &[String], docs: &Path) -> Vec<String> {
    let text = prose(&joined(docs));
    primitives
        .iter()
        .filter(|path| !mentions(&text, path))
        .cloned()
        .collect()
}

fn assert_none(problems: Vec<String>) {
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

fn scratch_tree(name: &str, every_chapter: bool) -> (gband_scratch::Scratch, Tutorial) {
    let scratch = gband_scratch::Scratch::new("lua", &format!("tutorial-{name}"));
    let tutorial = Tutorial {
        docs: scratch.join("docs"),
        examples: scratch.join("examples"),
        every_chapter,
    };
    fs::create_dir_all(&tutorial.docs).unwrap();
    fs::create_dir_all(&tutorial.examples).unwrap();
    (scratch, tutorial)
}

#[test]
fn every_chapter_has_a_directory() {
    let tutorial = Tutorial::scripting();
    let chapters = chapters(&tutorial.docs);
    assert!(chapters.first().is_some_and(|c| c.slug == "00-setup"));
    assert!(chapters.last().is_some_and(|c| c.slug == "11-testing"));
    assert_eq!(chapters.len(), 12);
    assert_none(unmatched(&tutorial));
}

#[test]
fn every_lua_block_is_in_its_directory() {
    assert_none(stale_lua(&Tutorial::scripting()));
}

#[test]
fn every_screen_block_equals_its_reference() {
    assert_none(stale_screens(&Tutorial::scripting()));
}

#[test]
fn every_api_field_is_the_api_version() {
    let (count, problems) = wrong_api(&Tutorial::scripting().examples);
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
    assert_none(untaught(&fields, &joined(&Tutorial::scripting().docs)));
}

#[test]
fn a_chapter_without_a_directory_is_reported() {
    let (_scratch, tutorial) = scratch_tree("unmatched", true);
    write(&tutorial.docs.join("12-more.md"), "# More\n");
    write(&tutorial.docs.join("README.md"), "# Index\n");
    fs::create_dir_all(tutorial.examples.join("13-orphan")).unwrap();
    let problems = unmatched(&tutorial);
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].contains("docs/12-more.md"), "{problems:?}");
    assert!(problems[1].contains("13-orphan"), "{problems:?}");
}

#[test]
fn a_stale_lua_block_is_reported() {
    let (_scratch, tutorial) = scratch_tree("stale-lua", true);
    write(
        &tutorial.docs.join("04-events.md"),
        "```lua\nlocal kept = 1\n```\n\n```lua\ngband.on(\"Gone\", print)\nlocal x = 2\n```\n\n```sh\nnot checked\n```\n",
    );
    write(
        &tutorial.examples.join("04-events/user/init.lua"),
        "local kept = 1\nlocal x = 2\n",
    );
    let problems = stale_lua(&tutorial);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("docs/04-events.md"), "{problems:?}");
    assert!(
        problems[0].contains("`gband.on(\"Gone\", print)`"),
        "{problems:?}"
    );
}

#[test]
fn a_changed_screen_is_reported() {
    let (_scratch, tutorial) = scratch_tree("stale-screen", true);
    let reference = "tests/screenshots/bars_spec/two.txt";
    write(
        &tutorial.examples.join("08-bars").join(reference),
        "size 10x2 cursor 0:0 shown\n0|I abc\n1|\n--\n0:0 bold\n",
    );
    write(
        &tutorial.docs.join("08-bars.md"),
        &format!(
            "```screen {reference}\nI abc\n\n```\n\n```screen {reference}\nI abd\n\n```\n\n```screen tests/missing.txt\nI\n```\n"
        ),
    );
    let problems = stale_screens(&tutorial);
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
    let (_scratch, tutorial) = scratch_tree("api", true);
    let examples = tutorial.examples;
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
    let text = joined(&Tutorial::scripting().docs).replace("gband.sessions", "");
    let missing = untaught(&fields("untaught"), &text);
    assert_eq!(missing, ["gband.sessions"]);
}

#[test]
fn every_internals_chapter_with_reader_code_has_a_directory() {
    let tutorial = Tutorial::internals();
    let chapters = chapters(&tutorial.docs);
    assert!(chapters.first().is_some_and(|c| c.slug == "00-boundary"));
    assert!(chapters.last().is_some_and(|c| c.slug == "12-own-prelude"));
    assert_eq!(chapters.len(), 13);
    assert_none(unmatched(&tutorial));
}

#[test]
fn every_internals_lua_block_is_in_its_directory() {
    assert_none(stale_lua(&Tutorial::internals()));
}

#[test]
fn every_internals_screen_block_equals_its_reference() {
    assert_none(stale_screens(&Tutorial::internals()));
}

#[test]
fn every_quote_matches_one_run_of_its_file() {
    let (quotes, problems) = quotes(&Tutorial::internals().docs, &bundled("quoted"));
    assert!(!quotes.is_empty());
    assert_none(problems);
}

#[test]
fn every_covered_line_is_quoted_once() {
    let bundled = bundled("covered");
    let (quotes, problems) = quotes(&Tutorial::internals().docs, &bundled);
    assert_none(problems);
    assert_none(uncovered(&bundled, &quotes));
}

#[test]
fn every_covered_file_is_named() {
    assert_none(unnamed(&Tutorial::internals().docs, &bundled("named")));
}

#[test]
fn every_primitive_is_taught() {
    let primitives = primitives("taught");
    for primitive in ["gband.core.provide", "gband.core.palette.set"] {
        assert!(primitives.contains(&primitive.to_owned()), "{primitives:?}");
    }
    assert_none(untaught_primitives(
        &primitives,
        &Tutorial::internals().docs,
    ));
}

#[test]
fn a_plain_lua_block_needs_a_directory() {
    let (_scratch, tutorial) = scratch_tree("internals-unmatched", false);
    write(
        &tutorial.docs.join("02-highlights.md"),
        "```lua defaults/lua/gband/hl.lua\nlocal x\n```\n\n```text\nfree\n```\n",
    );
    write(
        &tutorial.docs.join("04-bars.md"),
        "```lua\nlocal mine = 1\n```\n",
    );
    write(
        &tutorial.docs.join("06-window-provider.md"),
        "```screen tests/one.txt\nrow\n```\n",
    );
    fs::create_dir_all(tutorial.examples.join("13-orphan")).unwrap();
    let problems = unmatched(&tutorial);
    assert_eq!(problems.len(), 3, "{problems:?}");
    assert!(problems[0].contains("docs/04-bars.md"), "{problems:?}");
    assert!(
        problems[1].contains("docs/06-window-provider.md"),
        "{problems:?}"
    );
    assert!(problems[2].contains("13-orphan"), "{problems:?}");
    let stale = stale_lua(&tutorial);
    assert_eq!(stale.len(), 1, "{stale:?}");
    assert!(stale[0].contains("`local mine = 1`"), "{stale:?}");
}

#[test]
fn a_stale_ambiguous_or_unknown_quote_is_reported() {
    let (_scratch, tutorial) = scratch_tree("quotes", false);
    let bundled = bundled("quotes");
    let bar: Vec<&str> = bundled["defaults/lua/gband/bar.lua"].lines().collect();
    write(
        &tutorial.docs.join("04-bars.md"),
        &format!(
            "```lua defaults/lua/gband/bar.lua\n{}\n```\n\n```lua defaults/lua/gband/bar.lua\n{}\nlocal gone = true\n```\n",
            bar[..12].join("\n"),
            bar[0]
        ),
    );
    write(
        &tutorial.docs.join("05-plugin-windows.md"),
        "```lua defaults/lua/gband/win.lua\n  end\n```\n\n```lua defaults/lua/gband/menu.lua\nlocal menu = {}\n```\n",
    );
    let (quotes, problems) = quotes(&tutorial.docs, &bundled);
    assert_eq!(quotes.len(), 1);
    assert_eq!((quotes[0].start, quotes[0].len), (0, 12));
    assert_eq!(problems.len(), 3, "{problems:?}");
    assert!(problems[0].contains("docs/04-bars.md"), "{problems:?}");
    assert!(
        problems[0].contains("defaults/lua/gband/bar.lua"),
        "{problems:?}"
    );
    assert!(
        problems[0].contains(&format!("`{}`", bar[0])),
        "{problems:?}"
    );
    assert!(problems[0].contains("matches no run"), "{problems:?}");
    assert!(
        problems[1].contains("docs/05-plugin-windows.md"),
        "{problems:?}"
    );
    assert!(
        problems[1].contains("defaults/lua/gband/win.lua"),
        "{problems:?}"
    );
    assert!(problems[1].contains("`  end`"), "{problems:?}");
    assert!(problems[1].contains("more than one"), "{problems:?}");
    assert!(
        problems[2].contains("defaults/lua/gband/menu.lua"),
        "{problems:?}"
    );
    assert!(problems[2].contains("does not write"), "{problems:?}");
}

#[test]
fn an_uncovered_or_twice_quoted_line_is_reported() {
    let (_scratch, tutorial) = scratch_tree("coverage", false);
    let mut bundled = bundled("coverage");
    bundled.retain(|path, _| {
        [
            "defaults/lua/gband/palette.lua",
            "defaults/colors/nord.lua",
            UNCOVERED,
        ]
        .contains(&path.as_str())
    });
    let palette: Vec<&str> = bundled["defaults/lua/gband/palette.lua"].lines().collect();
    let skipped = palette
        .iter()
        .position(|line| *line == "  core.palette.set(parsed)")
        .unwrap();
    write(
        &tutorial.docs.join("02-highlights.md"),
        &format!(
            "```lua defaults/lua/gband/palette.lua\n{}\n```\n\n```lua defaults/lua/gband/palette.lua\n{}\n```\n",
            palette[..skipped].join("\n"),
            palette[skipped + 1..].join("\n")
        ),
    );
    write(
        &tutorial.docs.join("03-colorschemes.md"),
        &format!(
            "```lua defaults/lua/gband/palette.lua\n{}\n```\n",
            palette[0]
        ),
    );
    let (quotes, problems) = quotes(&tutorial.docs, &bundled);
    assert_none(problems);
    let problems = uncovered(&bundled, &quotes);
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(
        problems[0].starts_with("defaults/lua/gband/palette.lua:1 "),
        "{problems:?}"
    );
    assert!(
        problems[0].contains("docs/02-highlights.md"),
        "{problems:?}"
    );
    assert!(
        problems[0].contains("docs/03-colorschemes.md"),
        "{problems:?}"
    );
    assert!(
        problems[1].starts_with(&format!(
            "defaults/lua/gband/palette.lua:{} `{}`",
            skipped + 1,
            palette[skipped]
        )),
        "{problems:?}"
    );
    assert!(
        problems[1].contains("`function gband.palette.set(spec)`"),
        "{problems:?}"
    );
}

#[test]
fn an_unnamed_covered_file_is_reported() {
    let (_scratch, tutorial) = scratch_tree("naming", false);
    let mut bundled = bundled("naming");
    bundled.retain(|path, _| path.starts_with("defaults/colors/"));
    assert!(bundled.contains_key("defaults/colors/nord.lua"));
    write(
        &tutorial.docs.join("01-themes.md"),
        "Only the quote names it.\n\n```lua defaults/colors/gruvbox.lua\n-- defaults/colors/gruvbox.lua\n```\n",
    );
    assert_eq!(
        unnamed(&tutorial.docs, &bundled),
        ["defaults/colors/gruvbox.lua is named in no chapter"]
    );
    write(
        &tutorial.docs.join("01-themes.md"),
        "`defaults/colors/gruvbox.lua` is the theme.\n",
    );
    assert_none(unnamed(&tutorial.docs, &bundled));
}

#[test]
fn a_primitive_only_quoted_is_reported() {
    let (_scratch, tutorial) = scratch_tree("primitives", false);
    let primitives = primitives("quoted");
    let named: Vec<String> = primitives
        .iter()
        .filter(|path| *path != "gband.core.timer")
        .map(|path| format!("- `{path}`"))
        .collect();
    write(
        &tutorial.docs.join("00-boundary.md"),
        &format!(
            "{}\n\n```lua defaults/lua/gband/bar.lua\ngband.core.timer(1, f)\n```\n",
            named.join("\n")
        ),
    );
    assert_eq!(
        untaught_primitives(&primitives, &tutorial.docs),
        ["gband.core.timer"]
    );
}
