mod support;
use support::{
    fixed_clock,
    scripted::{Scripted, text},
};
use syncloom_core::*;

async fn fixture() -> (tempfile::TempDir, Studio, Piece) {
    let dir = tempfile::tempdir().unwrap();
    let studio = Studio::new(
        Store::open(dir.path()).unwrap(),
        Scripted::new(text("Build with evidence.")),
        fixed_clock(),
        Settings::default(),
    );
    let piece = studio
        .create_thought(ThoughtRequest {
            text: "Build with evidence.".into(),
            channels: vec!["manual_export".into()],
        })
        .await
        .unwrap()
        .pieces
        .remove(0);
    (dir, studio, piece)
}
fn request(p: &Piece, target: &str) -> manual_export::ExportRequest {
    manual_export::ExportRequest {
        expected_revision: p.revision.clone(),
        target: target.into(),
    }
}
fn approve(s: &Studio, p: &Piece) -> Piece {
    s.review(
        &p.piece_id,
        ReviewRequest {
            expected_revision: p.revision.clone(),
            status: Some(Status::Approved),
            text: None,
            expected_assets: p.assets.as_ref().map(|a| AssetHashes {
                image_sha256: a.image_sha256.clone(),
                carousel_sha256: a.carousel_sha256.clone(),
                slides_sha256: a.slides_sha256.clone(),
            }),
        },
    )
    .unwrap()
}

#[tokio::test]
async fn export_approval_revision_target_and_id_guards_write_nothing_when_denied() {
    let (dir, s, p) = fixture().await;
    assert_eq!(
        s.export(&p.piece_id, request(&p, "reddit")).unwrap_err(),
        StudioError::ApprovalRequired
    );
    let p = approve(&s, &p);
    let mut stale = request(&p, "reddit");
    stale.expected_revision = "stale".into();
    assert_eq!(
        s.export(&p.piece_id, stale).unwrap_err(),
        StudioError::RevisionConflict
    );
    assert_eq!(
        s.export(&p.piece_id, request(&p, "other")).unwrap_err(),
        StudioError::InvalidRequest
    );
    assert_eq!(
        s.export("../escape", request(&p, "reddit")).unwrap_err(),
        StudioError::PieceNotFound
    );
    assert!(support::files(&dir, "exports").is_empty());
    assert!(support::files(&dir, "outbox").is_empty());
}

#[tokio::test]
async fn export_repeat_restart_and_tiktok_notes_never_claim_posted() {
    let (dir, s, p) = fixture().await;
    let p = approve(&s, &p);
    let first = s.export(&p.piece_id, request(&p, "reddit")).unwrap();
    assert_eq!(first.status, "exported");
    assert!(first.created);
    assert_eq!(
        s.show(&p.piece_id).unwrap().publish_status.as_deref(),
        Some("exported")
    );
    let count = support::events(&dir).len();
    let reopened = Studio::new(
        Store::open(dir.path()).unwrap(),
        Scripted::new(text("unused")),
        fixed_clock(),
        Settings::default(),
    );
    let again = reopened.export(&p.piece_id, request(&p, "reddit")).unwrap();
    assert_eq!(again.request_id, first.request_id);
    assert_eq!(again.bundle, first.bundle);
    assert!(!again.created);
    assert_eq!(support::events(&dir).len(), count);
    let tiktok = s.export(&p.piece_id, request(&p, "tiktok")).unwrap();
    assert_ne!(tiktok.request_id, first.request_id);
    assert!(
        std::fs::read_to_string(std::path::Path::new(&tiktok.bundle).join("notes.md"))
            .unwrap()
            .contains("video")
    );
    for entry in std::fs::read_dir(dir.path().join("outbox")).unwrap() {
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
        assert_eq!(value["status"], "exported");
        assert!(value["post_id"].is_null());
        assert_eq!(value["attempts"], 0);
    }
    std::fs::write(
        std::path::Path::new(&first.bundle).join("text.txt"),
        b"damaged",
    )
    .unwrap();
    assert!(s.export(&p.piece_id, request(&p, "reddit")).is_err());
    assert_eq!(support::events(&dir).len(), count + 1);
}

#[tokio::test]
async fn visual_export_requires_exact_assets_and_copies_png_pdf_and_alt_text() {
    let (dir, s, p) = fixture().await;
    let p = s
        .render(
            &p.piece_id,
            RenderRequest {
                expected_revision: p.revision,
                template: "midnight".into(),
                slides: vec!["First slide alt text.".into(), "Second slide.".into()],
            },
        )
        .await
        .unwrap();
    let p = approve(&s, &p);
    let out = s.export(&p.piece_id, request(&p, "reddit")).unwrap();
    let path = std::path::Path::new(&out.bundle);
    assert_eq!(
        std::fs::read(path.join("image.png")).unwrap(),
        s.view_asset(&p.piece_id, AssetKind::Image).unwrap().0
    );
    assert_eq!(
        std::fs::read(path.join("carousel.pdf")).unwrap(),
        s.view_asset(&p.piece_id, AssetKind::Carousel).unwrap().0
    );
    assert_eq!(
        std::fs::read_to_string(path.join("alt-text.txt")).unwrap(),
        "First slide alt text."
    );
    std::fs::write(
        dir.path()
            .join("assets")
            .join(&p.assets.as_ref().unwrap().image_sha256),
        b"damaged",
    )
    .unwrap();
    assert_eq!(
        s.export(&p.piece_id, request(&p, "tiktok")).unwrap_err(),
        StudioError::AssetIntegrity
    );
}

#[cfg(unix)]
#[tokio::test]
async fn symlink_export_root_and_bundle_are_refused_without_touching_the_target() {
    let (dir, s, p) = fixture().await;
    let p = approve(&s, &p);
    let outside = tempfile::tempdir().unwrap();
    std::fs::remove_dir(dir.path().join("exports")).unwrap();
    std::os::unix::fs::symlink(outside.path(), dir.path().join("exports")).unwrap();
    assert!(s.export(&p.piece_id, request(&p, "reddit")).is_err());
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    std::fs::remove_file(dir.path().join("exports")).unwrap();
    std::fs::create_dir(dir.path().join("exports")).unwrap();
    let key = outbox::intent_key("export:reddit", &p.piece_id, &p.revision);
    std::os::unix::fs::symlink(outside.path(), dir.path().join("exports").join(&key)).unwrap();
    assert!(s.export(&p.piece_id, request(&p, "reddit")).is_err());
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
}

#[tokio::test]
async fn partial_bundle_failure_records_no_success_and_a_retry_can_complete() {
    let (dir, s, p) = fixture().await;
    let p = approve(&s, &p);
    let key = outbox::intent_key("export:reddit", &p.piece_id, &p.revision);
    let bundle = dir.path().join("exports").join(key);
    std::fs::create_dir(&bundle).unwrap();
    std::fs::create_dir(bundle.join("text.txt")).unwrap();
    assert!(s.export(&p.piece_id, request(&p, "reddit")).is_err());
    assert!(support::files(&dir, "outbox").is_empty());
    assert_eq!(s.show(&p.piece_id).unwrap().publish_status, None);
    std::fs::remove_dir(bundle.join("text.txt")).unwrap();
    let receipt = s.export(&p.piece_id, request(&p, "reddit")).unwrap();
    assert_eq!(receipt.status, "exported");
    assert!(receipt.created);
    assert!(bundle.join("manifest.json").is_file());
}
