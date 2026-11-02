# Design

References: security, ui, data; Code 5.4.2.

## Decisions

Panel sentences live in the existing page status projection, keyed by stable reason codes plus `Channel::display_name`. Rejected changing `Connection.message`: that string is the API contract. Rejected copying grant or identity checks into templates.

Receipt and readiness notes call that one projection. Unknown cannot-post reasons use the missing-permission sentence so raw connection text never reaches the panel.

Disabled Publish changes only `button:disabled` in `studio.css`. Fill uses `--surface`, label `--muted`, border `--border`. Rejected panel opacity, hiding reconnect, and restyling enabled accent buttons. `--subtle` and `--accent` read as active fills in one of the themes.

Approval time stays the last matching review event. The user-path test advances only the studio clock before fresh approval. If that date already matches the new event, this is a fixture proof correction, not a policy change. Rejected rewriting stored events or DOM dates.

## Security boundaries

Who may act is unchanged: same session, CSRF, Host and Origin guards. The page displays the existing reason; it does not decide it. No new secrets or logs.
