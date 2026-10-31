# Platform rules delta

## ADDED Requirements

### Requirement: Q1 channels
The channel table SHALL additionally recognize `bluesky`, `x` and `threads`, each mapping to its namesake platform with minimum one text unit.

#### Scenario: Conservative limits
- **WHEN** a Q1 draft is checked
- **THEN** Bluesky allows 300 graphemes and 3000 bytes; X allows 280 conservative weighted units; Threads allows 500 conservative UTF-8 bytes
- **AND** no text is truncated and an over-limit draft receives REVIEW
