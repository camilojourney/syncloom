## ADDED Requirements

### Requirement: Discover saved pieces
`GET /v1/pieces` with optional `after` SHALL return a deterministic page of saved pieces and `next_after`, or 400 `invalid_request` for an invalid cursor.

#### Scenario: Multiple pages
- **WHEN** more than twenty pieces exist
- **THEN** the response is 200 with `{"pieces":<up to twenty pieces>,"next_after":<last id or null>}` ordered by piece id
- **AND** passing `next_after` as `after` returns the next page without duplicates
- **AND** no pieces returns an empty array and null cursor
- **AND** interrupted-write files are ignored, malformed selected records return `internal_error`, and symlinks are refused
