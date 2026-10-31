# Design

References: security, ui.

## Decisions

- Channel owns display names separately from machine ids; changing serialization would break saved data and API callers.
- Keep browser consent in server-session memory, keyed by piece id and bound to revision plus the full CostEstimate. Reject persistent Piece schema changes because this is browser form state, not a new core publishing policy; restart safely requires acceptance again.
- Compare submitted revision, micro-USD and pricing date against the current piece and estimate before recording consent. Invalidate mismatches when rendering or publishing; a stale tab cannot transfer consent to another revision.
- Only browser Publish injects the recorded accepted amount into the existing core request. Ignore client-supplied amounts for that action and refuse absent consent even for contained publishing; JSON and CLI keep their existing explicit confirmation paths.
- Keep delivery guidance, availability and consent together in the publish-controls renderer rather than growing the overall page renderer; refactoring preserves the exact emitted HTML.
- Native POST navigation replaces scripting, preserving CSP and accessible disabled styling. Use a nowrap time element for the date rather than widening the card.

## Security boundary

The laptop operator acts through existing Host, Origin and CSRF checks for every form POST, including accept-cost. No token or draft enters consent records or logs; session secrets stay in Boundary. Consent never approves or publishes, and core revision/cost checks still run at dispatch. Tests must reject missing/foreign sessions, stale revision, wrong amount/date and unrecorded browser publish.

## Trade-offs

Consent is lost on server restart and must be accepted again. This is fail-closed and avoids new durable storage or policy owners. No migration is needed; rollback loses only transient consent.
