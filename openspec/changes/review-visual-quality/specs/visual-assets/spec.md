## ADDED Requirements

### Requirement: Local technical assessment
Visual assessment SHALL report actual current asset identity and named byte-level technical observations separately from unassessed semantic and aesthetic quality.

#### Scenario: Ordinary rendered thesis
- **WHEN** the rendered slide repeats a valid source thesis
- **THEN** local assessment reports PNG decoding and dimensions, PDF page count and geometry, and exact hash association without penalizing source-word overlap
- **AND** quality_score is null and semantic_quality is unassessed, even when every technical check is satisfied
- **AND** the result names its assessor version, revision, full asset hashes and limitations

#### Scenario: Failure and negative control
- **WHEN** bytes are missing, tampered, malformed or have incorrect geometry
- **THEN** assessment reports unavailable or failed evidence, never an overall passing score
- **AND** a blank but structurally valid image cannot establish semantic or aesthetic quality
- **AND** assessment never changes approval, delivery permission or publishing

### Requirement: Assessment identity guard
Assessment SHALL refuse stale expected revisions or asset identities using existing revision_conflict, assets_required, asset_conflict and asset_integrity errors.

#### Scenario: Repeat, restart and concurrent change
- **WHEN** unchanged assets are assessed repeatedly or after restart
- **THEN** the same observations and identity are returned from the durable assets without a separate assessment store
- **AND** a concurrent edit or rerender cannot associate observations with different bytes

### Requirement: Version-bound asset requests
Asset viewing with an expected SHA-256 SHALL return only matching current bytes or asset_conflict without substituting a newer render.

#### Scenario: Old preview page
- **WHEN** a preview requests a previous render's image, PDF or JPEG hash after replacement
- **THEN** the request fails rather than displaying replacement bytes under old evidence
- **AND** existing requests without an expected hash retain their current-asset behavior
