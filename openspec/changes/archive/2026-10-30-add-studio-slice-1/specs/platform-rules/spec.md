## Purpose

The one owner of channel identities, the platform each channel posts to, and the text bounds a draft is checked against. Every other capability names a channel through this table and never carries its own limit.

## ADDED Requirements

### Requirement: Channel table
The rules SHALL recognize exactly the channels of the table in their scenario and map each to its platform and text bounds.

#### Scenario: The table
- **WHEN** a channel name is looked up
- **THEN** `linkedin_text` maps to platform `linkedin` with bounds 20 to 3000 characters
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
