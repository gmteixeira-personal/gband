use gband_client::animation::Animations;
use gband_client::clipboard::{ClipboardError, Image, Reader, image, pick_mime};
use gband_client::{Display, Step, paste_image_from};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_protocol::{ClientMessage, ServerMessage};

struct Fake {
    types: Vec<String>,
    data: Vec<u8>,
}

impl Fake {
    fn new(types: &[&str], data: &[u8]) -> Self {
        Self {
            types: types.iter().map(|t| t.to_string()).collect(),
            data: data.to_vec(),
        }
    }
}

impl Reader for Fake {
    fn types(&self) -> Result<Vec<String>, ClipboardError> {
        Ok(self.types.clone())
    }

    fn read(&self, _mime: &str) -> Result<Vec<u8>, ClipboardError> {
        Ok(self.data.clone())
    }
}

fn focused() -> (Display, WindowId) {
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    layout.open(
        window,
        layout.bands()[0].id,
        None,
        None,
        &LayoutOptions::default(),
    );
    let mut display = Display::new(Size::new(80, 24), Animations::Off);
    display.apply(ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout,
    });
    assert_eq!(display.focused(), Some(window));
    (display, window)
}

#[test]
fn prefers_png() {
    let fake = Fake::new(&["image/bmp", "image/png"], b"PNG");
    let picked = image(&fake).expect("an image");
    assert_eq!(picked.mime, "image/png");
    assert_eq!(picked.data, b"PNG");
}

#[test]
fn any_image_when_no_png() {
    let fake = Fake::new(&["image/jpeg"], b"JPEG");
    let picked = image(&fake).expect("an image");
    assert_eq!(picked.mime, "image/jpeg");
}

#[test]
fn no_image_when_only_text() {
    let fake = Fake::new(&["text/plain", "text/html"], b"hi");
    assert!(matches!(image(&fake), Err(ClipboardError::NoImage)));
    assert_eq!(pick_mime(&["text/plain".to_string()]), None);
}

#[test]
fn reader_missing_names_the_tool() {
    let message = ClipboardError::ReaderMissing("wl-paste".to_string()).message();
    assert!(message.contains("wl-paste"), "{message}");
}

#[test]
fn forwards_the_image_to_the_focused_window() {
    let (mut display, window) = focused();
    let steps = paste_image_from(&mut display, || {
        Ok(Image {
            mime: "image/png".to_string(),
            data: b"PNG".to_vec(),
        })
    });
    assert_eq!(
        steps,
        vec![Step::Send(ClientMessage::PasteImage {
            window,
            mime: "image/png".to_string(),
            data: b"PNG".to_vec(),
        })]
    );
}

#[test]
fn reader_error_adds_one_error_and_sends_nothing() {
    let (mut display, _window) = focused();
    let steps = paste_image_from(&mut display, || {
        Err(ClipboardError::ReaderMissing("wl-paste".into()))
    });
    assert_eq!(steps, vec![Step::Nothing]);
    let errors = display.own_errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("wl-paste"), "{}", errors[0]);
}

#[test]
fn no_image_adds_one_error_and_sends_nothing() {
    let (mut display, _window) = focused();
    let steps = paste_image_from(&mut display, || Err(ClipboardError::NoImage));
    assert_eq!(steps, vec![Step::Nothing]);
    let errors = display.own_errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("image"), "{}", errors[0]);
}

#[test]
fn nothing_focused_sends_nothing() {
    let mut display = Display::new(Size::new(80, 24), Animations::Off);
    assert_eq!(display.focused(), None);
    let steps = paste_image_from(&mut display, || panic!("read must not run"));
    assert_eq!(steps, vec![Step::Nothing]);
}
