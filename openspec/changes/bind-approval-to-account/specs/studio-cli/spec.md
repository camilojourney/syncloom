## ADDED Requirements

### Requirement: Explicit account approval option
The CLI SHALL expose current_target through show and accept --expected-target on approve, sharing draft-review target checks and API error codes.

#### Scenario: Connected account
- **WHEN** a live-delivery piece is shown and approved with its revision and displayed current_target
- **THEN** approve returns the piece bound to that target
- **AND** a missing or stale expected target returns account_approval_required on stderr with exit 1

#### Scenario: Contained review
- **WHEN** no delivery target resolves
- **THEN** approval without --expected-target remains unbound for local operations
- **AND** it never authorizes a future resolved account without fresh review
