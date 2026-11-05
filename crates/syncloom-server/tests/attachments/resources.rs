use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn held_upload(server: &TestServer) -> Result<tokio::net::TcpStream> {
    let mut stream = tokio::net::TcpStream::connect(server.addr).await?;
    stream.write_all(format!("POST /v1/capture/attachment HTTP/1.1\r\nHost: {}\r\nX-Syncloom-Token: {}\r\nX-Request-Id: fixture-request-id\r\nContent-Type: multipart/form-data; boundary=held\r\nContent-Length: 1024\r\nExpect: 100-continue\r\nConnection: close\r\n\r\n",server.host(),server.token).as_bytes()).await?;
    // Hyper sends Continue only once the guarded handler starts reading this body.
    let mut head = [0; 25];
    stream.read_exact(&mut head).await?;
    assert_eq!(&head, b"HTTP/1.1 100 Continue\r\n\r\n");
    Ok(stream)
}

#[tokio::test]
async fn upload_deadline_and_two_reader_limit_release_capacity_without_content() -> Result {
    let provider = Scripted::new(text("Never called."));
    let server = start(provider.clone()).await;
    let started = std::time::Instant::now();
    let mut one = held_upload(&server).await?;
    let two = held_upload(&server).await?;
    let busy = upload(&server, FILE, "note.md", None).await?;
    assert_eq!(busy.status, 503);
    assert_eq!(busy.code(), "attachment_busy");
    assert_eq!(busy.as_fixture(), fixture("errors/attachment_busy.json"));
    let mut bytes = Vec::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(15),
        one.read_to_end(&mut bytes),
    )
    .await??;
    let response = String::from_utf8(bytes)?;
    assert!(response.starts_with("HTTP/1.1 408"), "{response}");
    assert!(response.contains("attachment_timeout"));
    let first = response.find('{').ok_or("missing error JSON")?;
    let last = response.rfind('}').ok_or("missing error JSON")?;
    let body: Value = serde_json::from_str(&response[first..=last])?;
    assert_eq!(
        json!({"status":408,"body":body}),
        fixture("errors/attachment_timeout.json")
    );
    assert!(started.elapsed() >= std::time::Duration::from_secs(9));
    drop(two);
    assert!(provider.requests().is_empty());
    assert!(files(&server.dir, "capture").is_empty());
    assert_eq!(upload(&server, FILE, "note.md", None).await?.status, 200);
    Ok(())
}

#[tokio::test]
async fn unsupported_upload_methods_keep_the_existing_method_not_allowed_contract() -> Result {
    let server = start(Scripted::new(text("Never called."))).await;
    let response = answer(
        client()
            .delete(server.url("/v1/capture/attachment"))
            .header("x-syncloom-token", &server.token)
            .send()
            .await?,
    )
    .await;
    assert_eq!(response.status, 405, "{}", response.raw);
    assert_eq!(response.code(), "method_not_allowed");
    Ok(())
}
