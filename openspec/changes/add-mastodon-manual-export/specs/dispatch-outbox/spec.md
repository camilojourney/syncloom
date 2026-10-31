# Dispatch delta

## ADDED Requirements

### Requirement: Mastodon delivery grant
The `mastodon` delivery grant SHALL dispatch Mastodon text and verified visual pieces through `mastodon-integration`, leaving all other grants unchanged.

#### Scenario: Mastodon and manual
- **WHEN** delivery names `mastodon`
- **THEN** an approved Mastodon piece makes one adapter attempt; without the grant it stays contained
- **AND** manual-export pieces never become API posts, even when another platform is granted

### Requirement: Concurrent attempts excluded
Concurrent publishes SHALL claim a retryable intent once under the same durable guard before any network operation.

#### Scenario: Duplicate during an attempt
- **WHEN** two callers publish the same approved revision concurrently
- **THEN** only one adapter call occurs and neither a stale approval nor a stale intent snapshot authorizes another attempt
