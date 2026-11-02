## MODIFIED Requirements

### Requirement: Explicit renewable-session refresh
Renewable integrations SHALL refresh only explicitly, durably replacing same-account credentials, preserving ordinary failures, and signing out after observed canonical-account mismatch.

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
- **AND** existing credentials and historical approvals are not silently rebound

#### Scenario: Renewal
- **WHEN** token renewal preserves the canonical account and exact target
- **THEN** identity remains stable independently of changed token bytes
- **AND** renewal cannot silently retarget a stored account approval
