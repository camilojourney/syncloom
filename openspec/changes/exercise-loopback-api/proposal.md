## Request

> Own the independently implementable server API E2E coverage slice and ordinary fixes it proves necessary; the text frontend's Argon design and later implementation are separate assigned tasks.

## Approval

pending

New capability requested: none.
Duplicate kept: none.

## Why

Existing loopback journeys cover platform delivery but omit HTTP refresh success and systematic all-route refusals. A router-complete evidence table prevents helper coverage being mistaken for API or browser proof.

## What Changes

- Extend existing Rust integration targets for missing API cases using temporary state and existing synthetic platform fakes.
- Return the specified 404 for unknown review actions; retain 405 for known POST-only actions.
- Record real TCP results and current consumer mappings outside the worktree; reconcile future pages with their frontend owner.
- Fix only ordinary defects reproduced through these requests. Escalate security, custody, destructive or retired-capability changes before implementation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. Existing studio-api Unknown path and Wrong method requirements stay unchanged; skip_specs is explicit. Checked platform-credentials, Bluesky, X and Threads owners and the router.

## Impact

Server integration tests and page action classification. No authentication, custody, dependency, layout or requirement changes.

## Reuse inventory

Timing: before first edit; findings inventory and broad route/test searches ran before artifacts or code changes.
Search: server routes/handlers, all server tests, core fake platforms, credential/session tests, page form consumers.
Candidates: tests/support::start_studio, social_delivery::setup/login, core support/fake_social, support/social clock.
Decision: extend api.rs and social_delivery.rs; reuse real serve and existing fakes. Do not add another runner.

## Open questions

None. Future frontend mapping remains explicitly pending, not a product assumption.
