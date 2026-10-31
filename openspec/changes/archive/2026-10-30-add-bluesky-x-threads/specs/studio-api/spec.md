# Studio API delta

## ADDED Requirements

### Requirement: Q1 connection surface
The API SHALL expose the three Q1 integration operations through the existing boundary and fixed error envelope.

#### Scenario: Operations and errors
- **WHEN** a client calls the Bluesky, X or Threads operations defined by their integration capabilities
- **THEN** existing Host, Origin and mutation-session checks apply without CORS
- **AND** missing app settings return 503 platform_not_configured; unconfirmed X delivery returns 409 cost_confirmation_required
- **AND** error responses match their fixed contract fixtures and never echo secret input
