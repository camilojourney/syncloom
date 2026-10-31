# platform-rules Specification

## Purpose
The one owner of channel identities, the platform each channel posts to, and the text bounds a draft is checked against. Every other capability names a channel through this table and never carries its own limit.

## Requirements

### Requirement: Channel table
The rules SHALL recognize exactly the channels of the table in their scenario and map each to its platform and text bounds.

#### Scenario: The table
- **WHEN** a channel name is looked up
- **THEN** `linkedin_text` maps to platform `linkedin` with bounds 20 to 3000 characters
- **AND** `mastodon` maps to platform `mastodon` with draft bounds 1 to 500; live instance limits are checked by `mastodon-integration`
- **AND** `manual_export` maps to platform `manual` with local drafting bounds 1 to 40000; target notes, not this bound, govern manual platform suitability
- **AND** any other name, including `linkedin`, `LinkedIn_Text` and `linkedin_text ` with a trailing space, is unknown

### Requirement: Platform fit
The rules SHALL mark a draft `PASS` when its character count is within the channel's bounds, inclusive, and `REVIEW` otherwise.

#### Scenario: Upper bound
- **WHEN** a `linkedin_text` draft has 3000 characters
- **THEN** `fit` is `{"verdict":"PASS","char_count":3000,"min_chars":20,"max_chars":3000}`
- **AND** a draft of 3001 characters has `verdict` `REVIEW` and `char_count` 3001

#### Scenario: Lower bound
- **WHEN** a `linkedin_text` draft has 20 characters
- **THEN** `verdict` is `PASS`
- **AND** a draft of 19 characters has `verdict` `REVIEW`

#### Scenario: Characters are Unicode scalar values
- **WHEN** a draft is 3000 characters of which some are multi-byte, such as `é` or `日`
- **THEN** `char_count` is 3000 and `verdict` is `PASS`

### Requirement: Fit follows the text
The piece's `fit` SHALL be recomputed whenever its text changes.

#### Scenario: Edit crosses the bound
- **WHEN** a piece with `PASS` is edited to 3001 characters
- **THEN** its `fit.verdict` is `REVIEW` and `fit.char_count` is 3001

### Requirement: Meta channels
The channel table SHALL recognize facebook_page for facebook and instagram for instagram with their text bounds.

#### Scenario: Meta bounds
- **WHEN** the Meta channels are looked up
- **THEN** facebook_page permits 1 to 63206 characters and instagram permits 1 to 2200 characters
- **AND** fit follows platform-rules without truncation

### Requirement: Q1 channels
The channel table SHALL additionally recognize `bluesky`, `x` and `threads`, each mapping to its namesake platform with minimum one text unit.

#### Scenario: Conservative limits
- **WHEN** a Q1 draft is checked
- **THEN** Bluesky allows 300 graphemes and 3000 bytes; X allows 280 conservative weighted units; Threads allows 500 conservative UTF-8 bytes
- **AND** no text is truncated and an over-limit draft receives REVIEW
