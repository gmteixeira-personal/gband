#!/usr/bin/env python3
"""Generate this change's delta specs from the current main specs.

  python3 rename_specs.py delta   rewrite specs/ from openspec/specs/
  python3 rename_specs.py main    rename scenario titles and Purpose sections in openspec/specs/
  python3 rename_specs.py check   list main-spec lines the map would still change

The deltas hold every requirement whose text the map changes, and the ADDED
requirements kept in added/<capability>.md. Scenario titles stay
as the main specs have them until `main` runs, because OpenSpec refuses to archive a
MODIFIED block that lacks a scenario title of the main spec.
"""
import re
import sys
from pathlib import Path

CHANGE = Path(__file__).resolve().parent
ROOT = CHANGE.parents[2]
MAIN = ROOT / "openspec" / "specs"
DELTA = CHANGE / "specs"
ADDED = CHANGE / "added"

PLUGIN_WINDOW_SPECS = {"plugin-windows", "lua-control", "client-attach"}

KEEP = [
    "plugin-windows",
    "terminal emulator window",
    "niri window manager",
]

PLUGIN_WINDOW_TERMS = [
    (r"\bPlugin pane windows\b", "Tiled plugin windows"),
    (r"\bPane windows\b", "Tiled plugin windows"),
    (r"\bPane window\b", "Tiled plugin window"),
    (r"\bpane windows\b", "tiled plugin windows"),
    (r"\bpane window\b", "tiled plugin window"),
    (r"\bPlugin panes\b", "Drawn windows"),
    (r"\bPlugin pane\b", "Drawn window"),
    (r"\bplugin panes\b", "drawn windows"),
    (r"\bplugin pane\b", "drawn window"),
    (r'kind `"pane"`', 'kind `"tiled"`'),
    (r'`"float"` or `"pane"`', '`"float"` or `"tiled"`'),
    (r'kind = "pane"', 'kind = "tiled"'),
]

BARE_PLUGIN_WINDOW = [
    (r"`WindowCursorLine`", "`PluginWindowCursorLine`"),
    (r"`WindowBorder`", "`PluginWindowBorder`"),
    (r"`WindowTitle`", "`PluginWindowTitle`"),
    (r"`Window`", "`PluginWindow`"),
    (r"`window`", "`plugin_window`"),
    (r"(?<![-\w])Windows\b", "Plugin \x02s"),
    (r"(?<![-\w])Window\b", "Plugin \x02"),
    (r"(?<![-\w])windows\b", "plugin \x02s"),
    (r"(?<![-\w])window\b", "plugin \x02"),
]

IDENTIFIERS = [
    ("gband.pane_state", "gband.window_state"),
    ("pane_state", "window_state"),
    ("gband.pane", "gband.window"),
    ("PaneStateChanged", "WindowStateChanged"),
    ("PaneOpened", "WindowOpened"),
    ("PaneClosed", "WindowClosed"),
    ("PaneExited", "WindowExited"),
    ("PaneOutput", "WindowOutput"),
    ("PaneInput", "WindowInput"),
    ("PaneMoved", "WindowMoved"),
    ("PaneSegment", "WindowSegment"),
    ("FloatingPane", "FloatingWindow"),
    ("OpenPane", "OpenWindow"),
    ("GBAND_PANE", "GBAND_WINDOW"),
    ("toggle_pane_floating", "toggle_window_floating"),
    ("focus_pane_down", "focus_window_down"),
    ("focus_pane_up", "focus_window_up"),
    ("move_pane_down", "move_window_down"),
    ("move_pane_up", "move_window_up"),
    ("grow_pane_height", "grow_window_height"),
    ("shrink_pane_height", "shrink_window_height"),
    ("reset_pane_height", "reset_window_height"),
    ("open_pane", "open_window"),
    ("close_pane", "close_window"),
    ("pane_spec", "window_spec"),
    ("floating-panes", "floating-windows"),
    ("floating-pane", "floating-window"),
]

