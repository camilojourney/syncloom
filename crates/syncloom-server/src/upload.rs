//! One multipart adapter. Boundary authenticates it before any core acquisition.
use crate::{failure::Failure, server::read_bounded};
use axum::{
    body::Body,
    extract::{FromRequest, Multipart, Request},
};
use syncloom_core::StudioError;

#[derive(Clone)]
pub(crate) struct Upload {
    pub filename: String,
    pub mime: Option<String>,
    pub bytes: Vec<u8>,
    pub csrf: Vec<String>,
}

pub(crate) fn is_path(path: &str) -> bool {
    matches!(path, "/capture/attachment" | "/v1/capture/attachment")
}

async fn file_part(
    field: axum::extract::multipart::Field<'_>,
) -> Result<(String, Option<String>, Vec<u8>), Failure> {
    let filename = field
        .file_name()
        .ok_or(StudioError::InvalidRequest)?
        .to_owned();
    let mime = field.content_type().map(str::to_owned);
    let bytes = field
        .bytes()
        .await
        .map_err(|_| StudioError::InvalidRequest)?
        .to_vec();
    Ok((filename, mime, bytes))
}

pub(crate) async fn read(request: Request, page: bool) -> Result<(Request, Upload), Failure> {
    let (parts, body) = request.into_parts();
    let bytes = read_bounded(body).await?;
    let mut parser_request = Request::new(Body::from(bytes));
    *parser_request.headers_mut() = parts.headers.clone();
    let invalid = || Failure::from(StudioError::InvalidRequest);
    let multipart = Multipart::from_request(parser_request, &())
        .await
        .map_err(|_| invalid())?;
    Ok((
        Request::from_parts(parts, Body::empty()),
        fields(multipart, page).await?,
    ))
}

async fn fields(mut multipart: Multipart, page: bool) -> Result<Upload, Failure> {
    let invalid = || Failure::from(StudioError::InvalidRequest);
    let mut file = None;
    let mut csrf = Vec::new();
    while let Some(field) = multipart.next_field().await.map_err(|_| invalid())? {
        match field.name() {
            Some("file") if file.is_none() => {
                file = Some(file_part(field).await?);
            }
            Some("csrf") if page && field.file_name().is_none() => {
                csrf.push(field.text().await.map_err(|_| invalid())?);
            }
            _ => return Err(invalid()),
        }
    }
    let (filename, mime, bytes) = file.ok_or_else(invalid)?;
    Ok(Upload {
        filename,
        mime,
        bytes,
        csrf,
    })
}
