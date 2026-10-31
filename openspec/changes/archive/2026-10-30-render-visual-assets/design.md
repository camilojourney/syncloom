# Design

References: security, data, integrations.

## Context

See proposal.md. Q7's verbatim reports and the old engine are read inputs; docs/decisions/0003-visual-renderer.md records the renderer choice.

## Goals / Non-Goals

Keep rendering local and revision-bound. No arbitrary HTML, browser distribution, remote fonts, uploads or template migration of all historical layouts.

## Decisions

- Use a small Rust CDP client over a private loopback debugging socket and one fresh Chromium per render, rather than a broad browser automation layer or long-lived browser; this bounds lifecycle and isolates content. Pin executable path/version in configuration, require exact Browser.getVersion product, and kill/reap the child on success, failure and deadline.
- Use software rendering, DPR 1, sRGB, disabled font hinting and a checksum-pinned embedded Arial font instead of system CSS fallback. Await document.fonts.ready, reject unsupported glyphs instead of using fallback fonts, and reject layout overflow rather than silently clipping.
- Generate the gradient as a PNG in Rust and embed it as a data URL, rather than emitting PDF gradient shadings; Q7 documents Preview's slow shading path. Print all sections in one HTML document with @page size 1080px 1350px and page breaks, not the old per-page merge/fallback.
- Accept structured slides and two palette templates, escape text, block network requests and apply CSP, rather than allowing arbitrary HTML, URLs or file access. No personal data is sent in headers; content exists only in the local CDP pipe/socket.
- Preserve text-only revision hashing; visual revisions add the template version, palette and slides. Clear assets/approval before changed input renders, recheck revision after the subprocess, and clear approval on every successful rerender instead of trusting earlier reviewed bytes.
- Persist immutable content-addressed bytes through the existing atomic file-store owner before publishing their manifest in the piece record. Require the caller's expected image/PDF hash pair at approval and verify both files, rather than approving whichever bytes a concurrent same-revision rerender last stored; visual publish stays contained even with LinkedIn granted.
- Normalize PDF CreationDate, ModDate and trailer ID with lopdf before hashing, rather than claiming native PDF byte stability. Chromium rounds the CSS paper height to 1013.04 points on this binary; normalize every MediaBox to 810x1012.5 points, trimming only the fractional blank edge rather than accepting incorrect dimensions. Test PNG and normalized PDF repeat stability on this pinned Mac, not across platforms.
- Reuse review and provenance owners; add optional serde-default fields so existing records survive, rather than migrating all historical text pieces.

## Boundaries

The existing loopback Host guard admits local callers for every new route; studio-generated ids and fixed asset kind names select files. Only lowercase SHA-256 names are accepted, symlinks are refused, and file bytes are checked against manifests; logs hold fixed codes, ids and hashes only. Browser config is operator-trusted, not request-controlled; credentials remain in existing environment/Keychain owners and never enter the renderer.

## Risks / Trade-offs

- Platform-specific rasterization: pinned binary/font and a 0.1% pixel tolerance at channel delta >8; inspect new goldens visually, never rebaseline a failing golden.
- Chromium is a critical optional feature dependency: failed rendering does not break text-only operations, but visual approval fails closed; each call has one total deadline and no retries.
- Detached content-addressed files after a storage failure are harmless; no automatic deletion or recovery layer is introduced.

## Migration Plan

Optional fields default absent; old text records and revision fixtures stay unchanged. Rollback to the pre-change binary requires retaining visual records for a later forward-compatible reader.
