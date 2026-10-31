## Request

> Main answer to x-consent-script: script-free - keep CSP script-src 'none' and the landed no-script rule. Replace the checkbox with a server consent step: an 'Accept estimated cost' form POST that records consent bound to the exact revision and the displayed estimate and pricing date; the review page then renders Publish enabled, and before that renders it with the real disabled attribute in the same disabled style as the Threads image-unavailable Publish. A new revision or a changed estimate clears the consent. Keep the server-side refusal. Add the OpenSpec delta for the consent step (proposal.md starts with exactly '## Request'), tests for record, clear-on-revision and refusal, and the before/after screenshots.

## Approval

pending

New capability requested: none.
Duplicate kept: none.

## Why

Review screens expose internal names, split a pricing date and imply X can publish before cost acceptance.

## What Changes

- Use channel-owned platform display names and keep the pricing date together.
- Replace X's checkbox with a separate script-free consent POST, bound to the saved revision and current dated estimate.
- Render Publish natively disabled until consent and existing approval requirements hold.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-api`: browser consent operation and proper platform identity. Checked studio-api, platform-rules, dispatch-outbox and the migrated review-screen owner; no unmigrated owner overlaps.

## Impact

Rust-rendered pages and server-session UI state only. CLI/API cost confirmation and core dispatch refusal stay unchanged. No new dependencies or connection-screen changes.

## Open questions

None.
