# ADR-0003: Pinned headless Chromium for local images and document carousels

**Date:** 2026-10-30
**Status:** Proposed
**Supersedes:** none
**OpenSpec changes:** [render-visual-assets](../../openspec/changes/render-visual-assets/)
**Agreed:** partly - the captain accepted the visual slice after Q7 in vision 6.2; the slice 3 implementation brief selects Chromium and excludes browser distribution and platform uploads
**Recommended:** keep Chromium as production renderer; gate any Typst replacement on a later spike
**Built:** implemented on branch `fm/syncloom-v2-slice3-visuals-20261030`; not yet landed
**Live-proven:** partly - real local Chromium renders tested through API and CLI; no platform upload or network integration is claimed

## Context

The old renderer used HTML/CSS but merged per-slide PDFs and could silently return only page one. Q7 supports retaining Chromium while treating platform-specific rasterization and slow PDF gradients explicitly.

## Decision

Use Rust CDP calls to an operator-supplied, version-pinned Chromium. Produce an image and a single-document carousel, with raster gradients, an embedded checksum-pinned font and normalized PDF metadata. Rendering, byte integrity and review behavior belong to the change's `visual-assets` and `draft-review` specs.

Keep the slice local. A conservative ten-slide cap follows the slice brief's Q3 guidance; it is not a claim that all Meta products share that limit.

## Rationale

| Option | Why chosen or rejected |
|---|---|
| Chromium R1 | Chosen: retains HTML typography and layout; measured locally with pinned inputs |
| Typst R2 | Gated later: Q7's conic-gradient and typography equivalence questions need a focused spike |
| resvg R3 | Not primary: wrapping and conic gradients would require another layout engine |
| WKWebView R4 | Not primary: OS-bound rendering and lifecycle complicate stable headless operation |
| Per-slide PDF merge | Rejected: print all sections once, without incomplete-output fallback |
| Live PDF gradients | Rejected: Q7 documents slow Preview shading; embedded PNG backgrounds keep text vector |
| Automatic browser install | Deferred: distribution and first-run downloads require a separate decision |

## Scope and boundaries

No image/document upload, new platform, review screen or Typst spike. Browser paths are trusted operator configuration; requests carry only structured plain text. No remote fonts, URL fetches or arbitrary HTML. Existing LinkedIn text behavior remains intact; visual pieces stay contained.

## Identities

OpenSpec CLI 1.15.0; `openspec/config.yaml` blob `8f976f578610adeefde25096a98101b9f9c2c12c`; Code 5.0.0 with security, data and integrations references. Chromium headless shell Playwright cache revision 1208, product version 145.0.7632.6; Arial checksum and executable configuration are in [visual-assets](../visual-assets.md). Rust crates: tokio-tungstenite 0.28.0, lopdf 0.38.0, png 0.18.1, ttf-parser 0.25.1, resolved in Cargo.lock. The build commit is recorded by the branch tip and eventual merge, not claimed landed here.

## Evidence and gaps

- [Q7 rendering study](../research/2026-10-30-q7-render-images-carousels-rust/README.md): verbatim reports, checksums, collection limits and renderer alternatives.
- [Q3 Meta study](../research/2026-10-30-q3-meta-facebook-instagram/README.md): carousel constraints; this slice uses the brief's conservative ten-slide cap without any platform call.
- [Local operation and test evidence](../visual-assets.md): repeat bytes, snapshot tolerance, page dimensions and revision/integrity checks.

The ChatGPT challenger scanned zero sources and weighs below the cited Claude report. Cross-machine pixel identity, accessibility conformance, actual Preview latency, first-run distribution and the Typst spike remain unproven. Local tests demonstrate selectable PDF text, not full PDF/UA compliance.

## Consequences

**Positive:**
- Approval requires the caller's expected image and carousel hashes, including after same-revision rerenders.
- No browser installer or platform side effect enters this slice.
- Print geometry and overflow failures are explicit.

**Negative / Trade-offs:**
- Rendering depends on an existing pinned browser and font.
- One fresh browser per render favors isolation over warm-start latency.
- Content-addressed historical bytes remain until a later cleanup design.

## References

- [Vision](../vision.md), [migration ledger](../migration.md), [visual operations](../visual-assets.md)
