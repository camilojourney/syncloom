# manual-export Specification

## Purpose
Own portable local handoff bundles and truthful export receipts, without implying that content was published to a platform.

## Requirements

### Requirement: Export operation
Exporting SHALL accept a piece id, expected revision and target, returning a local bundle location and an `exported` receipt, or the existing review and storage errors.

#### Scenario: Targets
- **WHEN** an approved piece is exported for `reddit` or `tiktok`
- **THEN** its bundle contains exact text, target-specific notes and, for a visual piece, verified PNG and PDF files plus alt text
- **AND** notes distinguish Reddit manual posting from TikTok's later video handoff; no API requests occur
- **AND** unknown targets return `invalid_request`

### Requirement: Export requires reviewed current bytes
Exporting SHALL enforce `draft-review` revision and approval checks and `visual-assets` integrity checks before writing a bundle.

#### Scenario: Refused exports
- **WHEN** export names a stale revision, an unapproved piece or damaged assets
- **THEN** it returns `revision_conflict`, `approval_required` or `asset_integrity` respectively without creating a bundle or receipt
- **AND** a traversal-shaped piece id returns `piece_not_found`

### Requirement: Durable truthful export receipt
A completed export SHALL record `exported`, never `posted`, with no post id or URL and repeat-safe identity per piece, revision and target.

#### Scenario: Repeat and reopen
- **WHEN** the same export is repeated after reopening the studio
- **THEN** the same bundle and request id are returned without another receipt event
- **AND** the piece reports `exported`, no posting attempt occurred, and damaged bundle files are refused rather than overwritten silently

#### Scenario: Partial failure
- **WHEN** bundle writing fails before completion
- **THEN** no successful export receipt is recorded and the export can be retried

### Requirement: Export files remain confined
Exporting SHALL write only inside the studio's exports area and refuse symlink destinations without changing their targets.

#### Scenario: Symlink
- **WHEN** the exports area or bundle destination is a symlink
- **THEN** export fails and outside files remain unchanged
