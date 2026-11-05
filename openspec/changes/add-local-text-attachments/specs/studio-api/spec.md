## ADDED Requirements

### Requirement: Guarded attachment upload
The loopback API SHALL expose explicit multipart attachment upload under existing Host, Origin, Fetch-Site, server-session, error, logging and response protections.

#### Scenario: API and browser operations
- **WHEN** `POST /v1/capture/attachment` supplies exactly one multipart field named `file` with the API session header
- **THEN** it returns the thought-drafting attachment result or fixed errors
- **AND** `POST /capture/attachment` accepts exactly one `file` and one valid `csrf` field, returning existing capture suggestions
- **AND** duplicate file, missing file, unknown fields and malformed multipart return 400 `invalid_request`
- **AND** missing, repeated, stale or another server's CSRF/session token returns 403 `session_required`
- **AND** no supplied filename is interpreted as a server path

#### Scenario: Resource controls
- **WHEN** upload is received
- **THEN** the total request body remains bounded to 1048576 bytes and the complete read/parse has a ten-second deadline
- **AND** total-body overflow or malformed framing returns `invalid_request`; deadline expiry returns 408 `attachment_timeout`
- **AND** at most two uploads are processed concurrently without waiting in an unbounded queue; an excess request returns 503 `attachment_busy`
- **AND** errors contain no uploaded content, filename, MIME, path or parser diagnostics
- **AND** upload routes log fixed labels `/capture/attachment` or `/v1/capture/attachment`

### Requirement: Native attachment journey
The workspace SHALL offer explicit local file selection, extraction, destination selection, preview and confirmation through native navigation without changing text or URL semantics.

#### Scenario: Useful selected content
- **WHEN** the founder selects a task-created text or Markdown file and requests extraction
- **THEN** the page shows its filename, validated format, original byte count/hash, capture time and complete extracted text
- **AND** the selection and preview reuse existing capture pages, never a second editor
- **AND** preview separately displays the actual normalized provider input under thought-drafting's existing whitespace rules
- **AND** consent explains that only that normalized text, not raw bytes or filename, reaches the configured drafting provider
- **AND** explicit confirm creates pending drafts only, with no approval or publication control

#### Scenario: Honest unsupported and empty states
- **WHEN** the workspace has no selected file
- **THEN** it names `.txt`/`.md`, UTF-8, byte/text limits and inert Markdown, without advertising PDF/image support
- **AND** unsupported, invalid, too-short, over-limit, busy, timed-out and stale outcomes visibly explain that no new draft was created and offer Workspace recovery
- **AND** empty destination selection preserves the source and asks for at least one destination

### Requirement: Attachment responsive keyboard journey
Attachment capture SHALL preserve visible navigation, complete content and keyboard focus at 320, 390 and 1440 CSS pixels in light and dark themes.

#### Scenario: Native selection through review
- **WHEN** the founder uses file selection, Tab, Enter, destination selection, preview, confirmation and Workspace navigation
- **THEN** controls have accessible names and visible focus, with no clipped words/buttons, hidden navigation or horizontal page overflow
- **AND** application-rendered extracted provenance/preview wraps complete filenames, full hashes and complete text rather than eliding them
- **AND** the native file picker retains its complete accessible selected value and an honest hint explains that the complete filename appears after local extraction; browser-native selected-name elision before extraction is permitted, not clipped author labels/buttons or hidden application content
- **AND** native browser navigation provides loading feedback without executing file content or adding scripts
