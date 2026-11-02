## ADDED Requirements

### Requirement: Plain posting readiness
The review panel SHALL name the platform and stable readiness reason in one plain sentence.

#### Scenario: Missing posting permission
- **WHEN** review shows reason posting_permission_missing for X
- **THEN** the panel says Your X account is connected, but Syncloom isn't allowed to post to it. Reconnect and allow posting.
- **AND** the connection API keeps reason posting_permission_missing, message connected, cannot post: posting permission missing., and action sign_in
- **AND** Publish is natively disabled and Reconnect account remains beside that sentence

#### Scenario: Unverified posting permission
- **WHEN** review shows reason posting_permission_unverified for Threads
- **THEN** the panel says Syncloom couldn't confirm it's allowed to post to your Threads account. Reconnect to check again.
- **AND** the API reason and error code remain posting_permission_unverified

#### Scenario: Unconfirmed identity
- **WHEN** review shows reason identity_unconfirmed for X or Threads
- **THEN** the panel says Syncloom couldn't confirm this is your X account. Reconnect to check again. or the Threads equivalent
- **AND** the API message remains connected, cannot post: account identity could not be confirmed. Sign in again.
- **AND** the panel does not show the raw connected, cannot post prefix

### Requirement: Inactive disabled Publish
A disabled Publish control SHALL look inactive in both themes while its label stays readable.

#### Scenario: Readiness refusal
- **WHEN** Publish is disabled beside a readiness sentence in light or dark theme
- **THEN** its fill is not the accent action fill or the subtle fill, and the label meets normal text contrast
- **AND** the sentence and Reconnect account control stay visible
- **AND** an enabled Publish control keeps the existing accent fill
