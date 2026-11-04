## ADDED Requirements

### Requirement: Visual assessment command
The CLI SHALL expose assess-visual with piece id, expected revision and asset hashes, returning visual-assets assessment JSON or the existing fixed core errors.

#### Scenario: Same evidence
- **WHEN** assess-visual receives the same piece identity as the loopback operation
- **THEN** it returns the same assessment and exits zero without provider or platform access
- **AND** stale or missing assets exit one with the same error code and message as the API
