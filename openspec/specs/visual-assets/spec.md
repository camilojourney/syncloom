# visual-assets Specification

## Purpose
Own local visual rendering and byte integrity for images and document carousels, independently of platform delivery.

## Requirements

### Requirement: Render visual assets
Rendering SHALL produce one 1080x1350 PNG and one PDF containing one to ten equally sized portrait pages from the piece's visual content.

#### Scenario: Two slides
- **WHEN** a render specifies two slides
- **THEN** the image shows the first slide and the PDF has two pages, each 810x1012.5 points
- **AND** empty slides, more than ten slides, blank text, unsupported templates and overflowing text are refused without usable assets

### Requirement: Pinned local renderer
Rendering SHALL require the configured local Chromium version and font checksum without downloading a browser or fetching external content.

#### Scenario: Reproducibility
- **WHEN** the pinned browser renders identical input twice on the same machine
- **THEN** the PNG bytes are identical
- **AND** PDF metadata is normalized before hashing
- **AND** snapshot comparison permits at most 0.1 percent of pixels differing by more than 8 channel levels

### Requirement: Render failures
Rendering SHALL report missing binary, version mismatch, font mismatch, timeout and crash distinctly without claiming success or approving assets.

#### Scenario: Browser failure
- **WHEN** the binary is absent, hangs or exits unsuccessfully
- **THEN** the result is respectively `renderer_missing`, `renderer_timeout` or `renderer_failed`
- **AND** no partial render becomes viewable or approved

### Requirement: Exact byte viewing
Viewing SHALL return only the requested current-revision PNG or PDF whose bytes match its recorded SHA-256, otherwise reporting `assets_required` or `asset_integrity`.

#### Scenario: Damaged file
- **WHEN** persisted bytes change after rendering
- **THEN** viewing and approval refuse them with `asset_integrity`

### Requirement: Meta JPEG slides
Rendering a Meta visual SHALL also save ordered 1080x1350 JPEG slides whose exact hashes are required for approval and rechecked before delivery.

#### Scenario: Revision-bound slides
- **WHEN** a Meta piece renders two slides
- **THEN** two JPEGs accompany the existing PNG and PDF and can be viewed through protected asset routes
- **AND** approval with missing, reordered or different JPEG hashes fails
- **AND** tampering prevents approval and dispatch
- **AND** legacy PNG/PDF records still load, but Instagram delivery requires a new render
