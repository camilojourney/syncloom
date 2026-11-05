# Design

## Context

See [proposal](proposal.md) and [scope decision](../../../docs/decisions/attachments/0001-local-text-scope.md). Read old `capture.py`, `capture/suggest.py`, browser picker paths, route tests and binary-contract-gap decision at `3efcd35370242a37985fbacb272dacf80b2d42dd` without executing the old application. It classified image/PDF metadata, not bytes; its probes and confirm-and-publish coupling are not ported.

References: security, data, integrations, llm, ui. Installed Code 5.4.4, plus strategic-design and self-review/reuse/residue references, loaded before project edits.

## Goals / Non-Goals

Extend existing capture snapshots with user-selected local text bytes. No arbitrary path reads, raw upload/artifact registry, public upload location, Markdown rendering, OCR, PDF, Office/archive parsing, connection probes, imports, autonomy, Research Radar or vision judging.

## Decisions

- Scope A is firstmate's ordinary build-sequencing decision, not final input-security acceptance; alternatives and evidence stay in the linked decision. Capture remains partial rather than relabeling unsupported PDF/images as migrated.
- One pure core attachment parser owns extension/MIME hints, strict UTF-8 validation, filename/text controls, BOM/newline extraction and exact bounds. Reject generic binary decoding, lossy replacement and trimming/truncation because they cannot preserve actual submitted content.
- Extend optional Source metadata and its text accessor; reuse existing capture records, source digests, channel-set binding and durable confirm claims. Reject another upload store or client-asserted provenance; original bytes are hashed then discarded, while historical text/URL records deserialize unchanged.
- Share snapshot persistence between URL retrieval and attachment acquisition at the existing Studio owner. Keep network policy in SourceClient and file-format policy in the attachment parser, because they change independently; do not generalize them into a document service.
- Use Axum's maintained multipart extractor behind the existing bounded request reader, not custom boundary splitting or browser JavaScript/base64. The single upload adapter validates multipart shape; Boundary remains the sole token/Host/Origin/Fetch-Site permission owner and passes validated upload data to the handler without reparsing.
- Acquire one of two upload permits before reading, impose a ten-second read/parse deadline and retain the existing 1 MiB request cap. Reject excess concurrent work immediately rather than building an unbounded wait queue; all permits release on errors/cancellation.
- Accept absent or generic octet-stream MIME only as a hint, because native pickers may omit type; actual UTF-8/control validation is mandatory. A supported extension is required but never sufficient, and unsupported claimed MIME is refused rather than silently overridden.
- Keep extracted text complete, display source provenance through the existing shared owner, and show normalized provider input on attachment preview. Reject a second editor or the misleading claim that unnormalized bytes are the model input; thought-drafting's existing whitespace normalization remains unchanged.
- Preserve native form loading and current tokens/layout/typography using pinned Impeccable 4.8.0 read-only guidance. Its executable/detector is absent; critique/audit are explicitly degraded single-context assessment, not independent review.

## Residue disposition

Kept: crates/syncloom-server/src/capture.rs ~ crates/syncloom-server/src/page/capture.rs because the new jscpd fragment is the two transport-specific attachment handler signatures and their identical call to the single Studio acquisition owner. JSON returns that snapshot; HTML renders suggestions. Neither adapter owns format, extraction, authorization, persistence or confirmation policy. A forwarding service or core-owning HTTP extractor would add a layer without removing a duplicated rule. This is one intentional clone retention, not a clean-scan claim.

Kept: crates/syncloom-server/src/upload.rs fields, because the server multipart-adapter maintainer owns this short single-pass protocol state machine: required file, duplicate-file refusal, page-only non-file CSRF and unknown-field rejection stay legible together. Cyclomatic 12 versus unchanged limit 10 is a true intentionally retained new identity, not a false finding or achieved target0. Firstmate reviewed the current source and native receipt and authorized this ordinary engineering retention in durable message 005 (2026-11-04T23:24:42Z); it is not captain security acceptance. No additional metric-only forwarding abstraction, tool, exclusion or threshold change is authorized.

The first measurement also found four new/worsened over-limit functions. Split the actual filename/format/text policies, multipart construction/field-shape policies, upload-budget authorization and native-browser text representation from orchestration. No threshold, target or exclusion changed. Re-run native measurements and all HTTP regressions after these behavior-preserving changes.

