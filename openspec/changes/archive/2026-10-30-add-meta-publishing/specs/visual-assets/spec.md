## ADDED Requirements

### Requirement: Meta JPEG slides
Rendering a Meta visual SHALL also save ordered 1080x1350 JPEG slides whose exact hashes are required for approval and rechecked before delivery.

#### Scenario: Revision-bound slides
- **WHEN** a Meta piece renders two slides
- **THEN** two JPEGs accompany the existing PNG and PDF and can be viewed through protected asset routes
- **AND** approval with missing, reordered or different JPEG hashes fails
- **AND** tampering prevents approval and dispatch
- **AND** legacy PNG/PDF records still load, but Instagram delivery requires a new render
