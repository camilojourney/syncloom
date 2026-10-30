# Local visual assets

Slice 3 renders locally, not to a social platform. Q7's renderer decision is [0003](decisions/0003-visual-renderer.md); behavior is in the [render-visual-assets change](../openspec/changes/render-visual-assets/).

## Render, view, approve

After creating a text piece, take its id and revision from `show`:

```sh
syncloom render <piece_id> --expected-revision <revision> --template midnight \
  --slide "Build with evidence." --slide "Review the exact bytes."
syncloom view <piece_id> --kind image > /tmp/piece.png
syncloom view <piece_id> --kind carousel > /tmp/piece.pdf
syncloom approve <piece_id> --expected-revision <rendered_revision> --image-sha256 <image_sha256> --carousel-sha256 <carousel_sha256>
```

Render returns the updated piece as JSON. Templates are `midnight` and `paper`. Slides are plain text, one to ten nonblank slides, at most 800 characters each; text must fit the layout and exist in the pinned font. Unsupported glyphs and overflow return `invalid_request`, never clipped or fallback-font output. The image shows slide one; the PDF holds every slide in order. View writes raw bytes, not JSON.

The matching API:
- `POST /v1/pieces/{piece_id}/render`: exactly `{"expected_revision":"...","template":"midnight","slides":["First slide","Second slide"]}`, returns 200 with the piece.
- `GET /v1/pieces/{piece_id}/assets/image`: `image/png`.
- `GET /v1/pieces/{piece_id}/assets/carousel`: `application/pdf`.
- Existing `PATCH /v1/pieces/{piece_id}` approves with `{"status":"approved","expected_revision":"...","expected_assets":{"image_sha256":"...","carousel_sha256":"..."}}`, using the hashes returned in the render manifest.

Byte responses include a SHA-256 ETag, `Cache-Control: no-store` and `X-Content-Type-Options: nosniff`. All routes retain the loopback Host guard, fixed error envelope and content-free logging.

## Revision and byte identity

A visual piece stores its slide text, template and template version. Its revision hashes those together with the piece id, channel and text. Text-only revision hashes stay compatible.

Changing text, slides or template invalidates assets and approval. Every rerender also invalidates approval, including a failed rerender. The returned `assets` manifest records the revision, both byte hashes, page count, browser version and font hash. Approval verifies both files against that manifest and requires the caller's expected hash pair; missing hashes return `assets_required`, and stale hashes return `asset_conflict`. This also protects a rerender that replaces bytes without changing the content revision. Provenance records the exact approved hashes. A concurrent edit rejects a render's completion with `revision_conflict`.

Bytes live under the data directory's `assets/<sha256>`, with atomic writes through the existing store lock. Old bytes are retained locally; this slice does not implement cleanup or upload. A visual piece's publish stays `contained` even when LinkedIn delivery is granted, so it cannot silently dispatch as text.

## Renderer configuration

No installation, download or redistribution code exists. The operator supplies an already installed executable; the checked development default is:
`$HOME/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell`.

| Variable | Default |
|---|---|
| `SYNCLOOM_RENDERER_PATH` | local executable above |
| `SYNCLOOM_RENDERER_VERSION` | `145.0.7632.6`, checked against CDP Browser.getVersion |
| `SYNCLOOM_RENDERER_FONT` | `/System/Library/Fonts/Supplemental/Arial.ttf` |
| `SYNCLOOM_RENDERER_FONT_SHA256` | `525979822591a3447cfc49d943d6f7683508e25543407871c0ed8fed05fd2bd9` |

Configuration is shared by CLI and server. The whole render has a 30-second deadline. Missing binary, wrong browser version, wrong font, timeout and browser crash are distinct fixed errors; see the [error fixtures](../contracts/studio-api/errors/). Failed renders never expose partial assets.

The renderer uses software rendering, DPR 1, sRGB and no font hinting. It embeds the checksum-pinned font and a pre-rasterized gradient, escapes all copy, blocks URL loads and waits for font readiness. It prints one HTML document with `@page size 1080px 1350px`. PDF dates and trailer identity are stripped, and Chromium's rounded paper box is trimmed to exactly 810x1012.5 points.

## Evidence and maintenance

`cargo test` exercises the real pinned binary, API, CLI and process-boundary failure fixtures. PNGs and normalized PDFs repeat byte-identically on this Mac; cross-OS identity is not promised. The two reviewed PNG snapshots permit no more than 1,458 pixels (0.1%) with any RGB-channel difference above 8/255.

The tests check two- and ten-page PDF geometry, selectable text, raster rather than PDF gradient shadings, unsupported font fallback, literal markup, integrity damage, symlinks, missing assets, stale approval and edits during rendering. Every renderer failure has an HTTP fixture. Mutation checks must reject removed revision and approval guards.

Template layout changes must increment `TEMPLATE_VERSION` in the [renderer](../crates/syncloom-core/src/visual.rs). Do not rebaseline a failing snapshot; investigate it. Browser upgrades require a deliberate config and snapshot review. First-run distribution, Typst and a review screen remain later decisions.
