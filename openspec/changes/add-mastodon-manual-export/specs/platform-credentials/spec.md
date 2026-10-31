# Platform credentials delta

## ADDED Requirements

### Requirement: Instance-scoped non-expiring token
Mastodon credentials SHALL use an account keyed by the instance origin and remain signed in until deleted or refused, without fabricating an expiry.

#### Scenario: Independent instances
- **WHEN** one instance is signed in and another instance is selected
- **THEN** the second instance has no token and publishing sends nothing
- **AND** the first connection reports `signed_in` without `expires_at`; sign-out deletes only its token
- **AND** app secrets live in memory during sign-in only, never in the data directory
