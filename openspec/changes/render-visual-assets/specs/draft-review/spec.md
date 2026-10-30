# Review delta

## MODIFIED Requirements

### Requirement: Revision
A piece's revision SHALL identify its channel, id, text and, when configured, visual content.

#### Scenario: Worked example
- **WHEN** `piece_id` is `thought-0123456789ab-linkedin_text`, `channel` is `linkedin_text` and `text` is `hello world`
- **THEN** `revision` is `b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92`
- **AND** a different text gives a different revision

#### Scenario: Text-only compatibility
- **WHEN** a piece has no visual configuration
- **THEN** its revision remains the lowercase hex SHA-256 of compact sorted-key JSON containing channel, piece_id and text
- **AND** the existing hello-world worked example remains unchanged

#### Scenario: Visual revision
- **WHEN** text, slide content or template changes
- **THEN** the revision changes, assets are invalidated and approval is cleared
- **AND** a render finishing after a concurrent edit is refused with revision_conflict

## ADDED Requirements

### Requirement: Visual approval guard
Visual approval SHALL require intact current-revision assets matching the caller's expected image and carousel SHA-256 values.

#### Scenario: Render before approval
- **WHEN** visual content changes or rendering fails
- **THEN** approval is refused with assets_required until a successful render
- **AND** approval supplies both rendered PNG and PDF SHA-256 values as expected_assets
- **AND** missing hashes return assets_required and mismatched hashes return asset_conflict, including after a same-revision rerender
- **AND** rerendering clears approval even when the input revision stays unchanged

#### Scenario: No visual upload
- **WHEN** an approved visual piece is published with LinkedIn delivery granted
- **THEN** it stays contained without a platform request
