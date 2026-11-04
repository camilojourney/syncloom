use super::*;

pub(super) struct PreviewEvidence {
    pub html: String,
    /// Existing asset guards only; technical observations never introduce an approval gate.
    pub assets_usable: bool,
}

pub(super) fn assets(state: &AppState, piece: &Piece) -> Result<PreviewEvidence, Failure> {
    let id = escape(&piece.piece_id);
    let Some(assets) = &piece.assets else {
        let action = if piece.visual.is_some() {
            "Render new previews"
        } else {
            "Render optional previews"
        };
        return Ok(PreviewEvidence {
            html: format!(
                r#"<section class="visual-quality"><h2>Visual quality: not assessed</h2><p class="hint">No current rendered assets to inspect.</p><a href="/review/{id}/render">{action}</a></section>"#
            ),
            assets_usable: piece.visual.is_none(),
        });
    };
    let report = match state.studio.assess_visual(
        &piece.piece_id,
        syncloom_core::visual_quality::AssessmentRequest {
            expected_revision: piece.revision.clone(),
            expected_assets: assets.hashes(),
        },
    ) {
        Ok(report) => report,
        Err(
            StudioError::AssetIntegrity
            | StudioError::AssetConflict
            | StudioError::AssetsRequired
            | StudioError::RevisionConflict,
        ) => {
            return Ok(PreviewEvidence {
                html: format!(
                    r#"<section class="visual-quality"><h2>Could not inspect these previews</h2><p>Visual quality: not assessed. The saved bytes are unavailable, damaged or changed. No technical result is available.</p><a href="/review/{id}">Reload this version</a> · <a href="/review/{id}/render">Render new previews</a></section>"#
                ),
                assets_usable: false,
            });
        }
        Err(error) => return Err(error.into()),
    };
    let quality = assessment(&report);
    let image_hash = escape(&assets.image_sha256);
    let pdf_hash = escape(&assets.carousel_sha256);
    Ok(PreviewEvidence {
        html: format!(
            r#"{quality}<div class="preview-grid"><article class="preview-card"><div class="preview-label">Image / PNG</div><img class="asset-image" loading="lazy" src="/v1/pieces/{id}/assets/image?sha256={image_hash}" alt="First slide for this saved version"><div class="preview-details"><a href="/v1/pieces/{id}/assets/image?sha256={image_hash}" target="_blank" rel="noreferrer">Open image</a><details><summary>Details</summary><p class="label">Image SHA-256</p><code class="hash">{}</code></details></div></article><article class="preview-card"><div class="preview-label">Carousel / PDF · {} pages</div><div class="pdf-preview"><iframe title="PDF slideshow" src="/v1/pieces/{id}/assets/carousel?sha256={pdf_hash}#toolbar=0&amp;navpanes=0&amp;view=FitH"></iframe></div><div class="preview-details"><a href="/v1/pieces/{id}/assets/carousel?sha256={pdf_hash}" target="_blank" rel="noreferrer">Open PDF</a><p class="hint">Scroll inside the PDF to inspect every page. If your browser cannot embed PDFs, use the link above.</p><details><summary>Details</summary><p class="label">Carousel SHA-256</p><code class="hash">{}</code></details></div></article></div>{}"#,
            escape(&assets.image_sha256),
            assets.pages,
            escape(&assets.carousel_sha256),
            jpeg_previews(&id, assets)
        ),
        assets_usable: true,
    })
}

fn assessment(report: &syncloom_core::visual_quality::Assessment) -> String {
    use syncloom_core::visual_quality::ObservationStatus;
    let title = if report.technical_status == ObservationStatus::Satisfied {
        "Technical checks satisfied"
    } else {
        "Technical checks need attention"
    };
    let checks: String = report
        .checks
        .iter()
        .map(|check| {
            format!(
                "<li><strong>{}</strong><span>{}: {}</span></li>",
                escape(&check.check.replace("png_", "PNG ").replace("pdf_", "PDF ")),
                if check.status == ObservationStatus::Satisfied {
                    "Checked"
                } else {
                    "Not satisfied"
                },
                escape(&check.observed)
            )
        })
        .collect();
    format!(
        r#"<section class="visual-quality" aria-labelledby="quality-title"><h2 id="quality-title">Local technical checks</h2><p><strong>{title}.</strong> These checks apply to the exact previews below.</p><p><strong>Visual quality: not assessed.</strong> Review meaning, readability and design yourself. No quality score is available.</p><p class="hint">Not an approval. Nothing is sent by inspecting previews.</p><details><summary>Checks and limitations</summary><ul class="quality-checks">{checks}</ul><p class="hint">{}</p><p class="hint">Assessor: {} · Source: local rendered bytes</p><p class="label">Assessed revision</p><code class="hash">{}</code></details></section>"#,
        escape(report.limitations),
        escape(report.assessor),
        escape(&report.revision)
    )
}

fn jpeg_previews(id: &str, assets: &syncloom_core::visual::Assets) -> String {
    let slides:String=assets.slides_sha256.iter().enumerate().map(|(index,hash)|format!(
        r#"<article class="preview-card"><div class="preview-label">Meta / JPEG · slide {}</div><img class="asset-image" loading="lazy" src="/v1/pieces/{id}/assets/slides/{index}?sha256={hash}" alt="Slide {}"><div class="preview-details"><details><summary>Details</summary><p class="label">JPEG SHA-256</p><code class="hash">{}</code></details></div></article>"#,
        index+1,index+1,escape(hash))).collect();
    if slides.is_empty() {
        slides
    } else {
        format!(r#"<div class="preview-grid meta-slides">{slides}</div>"#)
    }
}
