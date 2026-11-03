# Design

## Context

See proposal.md. Old `capture.py` suggested routes, previewed drafts, then approved and published. v2 already drafts through `Studio::create_thought` and reviews on `/review/{piece_id}`.

## Goals / Non-Goals

**Goals:**
- Text-only suggestions, a revision-bound preview, and confirm that only calls `create_thought`.
- One confirmation record per preview revision.

**Non-Goals:**
- Attachment classification, URL fetch, connection probes, clipboard reading, new adapters, or a second thought composer.
- Approve, grant, schedule, or publish on confirm.

## Decisions

1. Put suggestion and revision rules in `syncloom-core`, and keep HTTP/HTML thin. Rejected a server-only helper: the CLI and API must share one owner.
2. Confirm calls `Studio::create_thought` and stores no draft text of its own. Rejected saving the trimmed founder text as the piece: `thought-drafting` requires a provider draft, and a parallel writer would copy that rule.
3. Map old `twitter` to channel `x` and old `facebook` to `facebook_page`. Rejected retaining those old names: they are not `platform-rules` channels.
4. Do not truncate suggestion text. Rejected the old ellipsis trim: a shortened preview could be mistaken for the text confirm sends, and `platform-rules` does not truncate.
5. Revision hashes a version tag, byte-length-framed exact text and sorted channel names. Rejected newline-only separation: source text can contain channel names and alias another selection.
6. Reserve an incomplete confirmation under the store lock before calling `create_thought`, then complete it with piece pointers. A process mutex serializes concurrent requests; restart or partial-save leftovers conflict rather than draft again. Only documented provider failures clear a claim, because they persist nothing.
7. Reuse the workspace composer with an alternate Suggest destinations first button. Rejected a separate capture composer: the request explicitly forbids a second thought form. GET /capture returns to that composer; later pages display the exact source without an editor.

## Prelanding header correction

Main1436 (corr=69cc6c06c62e7f02) found the existing narrow-header CSS hid Workspace. The owned-server 390px reproduction showed `display: none` and a zero-sized link. Remove that rule from the existing shared header owner and allow wrapping when the header cannot fit; do not duplicate navigation in capture pages. Keep the existing draft-only copy unchanged. The HTTP stylesheet regression guards the hiding declaration only; actual computed visibility, keyboard return, both themes and viewport bounds require browser proof. Inventory before correction edits reported 0 candidates; broad searches for workspace-link, topbar, header and viewport found page/mod.rs and studio.css as the sole shared owners. No new dependency or policy.

## Intentional independent owners

Kept: `crates/syncloom-server/src/page/capture.rs::result` and `crates/syncloom-server/src/page/review.rs::review`, because their five-line Axum signature/path-extraction match is framework boilerplate, not a duplicated domain rule. Capture resolves a confirmation revision to its saved draft group; review inspects a piece and its publish state. A forwarding wrapper or cosmetic signature churn would hide independent owners without sharing a rule. The exact capture-side fragment is below; the tokenizer normalizes handler/local identifiers. Retain the raw identity and counts in Verify/residue reporting. This engineering disposition is not human approval or a claim of zero automated findings. No detector settings or checks are changed.

```rust
pub(crate) async fn result(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Html<String>, Failure> {
    let Path(revision) = path.map_err(|_| Failure::NotFound)?;
```

## Risks / Trade-offs

- [Interrupted confirmation] → Its durable claim refuses another provider call. Reopen saved pieces to inspect what exists; automatic recovery of an incomplete claim is outside this slice.
- [URL text still produces a link hint] → The hint is local. Nothing is fetched.
- [Low-confidence Facebook suggestions remain] → They are migrated behavior, not a new route. The founder must select them.

## Security boundaries

- Who may act: the local founder, through the existing loopback Host, Origin, and session token. No new permission owner.
- What is checked: session before mutation; channels must be a suggested subset; `expected_revision` must match the recomputed digest; confirmation filenames are digests, never caller paths; loaded piece ids are checked before use.
- What is logged: fixed route labels, request id, status. Submitted text and the session token are not logged.
- Where secrets live: unchanged. Confirm uses the existing provider adapter and fake in tests. No new key, grant, or outbound client.
- Evidence: completed retries survive a reopened core; an incomplete-claim fixture refuses duplicate drafting. This is not full process-kill or disk-failure injection. Boundary only adds the capture page family to existing form classification; Host, Origin and token policy is unchanged.

## Migration Plan

Update the capture row to partly migrated: text suggest, preview and confirm-to-draft are rebuilt; attachment intake and URL fetch stay deferred, and confirm-and-publish is deliberately not ported. Rollback is reverting this change. No data migration.

## Open Questions

None.

References: security, data, llm, ui.
