use std::time::Duration;

use gband_server::{CLIPBOARD_IMAGE_SUBDIR, prune_clipboard_images, write_clipboard_image};
use gband_test_support::*;

const HOUR: Duration = Duration::from_secs(3600);

#[test]
fn writes_a_png_under_the_images_subdir() {
    let base = runtime_dir("clip-write");
    let path = write_clipboard_image(&base, "image/png", b"PNG").expect("a written image");
    assert_eq!(path.extension().and_then(|ext| ext.to_str()), Some("png"));
    assert_eq!(
        path.parent()
            .and_then(|dir| dir.file_name())
            .and_then(|name| name.to_str()),
        Some(CLIPBOARD_IMAGE_SUBDIR)
    );
    assert!(path.starts_with(&*base));
    assert_eq!(std::fs::read(&path).unwrap(), b"PNG");
}

#[test]
fn jpeg_gets_a_jpg_extension() {
    let base = runtime_dir("clip-jpeg");
    let path = write_clipboard_image(&base, "image/jpeg", b"JPEG").expect("a written image");
    assert_eq!(path.extension().and_then(|ext| ext.to_str()), Some("jpg"));
}

#[test]
fn prunes_files_older_than_the_max_age() {
    let base = runtime_dir("clip-prune");
    let dir = base.join(CLIPBOARD_IMAGE_SUBDIR);
    std::fs::create_dir_all(&dir).unwrap();
    let old = dir.join("old.png");
    std::fs::write(&old, b"x").unwrap();
    let born = std::fs::metadata(&old).unwrap().modified().unwrap();
    prune_clipboard_images(&dir, born + 2 * HOUR, HOUR);
    assert!(!old.exists());
}

#[test]
fn keeps_files_within_the_max_age() {
    let base = runtime_dir("clip-keep");
    let dir = base.join(CLIPBOARD_IMAGE_SUBDIR);
    std::fs::create_dir_all(&dir).unwrap();
    let recent = dir.join("recent.png");
    std::fs::write(&recent, b"x").unwrap();
    let born = std::fs::metadata(&recent).unwrap().modified().unwrap();
    prune_clipboard_images(&dir, born + Duration::from_secs(600), HOUR);
    assert!(recent.exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn pastes_the_image_path_into_the_focused_window() {
    let dir = runtime_dir("clip-paste");
    let mut server_config = config(&dir, &["/bin/sh"]);
    server_config.clipboard_image_dir = Some(dir.join("shots"));
    let server = TestServer::start_with(dir, server_config).await;
    let mut client = server.attach(80, 24).await;
    let window = client.first();
    client.paste_image(window, "image/png", b"PNG").await;
    client
        .wait_until(|client| {
            client
                .window_screen(window)
                .contents()
                .split_whitespace()
                .collect::<String>()
                .contains("gband-images")
        })
        .await;
    let images = server
        .runtime_dir
        .join("shots")
        .join(CLIPBOARD_IMAGE_SUBDIR);
    let written: Vec<_> = std::fs::read_dir(&images).unwrap().flatten().collect();
    assert_eq!(written.len(), 1);
    assert_eq!(std::fs::read(written[0].path()).unwrap(), b"PNG");
}
