## MODIFIED Requirements

### Requirement: Delivery grant
`SYNCLOOM_DELIVERY` SHALL name the platforms whose intents are dispatched; other platforms' intents stay contained, and an unknown name refuses startup.

#### Scenario: No grant
- **WHEN** `SYNCLOOM_DELIVERY` is unset or blank and an approved piece is published
- **THEN** the intent is `contained` and the fake platform received no request

#### Scenario: LinkedIn granted
- **WHEN** `SYNCLOOM_DELIVERY` is `linkedin` and an approved `linkedin_text` piece is published
- **THEN** one dispatch attempt is made

#### Scenario: Meta grants
- **WHEN** SYNCLOOM_DELIVERY names facebook or instagram
- **THEN** the corresponding channel uses its Meta adapter and the other stays contained unless separately granted

#### Scenario: Unknown platform
- **WHEN** `SYNCLOOM_DELIVERY` is `linkedin,tiktok`
- **THEN** the process exits with an error naming `SYNCLOOM_DELIVERY` and binds no port

