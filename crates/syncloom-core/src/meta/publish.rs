use super::{
    Meta, MetaTarget,
    config::{numeric_id, post_id},
};
use crate::{PostFailure, PostRequest, Posted, Secret, Token};
use serde::Deserialize;
use serde_json::json;
impl Meta {
    pub(super) async fn publish(
        &self,
        target: MetaTarget,
        request: PostRequest,
    ) -> Result<Posted, PostFailure> {
        self.config
            .require()
            .map_err(|_| PostFailure::ConsentRequired)?;
        let token = crate::credentials::usable_token(
            self.store.as_ref(),
            &self.account(),
            &self.clock.now_rfc3339(),
        )?;
        match target {
            MetaTarget::Facebook => self.facebook(&token, request).await,
            MetaTarget::Instagram => self.instagram(&token, request).await,
        }
    }
    async fn facebook(&self, token: &Token, request: PostRequest) -> Result<Posted, PostFailure> {
        let page = self
            .config
            .page_id
            .as_deref()
            .ok_or(PostFailure::ConsentRequired)?;
        let has_image = request.image.is_some();
        let result: PagePost = if let Some(image) = request.image {
            let url = self.media.deliver(image.bytes, "image/png").await?;
            self.post(&format!("{page}/photos"),&token.access_token,json!({"url":url.as_str(),"caption":request.text,"alt_text_custom":image.alt_text,"published":true})).await?
        } else {
            self.post(
                &format!("{page}/feed"),
                &token.access_token,
                json!({"message":request.text}),
            )
            .await?
        };
        // Photo id is not a post id. A photo-only response cannot prove a published post.
        let id = if has_image { result.post_id } else { result.id }.ok_or(PostFailure::Unknown)?;
        if !post_id(&id) {
            return Err(PostFailure::Unknown);
        }
        Ok(Posted {
            url: format!("https://www.facebook.com/{id}"),
            post_id: id,
        })
    }
    async fn instagram(&self, token: &Token, request: PostRequest) -> Result<Posted, PostFailure> {
        if !numeric_id(&token.member_id) {
            return Err(PostFailure::ConsentRequired);
        }
        if !(1..=10).contains(&request.slides.len())
            || request.slides.iter().any(|image| {
                image.bytes.len() > 8 * 1024 * 1024
                    || !image.bytes.starts_with(&[255, 216])
                    || image.alt_text.trim().is_empty()
            })
        {
            return Err(PostFailure::Refused);
        }
        let carousel = request.slides.len() > 1;
        let mut ids = Vec::new();
        for image in request.slides {
            let url = self.media.deliver(image.bytes, "image/jpeg").await?;
            let mut body = json!({"image_url":url.as_str(),"alt_text":image.alt_text});
            if carousel {
                body["is_carousel_item"] = json!(true);
            } else {
                body["caption"] = json!(request.text);
                body["is_ai_generated"] = json!(true);
            }
            let id = self.container(token, body).await?;
            ids.push(id);
        }
        let creation_id = if carousel {
            self.container(token,json!({"media_type":"CAROUSEL","children":ids.join(","),"caption":request.text,"is_ai_generated":true})).await?
        } else {
            ids.remove(0)
        };
        let published: Id = self
            .post(
                &format!("{}/media_publish", token.member_id),
                &token.access_token,
                json!({"creation_id":creation_id}),
            )
            .await?;
        if !numeric_id(&published.id) {
            return Err(PostFailure::Unknown);
        }
        // No fabricated Instagram URL: Graph's id is not its public shortcode.
        let permalink: Permalink = self
            .get(
                &published.id,
                &token.access_token,
                &[("fields", "permalink")],
            )
            .await
            .map_err(|_| PostFailure::Unknown)?;
        let url = crate::config::plain_url(&permalink.permalink).ok_or(PostFailure::Unknown)?;
        if url.scheme() != "https"
            || !matches!(url.host_str(), Some("www.instagram.com" | "instagram.com"))
            || !url.path().starts_with("/p/")
        {
            return Err(PostFailure::Unknown);
        }
        Ok(Posted {
            post_id: published.id,
            url: permalink.permalink,
        })
    }
    async fn container(
        &self,
        token: &Token,
        body: serde_json::Value,
    ) -> Result<String, PostFailure> {
        let container: Id = self
            .post(
                &format!("{}/media", token.member_id),
                &token.access_token,
                body,
            )
            .await?;
        if !numeric_id(&container.id) {
            return Err(PostFailure::Unknown);
        }
        self.wait_ready(&token.access_token, &container.id).await?;
        Ok(container.id)
    }
    async fn wait_ready(&self, token: &Secret, id: &str) -> Result<(), PostFailure> {
        for _ in 0..20 {
            let status: ContainerStatus = self.get(id, token, &[("fields", "status_code")]).await?;
            match status.status_code.as_str() {
                "FINISHED" => return Ok(()),
                "ERROR" | "EXPIRED" => return Err(PostFailure::Refused),
                "IN_PROGRESS" => tokio::time::sleep(std::time::Duration::from_millis(50)).await,
                _ => return Err(PostFailure::Unknown),
            }
        }
        Err(PostFailure::Unknown)
    }
}
#[derive(Deserialize)]
struct Id {
    id: String,
}
#[derive(Deserialize)]
struct PagePost {
    id: Option<String>,
    post_id: Option<String>,
}
#[derive(Deserialize)]
struct ContainerStatus {
    status_code: String,
}
#[derive(Deserialize)]
struct Permalink {
    permalink: String,
}