WORDS = [
    (r"\bPanes\b", "Windows"),
    (r"\bPane\b", "Window"),
    (r"\bpanes\b", "windows"),
    (r"\bpane\b", "window"),
]


def rename(text, capability):
    held = {}
    for index, phrase in enumerate(KEEP):
        token = f"\x00{index}\x00"
        held[token] = phrase
        text = text.replace(phrase, token)
    marks = {}
    for index, (pattern, replacement) in enumerate(PLUGIN_WINDOW_TERMS):
        token = f"\x01{index}\x01"
        marks[token] = replacement
        text = re.sub(pattern, token, text)
    if capability in PLUGIN_WINDOW_SPECS:
        text = re.sub(r"\b([Pp]lugin) window", "\\1 \x02", text)
        for pattern, replacement in BARE_PLUGIN_WINDOW:
            text = re.sub(pattern, replacement, text)
        text = text.replace("\x02", "window")
    for old, new in IDENTIFIERS:
        text = text.replace(old, new)
    for pattern, replacement in WORDS:
        text = re.sub(pattern, replacement, text)
    for token, replacement in marks.items():
        text = text.replace(token, replacement)
    for token, phrase in held.items():
        text = text.replace(token, phrase)
    return text


def requirements(spec):
    lines = spec.splitlines(keepends=True)
    blocks, current = [], None
    for line in lines:
        if line.startswith("### Requirement:"):
            current = [line]
            blocks.append(current)
        elif line.startswith("## ") and current is not None:
            current = None
        elif current is not None:
            current.append(line)
    return ["".join(block).rstrip("\n") + "\n" for block in blocks]


def title_lines(block):
    return [line for line in block.splitlines() if line.startswith("#### Scenario:")]


def delta(capability, spec):
    renamed, modified = [], []
    for block in requirements(spec):
        new = rename(block, capability)
        if new == block:
            continue
        old_header = block.splitlines()[0]
        new_header = new.splitlines()[0]
        lines = new.splitlines()
        for old_title, index in zip(title_lines(block), [i for i, l in enumerate(lines) if l.startswith("#### Scenario:")]):
            lines[index] = old_title
        new = "\n".join(lines) + "\n"
        if old_header != new_header:
            renamed.append(f"- FROM: `{old_header}`\n- TO: `{new_header}`\n")
        modified.append(new)
    if not modified:
        return None
    parts = []
    if renamed:
        parts.append("## RENAMED Requirements\n\n" + "\n".join(renamed))
    parts.append("## MODIFIED Requirements\n\n" + "\n".join(modified))
    return "\n".join(parts)


def write_deltas():
    for path in sorted(DELTA.glob("*/spec.md")):
        path.unlink()
        path.parent.rmdir()
    for path in sorted(MAIN.glob("*/spec.md")):
        capability = path.parent.name
        text = delta(capability, path.read_text())
        added = ADDED / f"{capability}.md"
        if added.exists():
            text = (text + "\n" if text else "") + "## ADDED Requirements\n\n" + added.read_text()
        if text is None:
            continue
        target = DELTA / capability / "spec.md"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text)
        print(f"{capability}: {text.count('### Requirement:')} blocks")


def rename_main():
    for path in sorted(MAIN.glob("*/spec.md")):
        capability = path.parent.name
        out, in_purpose = [], False
        for line in path.read_text().splitlines(keepends=True):
            if line.startswith("## "):
                in_purpose = line.startswith("## Purpose")
            if line.startswith("#### Scenario:") or (in_purpose and not line.startswith("#")):
                line = rename(line, capability)
            out.append(line)
        path.write_text("".join(out))


def check():
    for path in sorted(MAIN.glob("*/spec.md")):
        capability = path.parent.name
        for number, line in enumerate(path.read_text().splitlines(), 1):
            if rename(line, capability) != line:
                print(f"{path.relative_to(ROOT)}:{number}: {line.strip()[:100]}")


if __name__ == "__main__":
    {"delta": write_deltas, "main": rename_main, "check": check}[sys.argv[1]]()
