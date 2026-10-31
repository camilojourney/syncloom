#[path = "support/rendering.rs"]
mod rendering;
#[path = "support/scripted.rs"]
mod scripted;
#[path = "support/scripted_platform.rs"]
mod scripted_platform;
use rendering::{config, executable};
use std::{sync::Arc, time::Duration};
use syncloom_core::{
    visual::{BROWSER_VERSION, digest},
    *,
};

fn request(rev: &str, template: &str, slides: &[&str]) -> RenderRequest {
    RenderRequest {
        expected_revision: rev.into(),
        template: template.into(),
        slides: slides.iter().map(|s| (*s).into()).collect(),
    }
}
fn review(rev: &str) -> ReviewRequest {
    ReviewRequest {
        expected_revision: rev.into(),
        status: Some(Status::Approved),
        text: None,
        expected_assets: None,
    }
}
fn reviewed(piece: &Piece) -> ReviewRequest {
    let assets = piece.assets.as_ref().unwrap();
    ReviewRequest {
        expected_assets: Some(AssetHashes {
            image_sha256: assets.image_sha256.clone(),
            carousel_sha256: assets.carousel_sha256.clone(),
        }),
        ..review(&piece.revision)
    }
}

async fn fixture(config: RendererConfig) -> (tempfile::TempDir, Studio, Piece) {
    let dir = tempfile::tempdir().unwrap();
    let studio = Studio::new(
        Store::open(dir.path()).unwrap(),
        scripted::Scripted::new(scripted::text("Build with evidence.")),
        Arc::new(FixedClock(time::macros::datetime!(2026-10-30 18:00 UTC))),
        Settings::default(),
    )
    .with_renderer(config);
    let piece = studio
        .create_thought(ThoughtRequest {
            text: "Build with evidence.".into(),
            channels: vec!["linkedin_text".into()],
        })
        .await
        .unwrap()
        .pieces
        .remove(0);
    (dir, studio, piece)
}

fn pixels(bytes: &[u8]) -> Vec<u8> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut output = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut output).unwrap();
    assert_eq!((info.width, info.height), (1080, 1350));
    assert_eq!(info.bit_depth, png::BitDepth::Eight);
    assert_eq!(info.color_type, png::ColorType::Rgb);
    output.truncate(info.buffer_size());
    output
}

#[tokio::test]
async fn pinned_renderer_snapshot_repeat_bytes_and_pdf_geometry() {
    let config = config();
    assert_eq!(config.version, BROWSER_VERSION);
    let content = request(
        "",
        "midnight",
        &["Build with evidence.", "Review the exact bytes."],
    )
    .content()
    .unwrap();
    let first = config.render(&content).await.unwrap();
    tokio::time::sleep(Duration::from_millis(1100)).await; // cross the PDF creation-time boundary
    let second = config.render(&content).await.unwrap();
    assert_eq!(
        digest(&first.image),
        digest(&second.image),
        "PNG repeat bytes"
    );
    assert_eq!(
        digest(&first.carousel),
        digest(&second.carousel),
        "normalized PDF repeat bytes"
    );
    if let Ok(dir) = std::env::var("SYNCLOOM_VISUAL_RECEIPTS") {
        std::fs::write(std::path::Path::new(&dir).join("render.png"), &first.image).unwrap();
        std::fs::write(
            std::path::Path::new(&dir).join("render.pdf"),
            &first.carousel,
        )
        .unwrap();
    }
    let doc = lopdf::Document::load_mem(&first.carousel).unwrap();
    assert_eq!(doc.get_pages().len(), 2);
    for id in doc.get_pages().values() {
        let page = doc.get_object(*id).unwrap().as_dict().unwrap();
        let media = page.get(b"MediaBox").unwrap().as_array().unwrap();
        let values: Vec<f32> = media.iter().map(|n| n.as_float().unwrap()).collect();
        assert_eq!(values, [0.0, 0.0, 810.0, 1012.5]);
    }
    assert!(
        doc.extract_text(&[1, 2])
            .unwrap()
            .contains("Build with evidence.")
    );
    assert!(
        !doc.objects
            .values()
            .any(|o| o.as_dict().is_ok_and(|d| d.has(b"ShadingType"))),
        "gradient must be raster, text vector"
    );
    let edge = pixels(&first.image);
    assert!(
        edge[1079 * 3..1080 * 3].iter().all(|c| *c < 100),
        "browser scrollbar leaked into image"
    );
    assert_snapshot("midnight", &first.image);
}

