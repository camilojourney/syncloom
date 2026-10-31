# bluesky-integration Specification

## Purpose
Own the founder's Bluesky app-password session lifecycle and approved text or PNG delivery through the official AT Protocol API.

## Requirements

### Requirement: App-password sign-in
`POST /v1/bluesky/sign-in` SHALL accept `identifier` and `app_password`, returning a connection or fixed sign-in errors through `platform-credentials`.

#### Scenario: Create and retain session
- **WHEN** generic test credentials sign in
- **THEN** exactly one createSession request is sent, and the app password and returned session are stored only through the credential store
- **AND** missing, empty, rejected or malformed credentials never produce a signed-in answer

### Requirement: Session renewal
`POST /v1/bluesky/refresh` SHALL renew the stored session, returning a connection or fixed sign-in errors without creating a new password session.

#### Scenario: Rotate
- **WHEN** refreshSession returns new access and refresh JWTs
- **THEN** both replace the old session before subsequent publishing
- **AND** refresh failure preserves the old stored session and sends no post

### Requirement: Bluesky connection operations
`GET /v1/bluesky/connection` and `DELETE /v1/bluesky/connection` SHALL return the stored connection, the latter after deleting credentials.

#### Scenario: Read without network
- **WHEN** connection is read and deleted twice
- **THEN** no API request is made and both deletions answer signed_out

### Requirement: Bluesky publishing
A Bluesky attempt SHALL deliver approved text and optional PNG with alt text, using `dispatch-outbox` for grant, approval, deadline, repeat safety and truthful outcomes.

#### Scenario: Image before record
- **WHEN** the approved piece includes a verified rendered image
- **THEN** uploadBlob precedes exactly one createRecord containing its blob and alt text
- **AND** only a successful record response with the expected repository URI and valid CID means posted

#### Scenario: Failure and limits
- **WHEN** text exceeds 300 graphemes or 3000 bytes, or an image lacks alt text or exceeds the supported size
- **THEN** no record is created
- **AND** HTTP 401, 429, other 4xx, 5xx, malformed success and timeout are never reported as posted
