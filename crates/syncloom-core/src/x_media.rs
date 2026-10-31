//! Current v2 PNG upload and nested alt metadata, verified against X OpenAPI 2.173.
use crate::{
    PostFailure, Secret,
    platform::{PostImage, request_empty, request_json},
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Media {
    data: MediaData,
}
#[derive(Deserialize)]
struct MediaData {
    id: String,
    processing_info: Option<Processing>,
}
#[derive(Deserialize)]
struct Processing {
    state: String,
}
#[expect(
    clippy::expect_used,
    reason = "The fixed image/png MIME literal is valid."
)]
pub(crate) async fn upload(
    client: &reqwest::Client,
    base: &str,
    token: &Secret,
    image: PostImage,
) -> Result<String, PostFailure> {
    let initialized:Media=request_json(client.post(format!("{base}/2/media/upload/initialize")).bearer_auth(token.expose()).json(&serde_json::json!({"total_bytes":image.bytes.len(),"media_type":"image/png","media_category":"tweet_image"}))).await?;
    let id = initialized.data.id;
    if !crate::social::numeric_id(&id) {
        return Err(PostFailure::Unknown);
    }
    // Units: segments are at most one MiB, with consecutive zero-based indices.
    for (index, chunk) in image.bytes.chunks(1024 * 1024).enumerate() {
        let form = reqwest::multipart::Form::new()
            .text("segment_index", index.to_string())
            .part(
                "media",
                reqwest::multipart::Part::bytes(chunk.to_vec())
                    .file_name("image.png")
                    .mime_str("image/png")
                    .expect("fixed MIME"),
            );
        request_empty(
            client
                .post(format!("{base}/2/media/upload/{id}/append"))
                .bearer_auth(token.expose())
                .multipart(form),
        )
        .await?;
    }
    let finalized: Media = request_json(
        client
            .post(format!("{base}/2/media/upload/{id}/finalize"))
            .bearer_auth(token.expose()),
    )
    .await?;
    if finalized.data.id != id
        || finalized
            .data
            .processing_info
            .is_some_and(|p| p.state != "succeeded")
    {
        return Err(PostFailure::Unknown);
    }
    request_empty(
        client
            .post(format!("{base}/2/media/metadata"))
            .bearer_auth(token.expose())
            .json(&serde_json::json!({"id":id,"metadata":{"alt_text":{"text":image.alt_text}}})),
    )
    .await?;
    Ok(id)
}
