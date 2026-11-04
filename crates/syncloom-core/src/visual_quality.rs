//! Local technical observations, never semantic judgement or an approval gate.
use crate::visual::{AssetHashes, Assets, HEIGHT, WIDTH};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssessmentRequest {
    pub expected_revision: String,
    pub expected_assets: AssetHashes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStatus {
    Satisfied,
    Failed,
    Unassessed,
}

#[derive(Debug, Serialize)]
pub struct Observation {
    pub check: &'static str,
    pub status: ObservationStatus,
    pub observed: String,
}

#[derive(Debug, Serialize)]
pub struct Assessment {
    pub assessor: &'static str,
    pub revision: String,
    pub assets: AssetHashes,
    pub technical_status: ObservationStatus,
    pub checks: Vec<Observation>,
    pub semantic_quality: ObservationStatus,
    pub quality_score: Option<u8>,
    pub limitations: &'static str,
}

fn observation(check: &'static str, satisfied: bool, observed: String) -> Observation {
    Observation {
        check,
        status: if satisfied {
            ObservationStatus::Satisfied
        } else {
            ObservationStatus::Failed
        },
        observed,
    }
}

/// Called only after the existing store has verified every hash under its lock.
pub(crate) fn inspect(assets: &Assets, image: &[u8], carousel: &[u8]) -> Assessment {
    let dimensions = png_dimensions(image);
    let pdf = lopdf::Document::load_mem(carousel);
    let pages = pdf.as_ref().ok().map(|doc| doc.get_pages());
    let geometry: Option<Vec<_>> = pdf.as_ref().ok().map(|doc| {
        doc.get_pages()
            .values()
            .map(|id| page_bounds(doc, *id))
            .collect()
    });
    let expected = [0.0, 0.0, WIDTH as f32 * 0.75, HEIGHT as f32 * 0.75];
    let checks = vec![
        observation(
            "png_decoding",
            dimensions.is_some(),
            if dimensions.is_some() {
                "Decoded full PNG"
            } else {
                "PNG could not be fully decoded within local limits"
            }
            .into(),
        ),
        observation(
            "png_dimensions",
            dimensions == Some((WIDTH, HEIGHT)),
            dimensions.map_or_else(
                || "Unknown dimensions".into(),
                |(w, h)| format!("{w} x {h} px; expected {WIDTH} x {HEIGHT} px"),
            ),
        ),
        observation(
            "pdf_decoding",
            pdf.is_ok(),
            if pdf.is_ok() {
                "Parsed PDF document"
            } else {
                "PDF could not be parsed"
            }
            .into(),
        ),
        observation(
            "pdf_pages",
            pages
                .as_ref()
                .is_some_and(|p| p.len() == assets.pages && !p.is_empty()),
            pages.map_or_else(
                || "Unknown page count".into(),
                |p| format!("{} pages; expected {}", p.len(), assets.pages),
            ),
        ),
        observation(
            "pdf_geometry",
            geometry.as_ref().is_some_and(|pages| {
                !pages.is_empty()
                    && pages
                        .iter()
                        .all(|bounds| bounds.as_deref() == Some(expected.as_slice()))
            }),
            geometry.map_or_else(
                || "Unknown page geometry".into(),
                |pages| {
                    let bounds = pages
                        .iter()
                        .map(|page| {
                            page.as_ref()
                                .map_or_else(|| "unknown".into(), |values| format!("{values:?}"))
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                    format!("Page bounds (pt): {bounds}. Expected every page: {expected:?}")
                },
            ),
        ),
    ];
    Assessment {
        assessor: "local-technical-v1",
        revision: assets.revision.clone(),
        assets: assets.hashes(),
        technical_status: if checks
            .iter()
            .all(|c| c.status == ObservationStatus::Satisfied)
        {
            ObservationStatus::Satisfied
        } else {
            ObservationStatus::Failed
        },
        checks,
        semantic_quality: ObservationStatus::Unassessed,
        quality_score: None,
        limitations: "No vision model ran. Meaning, factual accuracy, readability, visual hierarchy, aesthetics and audience fit are not assessed. JPEG slides are hash-verified only, not decoded. Technical checks are not approval or publishing permission.",
    }
}

fn page_bounds(doc: &lopdf::Document, id: lopdf::ObjectId) -> Option<Vec<f32>> {
    doc.get_object(id)
        .ok()?
        .as_dict()
        .ok()?
        .get(b"MediaBox")
        .ok()?
        .as_array()
        .ok()?
        .iter()
        .map(|n| n.as_float().ok())
        .collect()
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_limits(png::Limits {
        bytes: 16 * 1024 * 1024,
    });
    let mut reader = decoder.read_info().ok()?;
    let size = reader.output_buffer_size()?;
    if size > 16 * 1024 * 1024 {
        return None;
    }
    let info = reader.next_frame(&mut vec![0; size]).ok()?;
    reader.finish().ok()?;
    Some((info.width, info.height))
}
