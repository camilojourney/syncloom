# Platform credentials delta

## ADDED Requirements

### Requirement: Explicit renewable-session refresh
Renewable integrations SHALL refresh only on explicit request, replacing credentials durably before reuse and preserving existing credentials when refresh fails.

#### Scenario: No implicit renewal
- **WHEN** a dispatch finds an expired access token
- **THEN** the existing absent-or-expired policy applies and no network request is made
- **AND** the founder can explicitly refresh X or Bluesky credentials, or unexpired eligible Threads credentials
