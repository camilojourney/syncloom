use super::*;
use std::{path::Path, sync::Arc};
use syncloom_core::{Settings, Store, Studio};

async fn reopen(
    dir: &Path,
    provider: Arc<Scripted>,
) -> Result<(String, String, tokio::task::JoinHandle<std::io::Result<()>>)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let studio = Studio::new(
        Store::open(dir)?,
        provider,
        fixed_clock(),
        Settings::default(),
    );
    let task = tokio::spawn(syncloom_server::serve(listener, Arc::new(studio)));
    let session: Value = client()
        .get(format!("{base}/v1/session"))
        .send()
        .await?
        .json()
        .await?;
    Ok((
        base,
        session["token"].as_str().ok_or("missing token")?.into(),
        task,
    ))
}
async fn send(base: &str, token: &str, path: &str, body: &Value) -> Result<Answer> {
    Ok(answer(
        client()
            .post(format!("{base}{path}"))
            .header("x-syncloom-token", token)
            .json(body)
            .send()
            .await?,
    )
    .await)
}

#[tokio::test]
async fn snapshot_confirmation_and_incomplete_claim_survive_real_server_restart() -> Result {
    let dir = tempfile::TempDir::new()?;
    let provider = Scripted::new(text("A pending restarted draft."));
    let (base, old_token, task) = reopen(dir.path(), provider.clone()).await?;
    let captured = answer(
        client()
            .post(format!("{base}/v1/capture/attachment"))
            .header("x-syncloom-token", &old_token)
            .multipart(form(FILE, "restart.md", None)?)
            .send()
            .await?,
    )
    .await;
    assert_eq!(captured.status, 200);
    let selected = selection(&captured.body);
    let preview = send(&base, &old_token, "/v1/capture/preview", &selected).await?;
    let mut confirm = selected;
    confirm["expected_revision"] = preview.body["revision"].clone();
    task.abort();
    let _ = task.await;
    assert!(provider.requests().is_empty());
    let (base, token, task) = reopen(dir.path(), provider.clone()).await?;
    assert_ne!(token, old_token);
    assert_eq!(
        send(&base, &old_token, "/v1/capture/confirm", &confirm)
            .await?
            .code(),
        "session_required"
    );
    let drafted = send(&base, &token, "/v1/capture/confirm", &confirm).await?;
    assert_eq!(drafted.status, 201);
    assert_eq!(
        send(&base, &token, "/v1/capture/confirm", &confirm)
            .await?
            .body,
        drafted.body
    );
    let revision = preview.body["revision"]
        .as_str()
        .ok_or("missing revision")?;
    assert_eq!(
        client()
            .get(format!("{base}/capture/result/{revision}"))
            .send()
            .await?
            .status(),
        200
    );
    // A durable interrupted claim must not become permission to call the provider after restart.
    let mut second = confirm.clone();
    second["channels"] = json!(["facebook_page"]);
    let second_preview = send(&base, &token, "/v1/capture/preview", &second).await?;
    // Preview is strict: remove the confirm-only scalar for the new selection.
    assert_eq!(second_preview.code(), "invalid_request");
    let mut second_selection = selection(&captured.body);
    second_selection["channels"] = json!(["facebook_page"]);
    let second_preview = send(&base, &token, "/v1/capture/preview", &second_selection).await?;
    assert_eq!(second_preview.status, 200);
    let interrupted = second_preview.body["revision"]
        .as_str()
        .ok_or("missing revision")?;
    std::fs::write(
        dir.path()
            .join("capture")
            .join(format!("{interrupted}.json")),
        serde_json::to_vec(
            &json!({"revision":interrupted,"channels":["facebook_page"],"group_id":"","piece_ids":[]}),
        )?,
    )?;
    second_selection["expected_revision"] = json!(interrupted);
    task.abort();
    let _ = task.await;
    let (base, token, task) = reopen(dir.path(), provider.clone()).await?;
    assert_eq!(
        send(&base, &token, "/v1/capture/confirm", &second_selection)
            .await?
            .code(),
        "revision_conflict"
    );
    assert_eq!(provider.requests().len(), 1);
    assert_eq!(
        client()
            .get(format!("{base}/capture/result/{revision}"))
            .send()
            .await?
            .status(),
        200
    );
    task.abort();
    let _ = task.await;
    Ok(())
}
