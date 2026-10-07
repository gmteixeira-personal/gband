use std::fs;
use std::path::Path;

pub const WALK: &str = r#"
local roots = ...
local paths = {}
local function walk(prefix, value, inside)
  if type(value) ~= "table" or inside[value] then
    return
  end
  inside[value] = true
  for key, field in next, value do
    if type(key) == "string" then
      local path = prefix .. "." .. key
      paths[#paths + 1] = path
      walk(path, field, inside)
    end
  end
  inside[value] = nil
end
for _, root in ipairs(roots) do
  walk(root[1], root[2], {})
end
table.sort(paths)
return paths
"#;

struct Section {
    heading: String,
    body: String,
}

pub struct Docs {
    text: String,
    sections: Vec<Section>,
}

fn identifier(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn mentions(text: &str, needle: &str) -> bool {
    text.match_indices(needle).any(|(at, _)| {
        !text[at + needle.len()..]
            .chars()
            .next()
            .is_some_and(identifier)
    })
}

fn level(line: &str) -> Option<usize> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    (hashes > 0 && line[hashes..].starts_with(' ')).then_some(hashes)
}

impl Docs {
    pub fn read(path: &Path) -> Self {
        let text = fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        let mut sections = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            let Some(depth) = level(line) else {
                continue;
            };
            let body = lines[index + 1..]
                .iter()
                .take_while(|line| level(line).is_none_or(|deeper| deeper > depth))
                .copied()
                .collect::<Vec<_>>()
                .join("\n");
            sections.push(Section {
                heading: (*line).to_owned(),
                body,
            });
        }
        Self { text, sections }
    }

    pub fn documents(&self, path: &str) -> bool {
        if mentions(&self.text, path) {
            return true;
        }
        let Some((parent, last)) = path.rsplit_once('.') else {
            return false;
        };
        self.sections
            .iter()
            .filter(|section| mentions(&section.heading, parent))
            .any(|section| mentions(&section.body, &format!("`{last}")))
    }

    pub fn missing(&self, paths: &[String]) -> Vec<String> {
        paths
            .iter()
            .filter(|path| !self.documents(path))
            .cloned()
            .collect()
    }
}
