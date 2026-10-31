# Dispatch outbox delta

## ADDED Requirements

### Requirement: X pre-publish estimate
X dry runs SHALL return an integer-micro-USD estimate requiring matching `accepted_cost_microusd` confirmation before delivery reserves an intent.

#### Scenario: Published pricing
- **WHEN** text has no URL-like token
- **THEN** the estimate is USD 0.015, plus USD 0.005 if PNG alt metadata is needed
- **AND** a URL-like token uses USD 0.200 instead of USD 0.015
- **AND** the response names USD, the pricing date and that this is an estimate, not a charged amount

#### Scenario: Explicit confirmation
- **WHEN** an approved X revision is delivered without matching confirmation
- **THEN** the answer is cost_confirmation_required, no intent is reserved and no request is sent
- **AND** the review confirmation and CLI/API show the estimate before the founder supplies confirmation

### Requirement: Durable X estimate
The X estimate SHALL survive restart alongside each intent's outcome, including uncertain and failed outcomes, without claiming an actual charge.

#### Scenario: Repeat
- **WHEN** a posted or uncertain X intent is repeated after restart
- **THEN** its original estimate and truthful outcome return without another request
