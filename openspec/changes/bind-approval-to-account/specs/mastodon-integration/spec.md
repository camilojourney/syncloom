## MODIFIED Requirements

### Requirement: Per-instance OAuth sign-in
Mastodon sign-in SHALL register an app on the configured instance and exchange a matching loopback OAuth callback for a resolved instance-scoped token.

#### Scenario: Consent
- **WHEN** sign-in starts on the explicitly configured instance
- **THEN** one app registration uses a generic client name, loopback redirect and read:accounts write:statuses write:media scopes
- **AND** the returned authorization URL leads to that instance
- **AND** completion resolves the issued token's canonical account with verify_credentials before storing it through platform-credentials
- **AND** wrong state, denied consent, token or identity rejection and deadline return fixed sign-in errors without storing new credentials
- **AND** the callback requires its exact Host, echoes no input and sends no CORS headers
