# studio-cli Specification

## Purpose
Own the command line `syncloom`: the same core functions as the HTTP API, with the same error codes and messages, for the operator and for agents working in a shell.

## Requirements

### Requirement: Commands
The CLI SHALL offer `thought`, `show`, `approve` and `publish`, each printing the core's JSON result on stdout and exiting 0.

#### Scenario: The four commands
- **WHEN** `syncloom thought --text <thought> --channel linkedin_text` runs with a reachable provider
- **THEN** stdout is the same `{"group_id","pieces"}` JSON the API returns, and `show <piece_id>` prints that piece
- **AND** `approve <piece_id> --expected-revision <revision>` prints the approved piece
- **AND** `publish <piece_id> --expected-revision <revision> --dry-run` prints the publish result with `status` `dry_run`, and without `--dry-run` with `status` `contained`

### Requirement: Same errors as the API
A failed command SHALL print `{"error":{"code","message"}}` on stderr with the code and message the API would give, and exit 1.

#### Scenario: Core errors
- **WHEN** `thought` runs with 7 non-space characters, `show` with an unknown id, `approve` with a stale revision, or `publish` on an unapproved piece
- **THEN** stderr carries `thought_too_short`, `piece_not_found`, `revision_conflict` and `approval_required` with the fixture messages, and the exit status is 1

### Requirement: Data directory
The CLI SHALL store records under `SYNCLOOM_DATA_DIR`, defaulting to `$HOME/.syncloom`, the same layout the server reads.

#### Scenario: Shared store
- **WHEN** a piece is created with the CLI and the server starts on the same `SYNCLOOM_DATA_DIR`
- **THEN** `GET /v1/pieces/{piece_id}` returns it
