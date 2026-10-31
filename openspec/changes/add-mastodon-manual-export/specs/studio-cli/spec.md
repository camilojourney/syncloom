# CLI delta

## MODIFIED Requirements

### Requirement: Commands
The CLI SHALL offer `thought`, `show`, `approve`, `publish`, `linkedin sign-in|status|sign-out`, `mastodon sign-in|status|sign-out` and `export`, each printing the core's JSON result on stdout and exiting 0.

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

#### Scenario: Mastodon and export commands
- **WHEN** `syncloom mastodon sign-in` runs against the local fake
- **THEN** it prints the authorization URL, waits for the callback and prints the safe connection
- **AND** `mastodon status` and `mastodon sign-out` report the connection without exposing credentials
- **AND** `export <piece_id> --expected-revision <revision> --target reddit` returns the local bundle receipt

