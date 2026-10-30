#[tokio::test]
async fn unsupported_glyphs_are_refused_instead_of_using_unpinned_fallback_fonts() {
    let config = syncloom_core::renderer_config(|n| std::env::var(n).ok());
    let content = syncloom_core::RenderRequest {
        expected_revision: String::new(),
        template: "paper".into(),
        slides: vec!["A unicorn: 🦄".into()],
    }
    .content()
    .unwrap();
    assert!(matches!(
        config.render(&content).await,
        Err(syncloom_core::StudioError::InvalidRequest)
    ));
}

#[tokio::test]
async fn markup_is_literal_text_and_never_fetches_a_url() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/image", listener.local_addr().unwrap());
    let hits = Arc::new(AtomicUsize::new(0));
    let count = hits.clone();
    let task = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            count.fetch_add(1, Ordering::SeqCst);
            let mut buf = [0; 4096];
            let _ = stream.read(&mut buf).await;
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length:0\r\n\r\n")
                .await;
        }
    });
    let config = syncloom_core::renderer_config(|n| std::env::var(n).ok());
    let text = format!("<img src='{url}'><script>fetch('{url}')</script>");
    let content = syncloom_core::RenderRequest {
        expected_revision: String::new(),
        template: "paper".into(),
        slides: vec![text],
    }
    .content()
    .unwrap();
    let result = config.render(&content).await;
    task.abort();
    let rendered = result.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 0);
    let doc = lopdf::Document::load_mem(&rendered.carousel).unwrap();
    assert!(doc.extract_text(&[1]).unwrap().contains("<img"));
}
