# Studio API contract fixtures

Owned by the OpenSpec capability `studio-api`. Each JSON file is `{"status": <int>, "body": {...}}` as the server returns it for the request id `fixture-request-id`.

- `errors/<code>.json` - one file per error code. The message is the fixed text for that code; it never carries request content. The core error table (`crates/syncloom-core/src/error.rs`) and the server's status map hold the same values, and `crates/syncloom-server/tests/contract.rs` proves every file equals what the server returns.

Do not edit a file to make a test pass: change the spec first.
