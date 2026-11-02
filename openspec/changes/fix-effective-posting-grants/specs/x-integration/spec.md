## ADDED Requirements

### Requirement: Effective X response scope provenance
X SHALL distinguish an absent scope field, explicit null and a present response value before computing readiness.

#### Scenario: Absent refresh scope
- **WHEN** an otherwise valid refresh response omits scope
- **THEN** the actual rotated credential retains the prior provider-returned scope, not requested defaults
- **AND** a fresh exchange with absent scope instead returns posting_permission_unverified without saving credentials

#### Scenario: Unverified present evidence
- **WHEN** an otherwise valid issued credential contains null, empty, noncanonical or wrong-typed scope evidence
- **THEN** it returns posting_permission_unverified and never carries forward broader permission
- **AND** a valid rotation retains its new credential with unresolved evidence atomically, reports cannot_post and sends no publishing write
- **AND** explicit valid reduced scope remains posting_permission_missing, not unknown

### Requirement: Optional X renewal credential
X sign-in SHALL accept an otherwise valid text posting grant without an offline grant or refresh token.

#### Scenario: Fresh text without renewal
- **WHEN** the token response grants `tweet.write users.read` with no refresh token
- **THEN** the connection can publish approved text
- **AND** media delivery still requires its separate effective media permission
- **AND** explicit refresh refuses before a request when no renewal credential exists
