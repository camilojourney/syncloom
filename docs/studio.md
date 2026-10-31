# Laptop review studio

Start `syncloom-server` as described in the [README](../README.md), then open `http://127.0.0.1:8790/`. If using `SYNCLOOM_ADDR=127.0.0.1:0`, use the port in the listening log. Do not substitute `localhost`: exact Host matching is intentional. The page is laptop-only, not a phone access path.

## Review loop

1. Enter a thought and choose **Draft a piece**. Claude drafting may take up to a minute. Saved pieces made by the CLI also appear in **Thoughts & pieces**; this is the same store and core, not a browser-only queue.
2. Open a piece. Read the source, saved draft, channel fit and full revision. **Edit draft** opens a separate editor without approve/publish controls. Saving a changed draft invalidates approval and previous rendered assets; canceling leaves the saved version unchanged.
3. Open **Render previews**, choose a template and enter one slide per line. This editor has no approve/publish controls, so unsaved slide changes cannot be mistaken for saved assets. Submitting **Render previews** calls the existing renderer, which may take 30 seconds. Read the PNG and scroll through the actual PDF at phone widths, or open their verified-byte links. The PDF's page count and both complete SHA-256 hashes appear beside the previews. This previews a small screen on the laptop; it does not expose the server to a phone.
4. **Approve exact revision** submits the displayed saved revision and hashes. Stale revisions, missing or mismatched assets, or damaged bytes fail with the same core codes as the API and CLI. No approval is silently refreshed.
5. **Publish** is separate. With no delivery grant, the receipt says `contained`, the attempt count is zero, and nothing was sent. A repeated contained publish returns the same intent. LinkedIn visual uploads remain contained. Granted Mastodon delivery can upload the approved PNG; PDF carousels stay local. Other configured outcomes, including `unknown`, are displayed as returned by the core, never renamed to posted.

The browser's native navigation/loading indicator covers form submissions. Empty views explain where to start; errors show the fixed core/transport code and request id with a recovery link. There is no client-side script runtime. Every action is keyboard accessible and the layout works at narrow widths. The page currently has one light palette, not a dark-mode variant.

## HTTP session handshake

The standalone CLI calls Studio directly. HTTP API clients must fetch `GET /v1/session`, read `{"token":"..."}`, and send the token once as `X-Syncloom-Token` on state-changing calls. Forms carry the same token once in their `csrf` field. The token is unpredictable, memory-only and changes with every server start; it is never a URL parameter, cookie or logged field. Do not persist it in scripts or source control.

Every route, including unknown routes and assets, checks the exact bound `127.0.0.1:<port>` Host. A supplied Origin must be the single exact `http://127.0.0.1:<port>` value. Cross-site and same-site fetch metadata are rejected even if Origin is absent; native clients may omit both. Rejections use `host_not_allowed`, `origin_not_allowed` or `session_required` before any model call or mutation. No response contains CORS headers.

All responses are no-store and nosniff, with a same-origin referrer policy (no off-origin disclosure), and have restrictive CSP. Pages cannot be framed and scripts are forbidden. Verified PDFs alone allow same-origin framing for the inline preview; off-origin framing is still forbidden. These controls defend the web boundary against cross-site requests and DNS rebinding, not malware already running on the laptop.

## Saved-piece discovery

`GET /v1/pieces` returns `{"pieces":[...],"next_after":"..."}` in ascending piece-id order, up to twenty records. Pass `next_after` as `after` for the next page; null means the end. Empty stores return `{"pieces":[],"next_after":null}`. An invalid cursor returns `invalid_request`; corrupt selected records or symlinks return `internal_error` instead of silently hiding data. Temporary interrupted writes are ignored. Concurrent inserts before the cursor appear when starting a new traversal, not retroactively inside an existing page.

## Offline development proof

Use the [fake Claude example](../README.md#run-and-check), an isolated `SYNCLOOM_DATA_DIR` outside the repository, and leave `SYNCLOOM_DELIVERY` unset. No real model key or social account is needed. Rendering requires the pinned local browser and font from the [visual asset guide](visual-assets.md).
