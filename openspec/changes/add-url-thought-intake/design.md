# Design

## Context

The adopted Rust plan section 1.4 and migration ledger defer URL retrieval, not the existing text/capture loop. Read old thought_pipeline.py:384-517, research/sources/base.py:115-230 and docs/lineage.md at pinned 3efcd35370242a37985fbacb272dacf80b2d42dd read-only. Old behavior preserves raw URL and extracted text, validates/pins every redirect and excludes URLs/content from lineage.

References: security, data, integrations, llm, ui. Installed Code 5.4.4; strategic design and review references also read.

## Goals / Non-Goals

One explicitly requested URL becomes an immutable capture snapshot, selected preview and pending drafts with truthful provenance. No crawler, Research Radar, background intake, uploads, account approval, grants, schedules, publishing or new editor/source/event store.

## Decisions

- Firstmate's url-host-allowlist resolution chooses an operator exact-host allowlist empty by default. Rejected old any-public-host behavior because installed Code security rule 7 requires an allowlist. This is not captain approval of hosts or landing.
- Source retrieval owns URL syntax, per-hop resolution/pinning and bounded text extraction. Rejected reusing config::http_client directly: platform/provider destinations are fixed, but sources require no proxy, cookies, credentials, decompression or implicit redirects. Reqwest's existing rustls transport remains the HTTP owner.
- Allow HTTP/HTTPS on standard ports; refuse userinfo, query and fragments to avoid credential-bearing links. Normalize configured ASCII DNS hosts case-insensitively, forbid IPs/wildcards/trailing dots/empty entries. Revalidate all addresses and redirects, reject mixed answers and HTTPS downgrade. Rejected credential reuse and forwarding headers.
- Use html5ever's pinned tokenizer, not old regex stripping; discard non-content containers and decode entities without executing/subresource fetching. Reject oversized/unsupported content rather than silently truncating it. One ten-second deadline includes DNS, hops and body; no retries.
- Extend existing Source and capture records for the server-owned retrieved snapshot. Receipt digests cover the complete provenance, and previews bind it plus selected channels. Rejected client-asserted provenance and refetch-on-confirm: neither proves what the founder saw. Existing capture claim and create-thought persistence own idempotence and pending drafts.
- Extracted text remains user-message data, never system/tool instructions. Confirmation explicitly consents to provider transfer; retrieval grants no other permission. Provider output still requires founder review.
- Use compile-time-only test-source support to map two fixed fixture hosts/public DNS answers to one owned loopback server. No production runtime bypass, configuration switch or environment exception. Real browser/API proof exercises the real server but is not public-Internet compatibility proof.
- Preserve incumbent Impeccable 4.8.0 Operate identity and native navigation/loading. Archive lacks its launcher/detector; manual read-only critique/audit is degraded single-context assessment, not independent review.

## Security boundaries

Who: existing single local founder session, Host and Origin guards before every mutation. Cross-session requests fail and neither server changes.
Checks: exact allowlist; syntax/port checks before DNS; all DNS answers public; pinned sockets; each hop rechecked; bounded UTF-8 text; source receipt and exact selection before provider calls; no client-supplied source metadata.
Logs: fixed routes/codes/timings only. Provenance keeps existing ids/hashes/status/usage, not URL or content. Source bodies/provenance exist only in the existing local capture/piece records.
Secrets: none added. Fetch headers contain only fixed product agent/accept values; no personal contacts, authorization, cookies or referrers. Proof uses a fake drafting provider and no Keychain/account path.

## Risks / Trade-offs

A public host can serve malicious or misleading text. Structural separation and explicit consent do not guarantee resistance to prompt injection or factual accuracy; no tools or publication authority are granted. Extracted HTML is broad visible text, not a generic article/readability crawler. Authenticated fetching and JavaScript rendering are unsupported; a public login/shell response can yield its own text rather than the intended article. Compressed/non-UTF-8/large responses fail explicitly. DNS policy is conservative and may reject special-use legitimate hosts. An allowlisted public server may itself proxy content; this app cannot establish upstream truth. Captures retain local source content under the existing data-directory privacy boundary; no new retention/deletion system is invented.

## Security self-review findings

The first tokenizer implementation cleared all nested hidden containers on a mismatched closing tag, revealing hidden instructions. A real-server controlled fixture reproduced 200 with that text instead of 422. Fixed independent bounded counters retain hidden content and keep processing linear; the regression remains. Reqwest's pinned 0.12.28 source documents automatic protocol retries, so this client explicitly uses retry::never rather than assuming the existing client default meant no retries. html5ever is pinned at 0.40.1, the current version reported by Cargo metadata during this build; no custom HTML regex/parser was ported.

Security self-review also mutation-tested Host, Origin, Fetch-Site, session token, form CSRF, exact-host allowlisting, mixed DNS answers, snapshot text, selection revision and TLS downgrade checks, observing selected regressions fail before restoring each guard. Host/session/receipt denial includes two isolated running servers with separate stores and fake providers. TLS downgrade is a pure redirect-policy unit proof, not an Internet TLS interoperability claim.

Manual Impeccable review found the generic error footer told URL-fetch users to open a nonexistent piece. A failing real-server browser-response assertion reproduced this. Source errors now explicitly say the fetch created no draft and offer workspace recovery; unrelated action errors keep their existing saved-piece guidance.

## Spec-Impact

Adds URL/source receipt behavior to thought-drafting and guarded URL/receipt operations to studio-api. Existing capture owns channel selection, preview, claim idempotence and confirm-to-pending semantics; its text-only scenario continues never opening links. Provenance-log's event schema and privacy policy remain unchanged. No baseline, old specifications, independent reviewer, publishing permission or security landing approval changes.

## Migration Plan

Update only thought-intake/capture rows to the actual narrower URL behavior, retaining unrelated deferred/retired work. Historical text records deserialize without new optional provenance. No old repository/data changes. Landing still requires exact-head captain decision through Main and four reviewed final captures.

## Open Questions

None for implementation. Landing/image prerequisites remain unresolved.
