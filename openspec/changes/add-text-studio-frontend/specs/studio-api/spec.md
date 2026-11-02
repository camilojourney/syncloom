## ADDED Requirements

### Requirement: Browser text channel selection
The thought form SHALL accept selected channels through strict form fields and delegate validation to thought-drafting.

#### Scenario: Repeated channels and strict scalars
- **WHEN** a same-origin form carries one csrf, one text and multiple channels fields
- **THEN** their submitted order reaches thought-drafting, and success redirects to the first saved piece
- **AND** absent or duplicate channels follow thought-drafting errors; unknown fields and repeated text return invalid_request
- **AND** missing or repeated csrf remains session_required

### Requirement: Comfortable text review
Every browser surface SHALL preserve readable, unclipped light/dark layouts, keyboard access and honest local text previews.

#### Scenario: Saved text and context
- **WHEN** a saved piece is reviewed
- **THEN** full draft whitespace, conservative channel fit and source-language review target are displayed
- **AND** its already-prefixed handle appears once; missing routing is labeled not recorded
- **AND** previews claim neither connected-account identity nor remote font, fold or layout parity
- **AND** home, review, edit, existing render and error views work at narrow/wide sizes without scripts

#### Scenario: Plain words and exact approval
- **WHEN** a writing or error page is opened
- **THEN** natural outcome and character-count words appear first, with technical codes, request identifiers, attempts, receipt fields and exact counting rules in closed Details
- **AND** the character label preserves the existing core counting and validation semantics
- **AND** an already-approved exact saved revision shows approved and its recorded approval date, or date not recorded when unavailable, without a second approval action

### Requirement: Composed browser actions
Browser publish affordances SHALL respect revision approval and the current publish intent, independently of the piece's last-operation summary.

#### Scenario: Unknown or posted intent after export
- **WHEN** a current revision has a posted, posting or unknown publish intent and is then exported
- **THEN** review still suppresses a fresh publish/retry action and shows that publish record separately from the last export
- **AND** persisted posting is unconfirmed, not proof of active progress
- **AND** approval alone does not claim account or delivery readiness

#### Scenario: Receipt availability
- **WHEN** the review page is reloaded
- **THEN** receipt details come from core read inspection when recorded, with unavailable values labeled honestly
- **AND** inspecting a record never invokes publish or changes state
- **AND** disclosures have distinct descriptive labels, and the latest-action summary is omitted only when the visible banner shows that same saved result; separate history remains visible
