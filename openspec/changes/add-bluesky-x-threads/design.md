# Design

References: security, data, integrations, ui (Code 5.0.0).

## Context

See proposal.md. Q1 was committed before this change. Decisions and primary-source checks: `docs/decisions/0006-social-platform-integrations.md`.

## Goals / Non-Goals

Keep the existing approval/outbox/grant owners; no scheduling, language routing, engagement or remote hosting. Prove local HTTP journeys, not real access eligibility.

## Decisions

- Use pinned ATrium API 0.26.0 typed clients with the existing bounded reqwest policy, rather than BskyAgent: BskyAgent follows returned DID endpoints, ignores store-write failures and implicitly retries expired-token calls. Explicit refresh keeps storage failures visible and one record-creation request per attempt.
- Preserve Token's public shape and encode platform-specific renewable bundles inside its Secret field, rather than spreading optional JWT/password fields into every adapter. Only TokenStore serializes durable secrets; production uses Keychain and tests use memory.
- Serialize credential commits against a generation invalidated by sign-out/new consent, rather than letting an in-flight exchange resurrect deleted credentials; the red sign-out/refresh race test proves the boundary.
- Keep app passwords in the same private Bluesky bundle as access/refresh JWTs; never re-login on publish. The founder explicitly refreshes expired X/Bluesky sessions; Threads only refreshes unexpired tokens aged 24 hours, rather than hiding network calls behind connection reads.
- Use the shared loopback Session for X PKCE. Threads has no local listener: a one-time state-bound external callback URL is pasted into the protected complete operation, rather than inventing TLS/hosting; configuring the redirect host remains a real-use prerequisite.
- Keep secrets in Keychain or request memory and out of response/debug/log bodies; configured Threads client secret comes from process environment and is never embedded. Meta discourages distributing app secrets in devices, so this personal local server is not a distributable OAuth client.
- Use verified current X paths `/2/media/upload/{id}/append|finalize` and nested metadata, rejecting report-era endpoint assumptions. Critical dependencies have fixed timeouts, no redirects and no automatic mutating retries; an ambiguous result remains unknown.
- One X pricing function owns integer micro-USD estimates (no rounding). URL-like tokens conservatively use the higher link rate, including bare domains; the estimate is never called a bill and requires explicit matching acceptance before dispatch.
- Record estimates in intents and responses with defaults for old records. Repeated terminal intents keep their original quote; no parallel cost ledger or migration is needed.
- Centralize channel limits and conservative counting in Channel/fit rather than duplicating validation in adapters; server rules may still reject text that our conservative checks permit.
- Platform wire types stay in separate modules, while OAuthLifecycle implements X/Threads refresh and connection once. CredentialSession also backs the existing loopback Session, reusing slice 5c's lifecycle and TokenStore dispatch guard. Consent generation travels with each callback so sign-out fences delayed token exchanges. No static-check waiver is retained.
- Separate handler and route-table modules. Meta and social tables call one typed connection-route registrar and one accepted-consent response formatter; application middleware still owns every guard. CLI connection parsing shares configuration and lifecycle owners.

## Security boundaries

Mutating connection/refresh/publish operations require the local session; connection reads and the browser-completion wait follow the existing read-only boundary policy; existing Host, Origin, CSRF and fixed-error guards apply. X callbacks check exact Host and constant-time state; Threads completion checks exact configured redirect and one-time state before any exchange. No provider-controlled URL is fetched: Bluesky keeps the configured PDS and Threads permalinks are output-only and allowlisted. Logs carry fixed route/status labels, never request bodies, credentials, callback codes or provider errors.

## Risks / Trade-offs

- Price changes -> dated estimate, exact confirmation, documented console check before real use; no claimed actual billing.
- Conservative counting/link detection -> possible false-positive refusal or overestimate, never silently truncate or understate obvious link pricing.
- Threads post succeeds but permalink lookup fails -> unknown, never retry blindly.
- No live OAuth eligibility proof -> explicitly report fake-only, with redirect host and real sign-in pending separate authorization.

## Migration Plan

New channels are contained by default; grant and credentials are required separately. Added optional intent/response estimate fields preserve old files. Rollback disables grants; never replay an unknown intent.
