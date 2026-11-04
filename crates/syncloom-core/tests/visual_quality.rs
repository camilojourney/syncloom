//! Owned synthetic diagnostics, exposed to this implementation; not semantic judge gold.
#[path = "support/scripted.rs"]
mod scripted;
use std::sync::Arc;
use syncloom_core::{visual_quality::*, *};

#[expect(
    clippy::unwrap_used,
    reason = "Test setup must fail immediately on invalid evidence."
)]
fn studio(dir: &std::path::Path) -> Studio {
    Studio::new(
        Store::open(dir).unwrap(),
        scripted::Scripted::new(scripted::text("Keep one owner per rule.")),
        Arc::new(FixedClock(time::macros::datetime!(2026-11-04 18:00 UTC))),
        Settings::default(),
    )
}
#[expect(
    clippy::unwrap_used,
    reason = "Test setup must fail immediately on invalid evidence."
)]
fn request(piece: &Piece) -> AssessmentRequest {
    AssessmentRequest {
        expected_revision: piece.revision.clone(),
        expected_assets: piece.assets.as_ref().unwrap().hashes(),
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "Test setup must fail immediately on invalid evidence."
)]
async fn rendered(studio: &Studio, template: &str) -> Piece {
    let piece = studio
        .create_thought(ThoughtRequest {
            text: "Keep one owner per rule.".into(),
            channels: vec!["linkedin_text".into()],
        })
        .await
        .unwrap()
        .pieces
        .remove(0);
    studio
        .render(
            &piece.piece_id,
            RenderRequest {
                expected_revision: piece.revision,
                template: template.into(),
                slides: vec![
                    "Keep one owner per rule.".into(),
                    "<literal markup> stays text.".into(),
                ],
            },
        )
        .await
        .unwrap()
}
#[expect(
    clippy::unwrap_used,
    reason = "Test setup must fail immediately on invalid evidence."
)]
fn blank_png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&vec![255; (width * height * 3) as usize])
        .unwrap();
    bytes
}

#[tokio::test]
async fn blank_malformed_and_wrong_geometry_never_become_a_quality_pass() {
    let dir = tempfile::tempdir().unwrap();
    let studio = studio(dir.path());
    let original = rendered(&studio, "midnight").await;
    let store = Store::open(dir.path()).unwrap();
    for (bytes, decoding, dimensions) in [
        (
            blank_png(1080, 1350),
            ObservationStatus::Satisfied,
            ObservationStatus::Satisfied,
        ),
        (
            blank_png(1080, 1349),
            ObservationStatus::Satisfied,
            ObservationStatus::Failed,
        ),
        (
            b"not a PNG".to_vec(),
            ObservationStatus::Failed,
            ObservationStatus::Failed,
        ),
    ] {
        let mut piece = original.clone();
        {
            let lock = store.lock().unwrap();
            piece.assets.as_mut().unwrap().image_sha256 = lock.save_asset(&bytes).unwrap();
            lock.save_piece(&piece).unwrap();
        }
        let report = studio
            .assess_visual(&piece.piece_id, request(&piece))
            .unwrap();
        assert_eq!(report.checks[0].status, decoding);
        assert_eq!(report.checks[1].status, dimensions);
        assert_eq!(report.semantic_quality, ObservationStatus::Unassessed);
        assert_eq!(
            report.quality_score, None,
            "Even a blank correct-size image is not a visual-quality pass"
        );
    }
    let mut piece = original;
    let pdf = std::fs::read(
        dir.path()
            .join("assets")
            .join(&piece.assets.as_ref().unwrap().carousel_sha256),
    )
    .unwrap();
    let mut doc = lopdf::Document::load_mem(&pdf).unwrap();
    let id = doc.get_pages()[&1];
    doc.get_object_mut(id).unwrap().as_dict_mut().unwrap().set(
        "MediaBox",
        vec![
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(810),
            lopdf::Object::Integer(1012),
        ],
    );
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    {
        let lock = store.lock().unwrap();
        piece.assets.as_mut().unwrap().carousel_sha256 = lock.save_asset(&bytes).unwrap();
        lock.save_piece(&piece).unwrap();
    }
    let report = studio
        .assess_visual(&piece.piece_id, request(&piece))
        .unwrap();
    assert_eq!(report.checks[4].status, ObservationStatus::Failed);
    assert_eq!(report.technical_status, ObservationStatus::Failed);
    assert_eq!(report.quality_score, None);
}

#[tokio::test]
async fn real_assets_recompute_after_restart_and_preserve_approval_and_content_free_audit() {
    let dir = tempfile::tempdir().unwrap();
    let first = studio(dir.path());
    let piece = rendered(&first, "paper").await;
    let approved = first
        .review(
            &piece.piece_id,
            ReviewRequest {
                expected_revision: piece.revision.clone(),
                status: Some(Status::Approved),
                text: None,
                expected_assets: Some(piece.assets.as_ref().unwrap().hashes()),
                expected_target: None,
            },
        )
        .unwrap();
    let before = std::fs::read(dir.path().join("provenance/events.jsonl")).unwrap();
    let report = first
        .assess_visual(&piece.piece_id, request(&piece))
        .unwrap();
    drop(first);
    let reopened = studio(dir.path());
    let repeated = reopened
        .assess_visual(&piece.piece_id, request(&piece))
        .unwrap();
    assert_eq!(
        serde_json::to_value(&report).unwrap(),
        serde_json::to_value(repeated).unwrap()
    );
    assert_eq!(reopened.show(&piece.piece_id).unwrap(), approved);
    let audit = std::fs::read(dir.path().join("provenance/events.jsonl")).unwrap();
    assert!(audit.starts_with(&before));
    assert!(!String::from_utf8(audit).unwrap().contains("Keep one owner"));
    assert!(!dir.path().join("assessments").exists());
}

#[tokio::test]
async fn concurrent_edit_returns_original_bound_evidence_or_conflict_never_mixed_identity() {
    let dir = tempfile::tempdir().unwrap();
    let studio = studio(dir.path());
    let piece = rendered(&studio, "paper").await;
    std::thread::scope(|scope| {
        let assessment = scope.spawn(|| studio.assess_visual(&piece.piece_id, request(&piece)));
        studio
            .review(
                &piece.piece_id,
                ReviewRequest {
                    expected_revision: piece.revision.clone(),
                    status: None,
                    text: Some("An edited draft.".into()),
                    expected_assets: None,
                    expected_target: None,
                },
            )
            .unwrap();
        match assessment.join().unwrap() {
            Ok(report) => {
                assert_eq!(report.revision, piece.revision);
                assert_eq!(report.assets, piece.assets.as_ref().unwrap().hashes());
            }
            Err(error) => assert_eq!(error, StudioError::RevisionConflict),
        }
    });
    assert_eq!(
        studio
            .assess_visual(&piece.piece_id, request(&piece))
            .unwrap_err(),
        StudioError::RevisionConflict
    );
}
