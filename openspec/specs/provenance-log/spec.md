# provenance-log Specification

## Purpose
Own the append-only provenance log: one event per state change, holding ids, hashes and statuses, never the thought or the draft text.

## Requirements

### Requirement: One event per state change
The studio SHALL append one event for each piece creation, review decision, edit, intent reservation, dispatch attempt and dispatch outcome, in the order they happened.

#### Scenario: Four changes, four events
- **WHEN** a thought is drafted, the piece approved, edited, approved again and published without a grant
- **THEN** `provenance/events.jsonl` holds, in order, `piece_created`, `piece_reviewed`, `piece_edited`, `piece_reviewed`, `intent_reserved`
- **AND** a dry run, a repeated publish and a refused request add no event

#### Scenario: Dispatch events
- **WHEN** an approved piece is published with delivery granted and the platform answers posted
- **THEN** the log gained, in order, `dispatch_attempted` then `dispatch_finished`
- **AND** a repeated publish of the `posted` piece adds no event

### Requirement: Event shape
Each event SHALL be one JSON line with `schema_version` `1.0`, `event_id`, `event`, `at`, `group_id`, `piece_id`, `channel`, `revision` and `status`.

#### Scenario: Fields
- **WHEN** an event is read back
- **THEN** `event_id` is a UUID, `at` is an RFC 3339 UTC instant ending in `Z`, `revision` equals the piece's revision at that moment and `status` its status
- **AND** `piece_created` also carries `usage` with `input_tokens` and `output_tokens`, and `intent_reserved` also carries `request_id`
- **AND** `dispatch_attempted` carries `request_id`, and `dispatch_finished` carries `request_id`, `outcome` and, when the outcome is `posted`, `post_id`

### Requirement: No content in events
No event SHALL contain the thought, the draft text, a provider key or a platform token.

#### Scenario: Canary text
- **WHEN** the thought and the provider's answer both contain a canary word
- **THEN** the canary word appears nowhere in `provenance/events.jsonl`

#### Scenario: Canary token
- **WHEN** the fake platform issued a token containing a canary word and a piece was dispatched with it
- **THEN** the canary word appears nowhere in `provenance/events.jsonl`

### Requirement: Append-only across restarts
Existing event lines SHALL never be rewritten; reopening the store and appending leaves earlier lines byte-identical.

#### Scenario: Reopen and append
- **WHEN** the log holds two events, the store is reopened and a third event is appended
- **THEN** the first two lines are unchanged and the file has three lines

### Requirement: Event failure is reported
A failed event append SHALL answer 500 `internal_error`; the piece write that preceded it stays.

#### Scenario: Log directory not writable
- **WHEN** the provenance directory refuses writes and a thought is drafted
- **THEN** status is 500 with code `internal_error`
- **AND** the piece exists when shown, so the operator can see what was drafted
