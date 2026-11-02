## Request

> LinkedIn: a grant without the posting permission must not show as connected - show 'connected, cannot post: posting permission missing' (or not connected) with a reconnect action, and refuse any post attempt with that reason. Check every other platform for the same two patterns and fix any you find the same way. Reproduce each first with a failing test, then fix.

## Approval

pending

New capability requested: none.
Duplicate kept: none.

## Why

Reduced grants currently appear publish-ready. Threads and Meta also persist requested scopes as if they were granted permissions.

## What Changes

- Evaluate effective permission for the selected publishing action in the existing credential owner.
- Distinguish denied permissions from legacy/unverified evidence; expose fixed reasons and existing sign-in recovery.
- Verify Threads permissions through its documented read-only protocol after exchange and rotation.
- Preserve synthetic success, refresh, containment and no-secret controls.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `platform-credentials`: action readiness, legacy evidence and precise refusal.
- `threads-integration`: effective evidence after exchange/rotation.
- `linkedin-integration`: reduced-consent refusal.
- `x-integration`: optional renewal credentials.
- `dispatch-outbox`: additive permission outcomes.

Checked LinkedIn, Mastodon, X, Bluesky, Meta, Threads, platform-credentials, dispatch-outbox and studio-api owners. Existing platform protocols and dispatch rules otherwise remain authoritative.

## Impact

Core credentials/adapters, additive connection/error/outbox contracts, browser review and loopback fixtures. No new dependencies, accounts, scopes or external requests during testing. Identity and dispatch snapshots remain the parallel account-approval slice's responsibility.

## Reuse inventory

Timing: after inherited first edits; pre-recovery inventories and broad searches are in design.md. Candidates: Token, CredentialSession, Session and inherited posting_grant. Decision: reuse credentials; remove the duplicate registry.

## Open questions

Will Main approve the before/after evidence and specifically authorize landing? Independent consequential-policy challenge remains pending.
