## ADDED Requirements

### Requirement: Local text attachment intake
`POST /v1/capture/attachment` with one file SHALL return a source receipt and capture suggestions without drafting, or a fixed attachment error.

#### Scenario: Selected UTF-8 file
- **WHEN** the founder explicitly uploads a supported `.txt` or `.md` file
- **THEN** status is 200 with `source_revision`, `source` and `suggestions`
- **AND** source type is `attachment`, with filename, validated format, original byte count, original-byte SHA-256, UTC capture time and complete extracted text
- **AND** filename and claimed MIME never substitute for validating actual bytes
- **AND** no draft, approval, grant, schedule, publish intent, provider call or network retrieval occurs

### Requirement: Bounded attachment formats
Attachment intake SHALL accept only bounded UTF-8 plain-text and literal Markdown files, rejecting unsupported, malformed, unextractable and over-limit material without truncation.

#### Scenario: Supported formats and metadata
- **WHEN** a file has a case-insensitive `.txt` or `.md` extension and a basename of 1 through 128 UTF-8 bytes
- **THEN** absent MIME, `application/octet-stream` and `text/plain` are accepted as hints for either extension; `text/markdown` is additionally accepted for `.md`
- **AND** path separators, control characters and directional-formatting characters in filenames return 422 `attachment_invalid`
- **AND** unsupported extensions or other claimed MIME return 422 `attachment_unsupported`
- **AND** PDF, image, OCR, Office and archive intake remain unsupported, even when metadata is forged

#### Scenario: Exact byte and text bounds
- **WHEN** original file bytes are at most 262144 and extracted text has at most 8000 Unicode scalar values
- **THEN** those upper boundaries are accepted if all other checks pass
- **AND** exceeding either boundary returns 422 `attachment_too_large`, never a shortened success
- **AND** malformed UTF-8 or control characters other than tab and normalized newline return 422 `attachment_invalid`
- **AND** directional-formatting characters U+061C, U+200E..U+200F, U+202A..U+202E and U+2066..U+2069 return `attachment_invalid`
- **AND** fewer than eight non-space characters returns existing `thought_too_short`

#### Scenario: Complete deterministic extraction
- **WHEN** UTF-8 bytes begin with one optional BOM and contain CRLF or CR line endings
- **THEN** only that leading BOM is removed and line endings become LF; all remaining text is retained exactly
- **AND** Markdown syntax, HTML and URLs remain inert literal text, without rendering, execution or subresource retrieval
- **AND** original-byte hash and byte count describe bytes before extraction

### Requirement: Attachment confirmation provenance
Attachment confirmation SHALL use the existing receipt-bound capture preview and confirmation behavior, preserving the selected snapshot as each pending draft's source.

#### Scenario: Immutable selected source
- **WHEN** a valid attachment receipt is previewed with its exact extracted text and suggested channels
- **THEN** preview binds the complete source snapshot and selected destinations
- **AND** confirm invokes existing thought-drafting only after its matching expected revision
- **AND** changed text, unknown receipt and stale selection fail with `revision_conflict` before a provider call
- **AND** complete retries, concurrent confirms, restarted receipts and incomplete claims follow existing capture semantics
- **AND** historical plain-text and URL records retain their meanings

### Requirement: Attachment content custody
Attachment intake SHALL retain extracted source and provenance only through existing capture and piece records, with no public upload or raw-byte store.

#### Scenario: Local record and content-free audit
- **WHEN** an attachment is accepted, previewed and confirmed through a fake provider
- **THEN** its original bytes are not saved as an upload or served through an asset path
- **AND** selected source text and attachment metadata persist in existing local source records
- **AND** existing provenance events contain ids, hashes, statuses and usage, never filenames, claimed MIME, source content or credentials
