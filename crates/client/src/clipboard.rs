use std::process::Command;

pub struct Image {
    pub mime: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardError {
    NoReader,
    ReaderMissing(String),
    NoImage,
    Reader(String),
}

impl ClipboardError {
    pub fn message(&self) -> String {
        match self {
            ClipboardError::NoReader => {
                "no clipboard reader found; install wl-paste, xclip or pbpaste".to_string()
            }
            ClipboardError::ReaderMissing(tool) => {
                format!("clipboard reader {tool} is not installed")
            }
            ClipboardError::NoImage => "the clipboard holds no image".to_string(),
            ClipboardError::Reader(detail) => {
                format!("reading the clipboard image failed: {detail}")
            }
        }
    }
}

pub trait Reader {
    fn types(&self) -> Result<Vec<String>, ClipboardError>;
    fn read(&self, mime: &str) -> Result<Vec<u8>, ClipboardError>;
}

pub fn pick_mime(types: &[String]) -> Option<String> {
    if types.iter().any(|mime| mime == "image/png") {
        return Some("image/png".to_string());
    }
    types
        .iter()
        .find(|mime| mime.starts_with("image/"))
        .cloned()
}

pub fn image(reader: &dyn Reader) -> Result<Image, ClipboardError> {
    let types = reader.types()?;
    let mime = pick_mime(&types).ok_or(ClipboardError::NoImage)?;
    let data = reader.read(&mime)?;
    if data.is_empty() {
        return Err(ClipboardError::NoImage);
    }
    Ok(Image { mime, data })
}

pub fn read() -> Result<Image, ClipboardError> {
    let reader = select()?;
    image(reader.as_ref())
}

fn select() -> Result<Box<dyn Reader>, ClipboardError> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return tool("wl-paste").map(|_| Box::new(WlPaste) as Box<dyn Reader>);
    }
    if std::env::var_os("DISPLAY").is_some() {
        return tool("xclip").map(|_| Box::new(Xclip) as Box<dyn Reader>);
    }
    if cfg!(target_os = "macos") {
        return tool("pbpaste").map(|_| Box::new(PbPaste) as Box<dyn Reader>);
    }
    Err(ClipboardError::NoReader)
}

fn tool(name: &str) -> Result<(), ClipboardError> {
    let found = std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()));
    if found {
        Ok(())
    } else {
        Err(ClipboardError::ReaderMissing(name.to_string()))
    }
}

fn run(command: &mut Command) -> Result<Vec<u8>, ClipboardError> {
    let output = command
        .output()
        .map_err(|error| ClipboardError::Reader(error.to_string()))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ClipboardError::Reader(detail));
    }
    Ok(output.stdout)
}

fn offered(bytes: Vec<u8>) -> Vec<String> {
    String::from_utf8_lossy(&bytes)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

struct WlPaste;

impl Reader for WlPaste {
    fn types(&self) -> Result<Vec<String>, ClipboardError> {
        Ok(offered(run(Command::new("wl-paste").arg("--list-types"))?))
    }

    fn read(&self, mime: &str) -> Result<Vec<u8>, ClipboardError> {
        run(Command::new("wl-paste")
            .arg("--no-newline")
            .arg("--type")
            .arg(mime))
    }
}

struct Xclip;

impl Reader for Xclip {
    fn types(&self) -> Result<Vec<String>, ClipboardError> {
        Ok(offered(run(Command::new("xclip").args([
            "-selection",
            "clipboard",
            "-t",
            "TARGETS",
            "-o",
        ]))?))
    }

    fn read(&self, mime: &str) -> Result<Vec<u8>, ClipboardError> {
        run(Command::new("xclip").args(["-selection", "clipboard", "-t", mime, "-o"]))
    }
}

struct PbPaste;

impl Reader for PbPaste {
    fn types(&self) -> Result<Vec<String>, ClipboardError> {
        Ok(Vec::new())
    }

    fn read(&self, _mime: &str) -> Result<Vec<u8>, ClipboardError> {
        Err(ClipboardError::NoImage)
    }
}
