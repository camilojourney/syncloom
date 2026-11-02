## Request

> [fm-from-firstmate]⁣corr=2c253426f2bf82e4 Main1381 - combined candidate 7020bcdf reviewed. Security and verification: accepted (both exact-tree Verify runs, the seven account controls, 15 X cases, rotation and joint controls, both sensitivity mutations, exactly the authorized retained findings). Main opened switched light 1440, missing dark 390, unverified light 1440 and fresh dark 390. Three visible fixes before Main puts it to the captain, no security logic change:

> 1. Developer wording in the publish panel. "connected, cannot post: posting permission missing. Sign in again to reconnect." and "connected, cannot post: posting permission could not be verified. ..." are raw status text, lowercase. Write them as plain sentences that name the platform, for example: "Your X account is connected, but Syncloom isn't allowed to post to it. Reconnect and allow posting." and "Syncloom couldn't confirm it's allowed to post to your Threads account. Reconnect to check again." Same plain style for the unconfirmed-identity reason.
> 2. Fresh approval date. The fresh-approval state still reads "Approved on 2026-10-30", the same date as the old approval it replaced. Show the new approval's own date. If the fixture clock is fixed at 10-30, move it forward for the fresh approval so the capture proves the date changes, and add an assertion that it does.
> 3. Disabled Publish in light theme (pale green fill, dark label) reads as clickable. Make the disabled state look disabled in both themes, keeping contrast for the label, and keep the reason text next to it.

> Then: commit as a new head (7020bcdf stays preserved), rerun both exact-tree Verify runs, recapture all 16 with the same route, and send Main the head and images. Report any new finding separately.

## Approval

pending

New capability requested: none.
Duplicate kept: none.

## Why

The accepted security candidate still shows raw readiness text, a fresh approval date that matches the old one, and a disabled Publish control that looks clickable.

## What Changes

- Project stable readiness reasons and the existing platform label into plain review sentences. Keep API reason keys, messages and reconnect operations unchanged.
- Prove a later approval displays its own recorded instant when the controlled studio clock moves. Do not change approval policy if that owner already does.
- Restyle only the native disabled button so it is inactive in both themes, without hiding the reason or reconnect control.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-api`: browser readiness sentences and inactive disabled Publish.
- `draft-review`: displayed approval instant matches that approval event.

Checked studio-api, draft-review, platform-credentials, dispatch-outbox and the posting-permission decision. API refusal contracts stay with those owners.

## Impact

Review copy, served stylesheet and docs. No credential, store, OAuth or dispatch policy change.

## Open questions

None.
