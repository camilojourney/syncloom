## ADDED Requirements

### Requirement: Optional X renewal credential
X sign-in SHALL accept an otherwise valid text posting grant without an offline grant or refresh token.

#### Scenario: Fresh text without renewal
- **WHEN** the token response grants `tweet.write users.read` with no refresh token
- **THEN** the connection can publish approved text
- **AND** media delivery still requires its separate effective media permission
- **AND** explicit refresh refuses before a request when no renewal credential exists
