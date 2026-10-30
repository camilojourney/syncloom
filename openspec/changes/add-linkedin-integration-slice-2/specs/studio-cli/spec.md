## Purpose

Own the command line `syncloom`: the same core functions as the HTTP API, with the same error codes and messages, for the operator and for agents working in a shell.

## MODIFIED Requirements

### Requirement: Commands
The CLI SHALL offer `thought`, `show`, `approve`, `publish` and `linkedin sign-in|status|sign-out`, each printing the core's JSON result on stdout and exiting 0.

#### Scenario: The four commands
- **WHEN** `syncloom thought --text <thought> --channel linkedin_text` runs with a reachable provider
- **THEN** stdout is the same `{"group_id","pieces"}` JSON the API returns, and `show <piece_id>` prints that piece
- **AND** `approve <piece_id> --expected-revision <revision>` prints the approved piece
- **AND** `publish <piece_id> --expected-revision <revision> --dry-run` prints the publish result with `status` `dry_run`, and without `--dry-run` with `status` `contained` when no delivery is granted

#### Scenario: Sign in, publish, repeat, sign out
- **WHEN** `syncloom linkedin sign-in` runs with the client id and secret set against the fake LinkedIn
- **THEN** stdout's first line is `{"status":"awaiting_consent","authorization_url":<url>,"expires_at":<instant>}` and, once the browser-like client has followed the URL, its last line is the `signed_in` connection
- **AND** `syncloom linkedin status` prints the `signed_in` connection
- **AND** with `SYNCLOOM_DELIVERY=linkedin`, `publish <piece_id> --expected-revision <revision>` on an approved piece prints `status` `posted` with `post_id` and `post_url`, and a repeat prints the same `post_id` with `created` false while the fake received one post
- **AND** `syncloom linkedin sign-out` prints the `signed_out` connection and the Keychain item is gone

### Requirement: Same errors as the API
A failed command SHALL print `{"error":{"code","message"}}` on stderr with the code and message the API would give, and exit 1.

#### Scenario: Core errors
- **WHEN** `thought` runs with 7 non-space characters, `show` with an unknown id, `approve` with a stale revision, `publish` on an unapproved piece, `linkedin sign-in` without a client id, or `linkedin sign-in` when the browser denies consent
- **THEN** stderr carries `thought_too_short`, `piece_not_found`, `revision_conflict`, `approval_required`, `linkedin_not_configured` and `sign_in_denied` with the fixture messages, and the exit status is 1
