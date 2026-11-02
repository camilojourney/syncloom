## ADDED Requirements

### Requirement: Displayed approval instant
The review page SHALL show the instant recorded for that exact approval event.

#### Scenario: Fresh account approval
- **WHEN** a revision approved at one instant is approved again for another account at a later controlled instant
- **THEN** fresh review shows the later instant and the earlier review keeps the earlier instant
- **AND** both instants match their piece_reviewed events, and the fresh page has no second Approve control
