## ADDED Requirements

### Requirement: Explicit URL retrieval
`POST /v1/capture/url` with exactly `url` SHALL return a source receipt and capture suggestions without drafting, or a fixed retrieval error.

#### Scenario: Retrieved source
- **WHEN** the founder explicitly fetches an allowed public HTML or plain-text URL
- **THEN** status is 200 with `source_revision`, `source` and `suggestions`
- **AND** `source.type` is `url`, `raw_input` is the requested URL, and `retrieval` contains `final_url`, UTC `fetched_at`, response `content_sha256` and `extracted_text`
- **AND** no model call, approval, grant, schedule, piece or publish intent exists
- **AND** ordinary thought and suggestion submissions containing links still never fetch them

### Requirement: URL destination policy
Retrieval SHALL accept only configured exact hosts resolving exclusively to public addresses, including every redirect destination.

#### Scenario: Empty and exact allowlist
- **WHEN** `SYNCLOOM_SOURCE_HOSTS` is unset or blank
- **THEN** retrieval returns 503 `source_not_configured`
- **AND** comma-separated DNS hosts are case-normalized, whitespace-trimmed and deduplicated at startup
- **AND** wildcards, URLs, ports, IP literals, empty entries, trailing dots and malformed labels fail configuration
- **AND** configuring `source.example` does not allow `sub.source.example`

#### Scenario: Forbidden destinations
- **WHEN** a source uses credentials, query, fragment, non-HTTP scheme, nonstandard port, IP literal, trailing-dot host or unlisted host
- **THEN** status is 422 `source_not_allowed` before an HTTP request
- **AND** private, loopback, link-local, metadata, multicast, unspecified, reserved and mixed public/private DNS results are refused
- **AND** the connection uses only the validated addresses, not a second DNS lookup or environment proxy

#### Scenario: Redirects
- **WHEN** a source redirects
- **THEN** each hop obeys the same destination policy and HTTPS never downgrades to HTTP
- **AND** at most three redirects are followed
- **AND** missing or malformed Location and excess redirects return 502 `source_fetch_failed`

### Requirement: Bounded uncredentialed retrieval
Retrieval SHALL fetch one bounded text response without credentials, cookies, subresources, retries or executing remote content.

#### Scenario: Supported content
- **WHEN** a successful response is UTF-8 HTML or plain text within 262144 bytes and 8000 extracted characters
- **THEN** text is normalized, HTML markup and script/style/head/template content are excluded, and entities are decoded
- **AND** fewer than eight non-space characters returns `thought_too_short`
- **AND** no content is silently truncated

#### Scenario: Failed response
- **WHEN** the source times out after the total ten-second retrieval budget
- **THEN** status is 504 `source_timeout`
- **AND** HTTP failures, oversized bodies/text, unsupported media/encoding, malformed UTF-8 and compressed responses return 502 `source_fetch_failed`
- **AND** no raw network error, URL or source content appears in errors or logs

### Requirement: URL confirmation provenance
URL confirmation SHALL draft only the exact retrieved snapshot and selected destinations using the existing capture claim and thought-drafting behavior.

#### Scenario: Receipt-bound selection
- **WHEN** preview supplies `source_revision`, matching extracted `text` and suggested `channels`
- **THEN** the preview identifies that receipt and selection, without refetching
- **AND** confirm with its `expected_revision` creates pending pieces whose source equals the retrieved source
- **AND** a duplicate returns the original group without another fetch or provider call
- **AND** changed text, unknown receipt or stale selection returns 409 `revision_conflict`
- **AND** capture result remains usable after restart

### Requirement: Untrusted source consent
The URL preview SHALL show retrieved content as untrusted source data and require explicit confirmation before sending it to the configured drafting provider.

#### Scenario: Founder decides separately
- **WHEN** retrieval succeeds
- **THEN** the browser displays requested URL, final URL, retrieval time, response hash and extracted text
- **AND** selection and preview reuse capture pages, without another editor
- **AND** preview explains provider transfer and that confirmation is not approval or publication
- **AND** remote markup is inert and remote instructions receive no tool authority

### Requirement: URL audit privacy
URL intake SHALL preserve `provenance-log` content-free events without recording source URLs or retrieved text in audit logs.

#### Scenario: Source remains local
- **WHEN** a controlled URL is fetched, previewed and confirmed through a fake provider
- **THEN** pieces retain the source and provenance events contain only existing ids, hashes, statuses and provider usage
- **AND** no URL, source content, header credential or personal contact detail appears in request headers or logs
