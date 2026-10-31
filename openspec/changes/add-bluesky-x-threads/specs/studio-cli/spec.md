# Studio CLI delta

## ADDED Requirements

### Requirement: Q1 connection commands
The CLI SHALL offer `connect <bluesky|x|threads> sign-in|status|refresh|sign-out`, returning core JSON or existing fixed errors.

#### Scenario: Secret inputs
- **WHEN** Bluesky sign-in takes a handle or Threads consent awaits its external callback
- **THEN** secrets are read from bounded stdin, hidden on a terminal, never from command arguments
- **AND** X waits for the state-bound loopback callback

### Requirement: X cost confirmation option
The CLI SHALL accept `--accepted-cost-microusd` on publish, using the estimate returned by its dry-run operation.

#### Scenario: No confirmation
- **WHEN** granted X delivery is requested without exact acceptance
- **THEN** cost_confirmation_required is printed and no platform request is sent