fn assert_snapshot(name: &str, bytes: &[u8]) {
    let golden = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/{name}.png")),
    )
    .expect("reviewed initial golden");
    let expected = pixels(&golden);
    let actual = pixels(bytes);
    assert_eq!(actual.len(), expected.len());
    let changed = actual
        .as_chunks::<3>()
        .0
        .iter()
        .zip(expected.as_chunks::<3>().0.iter())
        .filter(|(a, b)| a.iter().zip(*b).any(|(x, y)| x.abs_diff(*y) > 8))
        .count();
    assert!(
        changed <= 1458,
        "{changed} pixels differ above 8/255; tolerance 0.1% of 1,458,000 pixels"
    );
}

#[tokio::test]
async fn ten_pages_are_uniform_and_overflow_is_refused() {
    let config = config();
    let content = request("", "paper", &["One slide"; 10]).content().unwrap();
    let rendered = config.render(&content).await.unwrap();
    if let Ok(dir) = std::env::var("SYNCLOOM_VISUAL_RECEIPTS") {
        std::fs::write(
            std::path::Path::new(&dir).join("paper.png"),
            &rendered.image,
        )
        .unwrap();
    }
    assert_snapshot("paper", &rendered.image);
    let doc = lopdf::Document::load_mem(&rendered.carousel).unwrap();
    assert_eq!(doc.get_pages().len(), 10);
    for id in doc.get_pages().values() {
        let media = doc
            .get_object(*id)
            .unwrap()
            .as_dict()
            .unwrap()
            .get(b"MediaBox")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(media[2].as_float().unwrap(), 810.0);
        assert_eq!(media[3].as_float().unwrap(), 1012.5);
    }
    let overflow = request("", "paper", &[&"line\n".repeat(100)])
        .content()
        .unwrap();
    assert!(matches!(
        config.render(&overflow).await,
        Err(StudioError::InvalidRequest)
    ));
}

#[tokio::test]
async fn text_and_template_revisions_and_approval_integrity_are_bound() {
    let (dir, studio, original) = fixture(config()).await;
    let rendered = studio
        .render(
            &original.piece_id,
            request(&original.revision, "midnight", &["One slide"]),
        )
        .await
        .unwrap();
    let approved = studio
        .review(&original.piece_id, reviewed(&rendered))
        .unwrap();
    assert_eq!(approved.status, Status::Approved);
    let rerendered = studio
        .render(
            &original.piece_id,
            request(&rendered.revision, "midnight", &["One slide"]),
        )
        .await
        .unwrap();
    assert_eq!(rerendered.status, Status::PendingReview);
    assert!(rerendered.review_decision_id.is_none());
    let changed = studio
        .render(
            &original.piece_id,
            request(&rerendered.revision, "paper", &["One slide"]),
        )
        .await
        .unwrap();
    assert_ne!(changed.revision, rendered.revision);
    assert_eq!(
        studio.review(&original.piece_id, review(&rendered.revision)),
        Err(StudioError::RevisionConflict)
    );
    let store = Store::open(dir.path()).unwrap();
    let mut stale = changed.clone();
    stale.assets.as_mut().unwrap().revision = "0".repeat(64);
    store.lock().unwrap().save_piece(&stale).unwrap();
    assert_eq!(
        studio.review(&original.piece_id, reviewed(&changed)),
        Err(StudioError::AssetsRequired)
    );
    store.lock().unwrap().save_piece(&changed).unwrap();
    let platform = scripted_platform::ScriptedPlatform::new(scripted_platform::posted());
    let studio = studio.with_delivery("linkedin", platform.clone());
    studio
        .review(&original.piece_id, reviewed(&changed))
        .unwrap();
    let outcome = studio
        .publish(
            &original.piece_id,
            PublishRequest {
                expected_revision: changed.revision.clone(),
                accepted_cost_microusd: None,
                dry_run: false,
            },
        )
        .await
        .unwrap();
    assert_eq!(outcome.status, "contained");
    assert!(platform.requests().is_empty());
    let assets = changed.assets.as_ref().unwrap();
    let file = dir.path().join("assets").join(&assets.image_sha256);
    std::fs::write(&file, b"damaged").unwrap();
    assert_eq!(
        studio.review(&original.piece_id, reviewed(&changed)),
        Err(StudioError::AssetIntegrity)
    );
    assert_eq!(
        studio.view_asset(&original.piece_id, AssetKind::Image),
        Err(StudioError::AssetIntegrity)
    );
    std::fs::remove_file(&file).unwrap();
    std::os::unix::fs::symlink("/etc/hosts", &file).unwrap();
    assert_eq!(
        studio.view_asset(&original.piece_id, AssetKind::Image),
        Err(StudioError::AssetIntegrity)
    );
    assert_eq!(
        studio.view_asset("../escape", AssetKind::Image),
        Err(StudioError::PieceNotFound)
    );
}

