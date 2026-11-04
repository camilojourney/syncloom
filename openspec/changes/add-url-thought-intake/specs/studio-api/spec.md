## ADDED Requirements

### Requirement: URL capture operations
The loopback API SHALL expose URL retrieval and receipt-bound capture selection under the existing session, Host, Origin, logging and response protections.

#### Scenario: API and browser operations
- **WHEN** the local founder uses URL capture
- **THEN** `POST /v1/capture/url` accepts exactly `url` and returns the `thought-drafting` retrieval result or fixed errors
- **AND** `POST /capture/url` fetches the single URL entered in the existing workspace composer and shows capture suggestions
- **AND** existing `/v1/capture/preview` and `/v1/capture/confirm` accept optional `source_revision` alongside their existing fields
- **AND** their browser counterparts carry that receipt without refetching or another editor
- **AND** malformed fields return 400 `invalid_request`; stale receipts return 409 `revision_conflict`

### Requirement: URL recovery guidance
The browser SHALL explain URL retrieval configuration and failure recovery without implying that a refused source was ingested.

#### Scenario: Not configured or unavailable
- **WHEN** URL retrieval is unconfigured or fails
- **THEN** the browser shows its fixed error and a visible Workspace return
- **AND** configuration errors name `SYNCLOOM_SOURCE_HOSTS` and restarting with an authorized exact host
- **AND** failed retrieval suggests another permitted text page or pasting user-owned text instead
- **AND** the workspace explains that Fetch URL only retrieves, while drafting and publication require separate decisions

### Requirement: URL responsive keyboard journey
URL capture SHALL preserve visible Workspace navigation and keyboard focus without horizontal clipping at 320, 390 and 1440 CSS pixels in both themes.

#### Scenario: Complete controlled journey
- **WHEN** a founder fetches a controlled URL, selects a destination, previews and confirms using the keyboard
- **THEN** focus remains visible, Workspace is reachable and returns to the composer
- **AND** long URLs, complete extracted text and full hashes wrap without clipping or horizontal page overflow
