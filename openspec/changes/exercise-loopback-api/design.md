# Design

## Context

Original launch 3a059a0; current parent c0584e8. Code 5.4.2 is loaded, newer than the repository's recorded release-tested Code 5.2.1. Current rules govern inventory, honest evidence, residue dispositions and original-base Verify.

References: security, data, integrations, ui; review, design and API security guidance also read. UI visual acceptance remains the frontend owner's gate, not an assertion from these HTTP tests.

## Goals / Non-Goals

Prove real TCP route behavior without personal state or external delivery. No new UI, runner, accounts, registrations, redirect hosts, custody changes or live social traffic.

## Decisions

Extend existing integration targets, rather than a shell/API wrapper: they already launch real serve on loopback and inject only external provider/credential boundaries. Keep platform-specific success assertions in social_delivery and generic boundary enumeration in api.

Use the existing mutable test clock for Threads' 24-hour refresh window rather than altering production clock or refresh rules. Record success separately from signed-out, malformed, configuration and auth refusals.

Keep the router authoritative; manually reconcile all route registrations plus dynamic platform/action variants and Axum HEAD companions against the endpoint table. Do not add generated routes or a second runtime router registry just for inventory.

A real GET of an unknown review action reproduced 405 where studio-api requires 404. Parse the existing action vocabulary once for GET and POST rather than copy a second allowlist into the editor; known POST-only actions still produce 405. Keep rendering, form validation and cost consent unchanged.

## Security boundaries

Only the local single-founder loopback client may mutate using the current session token after Host/Origin/fetch-site checks. Tests supply synthetic credentials to existing local fakes and in-memory TokenStore; no personal Keychain inspection, enumeration, read, write or deletion. Logs/evidence must not contain session tokens or credential values. Existing auth policy is unchanged; no success is inferred from a refusal.

## Risks / Trade-offs

Real rendering depends on the pinned installed browser and font; disclose failures rather than skip assets. Full workspace tests include isolated synthetic Keychain fixtures, which must be reviewed before running. Screenshot/browser quality belongs to the frontend owner. Most expansion protects existing correct behavior, so no red failure is invented for those cases. The unknown-action regression failed through real TCP before the production fix; preserve that log and rerun all page/consent journeys after the shared parser change.
