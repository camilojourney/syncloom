## MODIFIED Requirements

### Requirement: Explicit renewable-session refresh
Renewable integrations SHALL refresh only explicitly, durably replace same-account credentials, preserve credentials on pre-issuance exchange failure, retain actual newly issued bundles, and sign out on observed canonical-account mismatch.

#### Scenario: No implicit renewal
- **WHEN** a dispatch finds an expired access token
- **THEN** the existing absent-or-expired policy applies without a network request
- **AND** the founder can explicitly refresh X or Bluesky credentials, or unexpired eligible Threads credentials

#### Scenario: Identity mismatch
- **WHEN** an explicit refresh resolves a canonical account different from the stored account
- **THEN** refresh returns sign_in_rejected and the connection is signed_out
- **AND** approval history is not rebound or erased

## ADDED Requirements

### Requirement: Canonical posting identity
Each implemented remote adapter SHALL resolve a canonical provider account and exact posting target before authorizing delivery.

#### Scenario: Implemented providers
- **WHEN** sign-in completes
- **THEN** Bluesky uses its session DID, LinkedIn its member subject, X its authenticated user ID, and Threads its authenticated user ID
- **AND** LinkedIn and Threads retain the app namespace captured with their authenticated subject
- **AND** Mastodon scopes its authenticated account ID by instance authority
- **AND** Facebook uses the confirmed Page ID and Instagram uses the confirmed Page and linked Instagram account ID
- **AND** handles, token bytes, requested scopes, client IDs, slot keys and base URLs alone do not identify an account

#### Scenario: Unresolved identity
- **WHEN** canonical identity is missing, invalid or unavailable
- **THEN** no remote delivery is authorized and reauthentication is required to establish identity
- **AND** historical approvals are not silently rebound
- **AND** after successful X or Threads issuance, the actual new credential and grant evidence are retained atomically with the old canonical identity only expected and identity confirmation false for that token
- **AND** readiness reports cannot_post with identity_unconfirmed and sign-in-again recovery; a read-only confirmation retry on the same loaded token may precede a later publish, but no media or post is sent until it matches

#### Scenario: Renewal
- **WHEN** token renewal preserves the canonical account and exact target
- **THEN** identity remains stable independently of changed token bytes
- **AND** renewal cannot silently retarget a stored account approval
