# Platform rules delta

## MODIFIED Requirements

### Requirement: Channel table
The rules SHALL recognize exactly the channels of the table in their scenario and map each to its platform and text bounds.

#### Scenario: The table
- **WHEN** a channel name is looked up
- **THEN** `linkedin_text` maps to platform `linkedin` with bounds 20 to 3000 characters
- **AND** `mastodon` maps to platform `mastodon` with draft bounds 1 to 500; live instance limits are checked by `mastodon-integration`
- **AND** `manual_export` maps to platform `manual` with local drafting bounds 1 to 40000; target notes, not this bound, govern manual platform suitability
- **AND** any other name, including `linkedin`, `LinkedIn_Text` and `linkedin_text ` with a trailing space, is unknown

