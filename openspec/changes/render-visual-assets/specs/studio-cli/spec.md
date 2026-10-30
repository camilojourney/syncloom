# CLI delta

## ADDED Requirements

### Requirement: Visual commands
The CLI SHALL expose render and view commands with the API's core results and errors.

#### Scenario: Local visual workflow
- **WHEN** render <piece_id> --expected-revision <revision> --template midnight --slide <text> is called, repeating --slide up to ten times
- **THEN** JSON reports the piece with asset hashes
- **AND** view <piece_id> --kind image or carousel writes the verified bytes to stdout
- **AND** approve uses the existing command at the rendered revision
