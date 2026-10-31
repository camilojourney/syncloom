# Provenance delta

## ADDED Requirements

### Requirement: Visual evidence
The provenance log SHALL record visual configuration, render outcomes, byte viewing and approval with revisions and asset hashes, never slide text or local paths.

#### Scenario: Render, view, approve
- **WHEN** a visual piece is rendered, viewed and approved
- **THEN** events include visual_configured, assets_rendered, assets_viewed and piece_reviewed
- **AND** successful render, view and approval events carry the exact image and carousel hashes
- **AND** a renderer failure records render_failed with its fixed outcome code
- **AND** starting a same-content rerender records assets_invalidated when it clears the earlier render and approval
