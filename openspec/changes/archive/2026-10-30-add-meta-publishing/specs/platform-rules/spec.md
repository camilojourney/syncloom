## ADDED Requirements

### Requirement: Meta channels
The channel table SHALL recognize facebook_page for facebook and instagram for instagram with their text bounds.

#### Scenario: Meta bounds
- **WHEN** the Meta channels are looked up
- **THEN** facebook_page permits 1 to 63206 characters and instagram permits 1 to 2200 characters
- **AND** fit follows platform-rules without truncation