#[tokio::test]
async fn missing_timeout_crash_version_and_font_fail_truthfully() {
    let dir = tempfile::tempdir().unwrap();
    let content = request("", "midnight", &["One slide"]).content().unwrap();
    let mut missing = config();
    missing.binary = dir.path().join("missing");
    assert!(matches!(
        missing.render(&content).await,
        Err(StudioError::RendererMissing)
    ));
    let mut crash = config();
    crash.binary = executable(dir.path(), "exit 12");
    assert!(matches!(
        crash.render(&content).await,
        Err(StudioError::RendererFailed)
    ));
    let marker = dir.path().join("pid");
    let mut slow = config();
    slow.binary = executable(
        dir.path(),
        &format!("echo $$ > '{}'; exec /bin/sleep 30", marker.display()),
    );
    slow.deadline = Duration::from_millis(200);
    assert!(matches!(
        slow.render(&content).await,
        Err(StudioError::RendererTimeout)
    ));
    let pid = std::fs::read_to_string(marker).unwrap();
    assert!(
        !std::process::Command::new("/bin/kill")
            .args(["-0", pid.trim()])
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success(),
        "timed-out child must be reaped"
    );
    let mut version = config();
    version.version = "0.0.0.0".into();
    assert!(matches!(
        version.render(&content).await,
        Err(StudioError::RendererVersionMismatch)
    ));
    let mut font = config();
    font.font_sha256 = "0".repeat(64);
    assert!(matches!(
        font.render(&content).await,
        Err(StudioError::RendererFontMismatch)
    ));
    let (store, studio, original) = fixture(missing).await;
    assert_eq!(
        studio
            .render(
                &original.piece_id,
                request(&original.revision, "paper", &["One slide"])
            )
            .await,
        Err(StudioError::RendererMissing)
    );
    let changed = studio.show(&original.piece_id).unwrap();
    assert!(changed.assets.is_none());
    assert_eq!(
        studio.review(&original.piece_id, review(&changed.revision)),
        Err(StudioError::AssetsRequired)
    );
    let log = std::fs::read_to_string(store.path().join("provenance/events.jsonl")).unwrap();
    assert!(log.contains("render_failed"));
    assert!(log.contains("renderer_missing"));
}

#[tokio::test]
async fn same_revision_rerender_requires_current_hashes() {
    let (_dir, studio, original) = fixture(config()).await;
    let first = studio
        .render(
            &original.piece_id,
            request(&original.revision, "midnight", &["One slide"]),
        )
        .await
        .unwrap();
    studio.review(&original.piece_id, reviewed(&first)).unwrap();
    let mut bold = config();
    bold.font = "/System/Library/Fonts/Supplemental/Arial Bold.ttf".into();
    bold.font_sha256 = digest(&std::fs::read(&bold.font).unwrap());
    let studio = studio.with_renderer(bold);
    let second = studio
        .render(
            &original.piece_id,
            request(&first.revision, "midnight", &["One slide"]),
        )
        .await
        .unwrap();
    assert_eq!(first.revision, second.revision);
    assert_ne!(
        first.assets.as_ref().unwrap().image_sha256,
        second.assets.as_ref().unwrap().image_sha256
    );
    assert_eq!(
        studio.review(&original.piece_id, reviewed(&first)),
        Err(StudioError::AssetConflict)
    );
    assert_eq!(
        studio
            .review(&original.piece_id, reviewed(&second))
            .unwrap()
            .status,
        Status::Approved
    );
}

#[tokio::test]
async fn concurrent_edit_rejects_render_completion() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("started");
    let real = config();
    let mut delayed = real.clone();
    delayed.binary = executable(
        dir.path(),
        &format!(
            "touch '{}'; /bin/sleep 0.3; exec '{}' \"$@\"",
            marker.display(),
            real.binary.display()
        ),
    );
    let (_store, studio, original) = fixture(delayed).await;
    let id = original.piece_id.clone();
    let render = studio.render(&id, request(&original.revision, "paper", &["One slide"]));
    let edit = async {
        tokio::time::timeout(Duration::from_secs(5), async {
            while !marker.exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("renderer wrapper started");
        let current = studio.show(&id).unwrap();
        studio
            .review(
                &id,
                ReviewRequest {
                    expected_revision: current.revision,
                    text: Some("Changed while rendering.".into()),
                    status: None,
                    expected_assets: None,
                },
            )
            .unwrap()
    };
    let (result, edited) = tokio::join!(render, edit);
    assert_eq!(result, Err(StudioError::RevisionConflict));
    assert!(edited.assets.is_none());
    assert_eq!(studio.show(&id).unwrap(), edited);
}
