use super::*;

pub(super) fn assets(piece: &Piece) -> String {
    let id = escape(&piece.piece_id);
    match &piece.assets {
        Some(assets) => format!(
            r#"<div class="preview-grid"><article class="preview-card"><div class="preview-label">Image / PNG</div><img class="asset-image" loading="lazy" src="/v1/pieces/{id}/assets/image" alt="First slide for this saved version"><div class="preview-details"><a href="/v1/pieces/{id}/assets/image" target="_blank" rel="noreferrer">Open image</a><details><summary>Details</summary><p class="label">Image SHA-256</p><code class="hash">{}</code></details></div></article><article class="preview-card"><div class="preview-label">Carousel / PDF · {} pages</div><div class="pdf-preview"><iframe title="PDF slideshow" src="/v1/pieces/{id}/assets/carousel#toolbar=0&amp;navpanes=0&amp;view=FitH"></iframe></div><div class="preview-details"><a href="/v1/pieces/{id}/assets/carousel" target="_blank" rel="noreferrer">Open PDF</a><p class="hint">Scroll inside the PDF to inspect every page. If your browser cannot embed PDFs, use the link above.</p><details><summary>Details</summary><p class="label">Carousel SHA-256</p><code class="hash">{}</code></details></div></article></div>{}"#,
            escape(&assets.image_sha256),
            assets.pages,
            escape(&assets.carousel_sha256),
            jpeg_previews(&id, assets)
        ),
        None => String::new(),
    }
}

fn jpeg_previews(id: &str, assets: &syncloom_core::visual::Assets) -> String {
    let slides:String=assets.slides_sha256.iter().enumerate().map(|(index,hash)|format!(
        r#"<article class="preview-card"><div class="preview-label">Meta / JPEG · slide {}</div><img class="asset-image" loading="lazy" src="/v1/pieces/{id}/assets/slides/{index}" alt="Slide {}"><div class="preview-details"><details><summary>Details</summary><p class="label">JPEG SHA-256</p><code class="hash">{}</code></details></div></article>"#,
        index+1,index+1,escape(hash))).collect();
    if slides.is_empty() {
        slides
    } else {
        format!(r#"<div class="preview-grid meta-slides">{slides}</div>"#)
    }
}