## Security boundaries

Who may act: the current single local founder session. Existing Host/Origin/Fetch-Site checks precede body handling; API headers authenticate before upload reads, and browser multipart carries exactly one CSRF field checked by the existing secret-comparison owner. Another server's token cannot authorize a snapshot or draft.

What is checked: strict multipart fields, total/file/text/filename/time/concurrency bounds; basename-only metadata; extension/MIME hints plus actual bytes; exact receipt/text/channel revision before provider use. No path is supplied to a filesystem API, no external resource is retrieved, and Markdown/HTML is escaped as literal source text. Original input bytes never become an executable, served asset or persisted upload.

What is logged: fixed route labels, ids/statuses/timings and existing content-free provenance events only. Error bodies never echo filename/MIME/content/parser details. Extracted content and provenance stay under existing local capture/piece privacy, not in request logs/traces.

Secrets: no new credential. Existing API header/form CSRF stays in memory or no-store local pages, never URLs or logs. Proof uses task-created files and fake drafting only; no Keychain/account/provider/source-host activation.

## Risks / Trade-offs

- Valid text can be deceptive or contain prompt injection: explicit preview/consent and data-only provider input grant neither tools nor approval/publication. Factual correctness and semantic safety are not judged.
- UTF-8 validation cannot establish authorship or that every printable byte sequence is meaningful text: supported formats mean the documented structural contract, not a model verdict.
- Normalized line endings/BOM removal and provider whitespace collapsing lose formatting deliberately: preserve original-byte hash/count and display both extracted and actual provider text before confirm.
- Source text/filename remain in existing private data records: no new deletion/retention policy is claimed. Refused material creates no snapshot or draft.
- Multipart adds an untrusted parser boundary: final self-review, exact negative controls and a concrete captain exact-head decision remain mandatory; artifact structure is not proof.

## Native form newline correction

Real stable-Chrome file selection/extraction followed by native Preview returned 409 before destination handling. Filtered network evidence counted five `%0D%0A` sequences and no LF-only encoding; the retained HTTP regression passes LF but fails the native CRLF representation. Receipt expiry/corruption is not supported by the identical stored snapshot and successful LF control. Browser attachment form parsing must reuse the core line-ending canonicalization before existing receipt comparison; JSON APIs remain exact and text/URL semantics stay unchanged. No second whitespace/BOM policy or revision owner is added. Evidence: task `native-form-failure.png` and `red-native-crlf.txt`; green proof remains required.

## Dated native-filename amendment: 2026-11-04

Original approved wording: "long filenames, full hashes and complete text wrap rather than being elided." Actual stable Chrome at 320px elides a long selected filename inside its native file control before upload; the accessibility value is complete and extracted provenance wraps the full name. That universal pre-upload statement did not pass. Firstmate opened both actual images and authorized only A in durable message 006 at 2026-11-04T23:49:54Z: scope wrapping to application-rendered extracted provenance/preview, preserve full accessible native selection and add the honest full-name-after-local-extraction hint. No script, CSP change, custom picker, assertion reduction or blanket clipping waiver. This is owner-authorized clarification, not retroactive compliance or security/landing acceptance. All application text, controls and navigation must still remain visible.

The original observation and images remain in task evidence (`native-filename-question.md`, `native-filename-selected-320.png`, `native-filename-extracted-320.png`); final-head screenshots remain a separate requirement. A retained red HTTP regression for the missing hint precedes that copy change.

## Spec-Impact

Adds attachment intake/provenance to thought-drafting and multipart/browser operations to studio-api. Existing capture owns suggestions, receipt/selection preview, claims, stale/restart/idempotency and pending-only confirmation. Provenance event policy, platform rules, plain-text/URL semantics and shared Rust/OpenSpec baseline remain unchanged.

## Migration Plan

Optional metadata preserves historical records. Update only built text-file support in the Capture row, keeping PDF/image/binary intake and connection probes deferred. Reverting this slice removes intake; local attachment source records require this version to interpret their metadata. No existing data is rewritten. Final screenshots must match the committed head, reviewed before readiness; exact-head security acceptance is a separate landing gate.

## Open Questions

None for implementation design. Planning/apply approval and the narrow native-filename clarification are recorded above. Final exact-head security acceptance and final image review remain unresolved landing gates.
