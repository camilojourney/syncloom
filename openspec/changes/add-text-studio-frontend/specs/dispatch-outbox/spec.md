## ADDED Requirements

### Requirement: Current publish inspection
Core SHALL provide read-only inspection by piece id, returning the saved piece, its current-revision publish record and the actual recorded approval time when available, or existing not-found/storage errors.

#### Scenario: Per-operation isolation
- **WHEN** a piece is inspected after a separate export operation
- **THEN** inspection returns its publish intent independently of the last-operation summary
- **AND** no dispatch, credential lookup, event append or persisted status conversion occurs

#### Scenario: Revision and missing record
- **WHEN** a piece has been edited or has no current-revision publish intent
- **THEN** inspection returns the current piece without an old or fabricated publish record
- **AND** an invalid/missing piece returns piece_not_found

#### Scenario: Approval time provenance
- **WHEN** an approved revision is inspected after later dispatch or export activity
- **THEN** its approval time is the last matching approved review event, not its latest modification time
- **AND** missing history yields no invented date and corrupt history returns the existing storage error
