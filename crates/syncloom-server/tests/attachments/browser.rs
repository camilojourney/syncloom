use super::*;

#[tokio::test]
async fn native_browser_crlf_serialization_preserves_the_lf_attachment_snapshot() -> Result {
    let provider = Scripted::new(text("Never called."));
    let server = start(provider.clone()).await;
    let captured = upload(&server, FILE, "release.md", None).await?;
    let receipt = captured.body["source_revision"]
        .as_str()
        .ok_or("missing receipt")?;
    for source in [EXTRACTED.to_owned(), EXTRACTED.replace('\n', "\r\n")] {
        let response = answer(
            client()
                .post(server.url("/capture/preview"))
                .form(&[
                    ("csrf", server.token.as_str()),
                    ("source_revision", receipt),
                    ("text", source.as_str()),
                    ("channels", "linkedin_text"),
                ])
                .send()
                .await?,
        )
        .await;
        assert_eq!(
            response.status, 200,
            "native line ending representation must preserve receipt binding"
        );
        assert!(response.raw.contains("Confirm and draft"));
    }
    assert!(provider.requests().is_empty());
    Ok(())
}

#[tokio::test]
async fn tampered_attachment_provenance_cannot_reuse_the_original_receipt() -> Result {
    let provider = Scripted::new(text("Never called."));
    let server = start(provider.clone()).await;
    let captured = upload(&server, FILE, "release.md", None).await?;
    let path = std::fs::read_dir(server.dir.path().join("capture"))?
        .next()
        .ok_or("missing snapshot")??
        .path();
    let mut record: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    record["source"]["attachment"]["byte_count"] = json!(1);
    std::fs::write(path, serde_json::to_vec(&record)?)?;
    assert_eq!(
        post(&server, "/v1/capture/preview", selection(&captured.body))
            .await
            .code(),
        "revision_conflict"
    );
    assert!(provider.requests().is_empty());
    Ok(())
}

#[tokio::test]
async fn changed_source_text_cannot_borrow_an_attachment_receipt_in_suggestions() -> Result {
    let server = start(Scripted::new(text("Never called."))).await;
    let captured = upload(&server, FILE, "release.md", None).await?;
    let receipt = captured.body["source_revision"]
        .as_str()
        .ok_or("missing receipt")?;
    let response = answer(
        client()
            .post(server.url("/capture/suggest"))
            .form(&[
                ("csrf", server.token.as_str()),
                ("source_revision", receipt),
                ("text", "Different source text with borrowed provenance."),
            ])
            .send()
            .await?,
    )
    .await;
    assert_eq!(response.status, 409, "{}", response.raw);
    Ok(())
}

#[tokio::test]
async fn native_attachment_pages_show_exact_source_provider_input_and_safe_recovery() -> Result {
    let provider = Scripted::new(text("A pending local draft."));
    let server = start(provider.clone()).await;
    let home = get(&server, "/").await;
    assert!(home.raw.contains("enctype=\"multipart/form-data\""));
    assert!(home.raw.contains("Extract file only"));
    assert!(
        home.raw
            .contains("The complete filename is shown after local extraction.")
    );
    let captured = answer(
        client()
            .post(server.url("/capture/attachment"))
            .multipart(
                form(FILE, "release.MD", Some("text/markdown"))?.text("csrf", server.token.clone()),
            )
            .send()
            .await?,
    )
    .await;
    assert_eq!(captured.status, 200, "{}", captured.raw);
    for expected in [
        "Extracted file text",
        "release.MD",
        "Original bytes",
        "Source provenance",
        "&lt;script&gt;words&lt;/script&gt;",
    ] {
        assert!(
            captured.raw.contains(expected),
            "{expected}: {}",
            captured.raw
        );
    }
    assert!(!captured.raw.contains("<script>"));
    let files = std::fs::read_dir(server.dir.path().join("capture"))?
        .collect::<std::io::Result<Vec<_>>>()?;
    assert_eq!(files.len(), 1);
    let receipt: Value = serde_json::from_slice(&std::fs::read(files[0].path())?)?;
    let revision = receipt["revision"].as_str().ok_or("missing receipt")?;
    let fields = [
        ("csrf", server.token.as_str()),
        ("text", EXTRACTED),
        ("source_revision", revision),
    ];
    let empty = answer(
        client()
            .post(server.url("/capture/preview"))
            .form(&fields)
            .send()
            .await?,
    )
    .await;
    assert_eq!(empty.status, 400);
    assert!(empty.raw.contains("Select at least one"));
    assert!(empty.raw.contains("release.MD"));
    let mut fields = fields.to_vec();
    fields.push(("channels", "linkedin_text"));
    let preview = answer(
        client()
            .post(server.url("/capture/preview"))
            .form(&fields)
            .send()
            .await?,
    )
    .await;
    assert_eq!(preview.status, 200);
    assert!(preview.raw.contains("Text sent to the drafting provider"));
    assert!(preview.raw.contains("not the file bytes or filename"));
    assert!(preview.raw.contains("Not an approval"));
    assert!(
        preview
            .raw
            .contains("The normalized text shown above will be sent")
    );
    assert!(provider.requests().is_empty());
    let stale_revision = "a".repeat(64);
    fields.push(("expected_revision", stale_revision.as_str()));
    let stale = answer(
        client()
            .post(server.url("/capture/confirm"))
            .form(&fields)
            .send()
            .await?,
    )
    .await;
    assert_eq!(stale.status, 409);
    assert!(
        stale.raw.contains(
            "That capture preview no longer matches its source or selected destinations."
        )
    );
    assert!(
        stale
            .raw
            .contains("A previous confirmation may already have created drafts")
    );
    assert!(provider.requests().is_empty());
    for (bytes, name, code) in [
        (FILE, "image.pdf", "attachment_unsupported"),
        (b"seven!!".as_slice(), "tiny.txt", "thought_too_short"),
    ] {
        let error = answer(
            client()
                .post(server.url("/capture/attachment"))
                .multipart(form(bytes, name, None)?.text("csrf", server.token.clone()))
                .send()
                .await?,
        )
        .await;
        assert_eq!(error.status, 422);
        assert!(error.raw.contains(code));
        assert!(error.raw.contains("No new draft"));
        assert!(!error.raw.contains("open the piece to check what was saved"));
    }
    Ok(())
}
