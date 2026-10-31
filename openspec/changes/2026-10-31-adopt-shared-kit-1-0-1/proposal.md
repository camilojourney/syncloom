## Request

> the c configuraion and the tools we use there like the quality and owther should be versioned. so we can use tthem and improve all the other projects at the same time

## Approval

pending

New capability requested: none.
Duplicate kept: materialized Fleet configuration and checker, because repositories need standalone execution with immutable source ownership.

## Why

Kit 1.0.0 lacks the reviewer-token repair and Fleet checker custody. Adopt the next reviewed snapshot without changing product behavior.

## What Changes

- Adopt Shared Kit 1.0.1 from Fleet release f63507c26ebf9510bec1e92f692919ec62f12270.
- Merge baseline 1.0.2 preserving project context; materialize checker 1.0.0.
- Update the development record, retaining Rust profile 1.0.0 and recording Code 5.2.1 tested-with evidence.

## Capabilities

No new or modified product capabilities. Reviewed the OpenSpec capability inventory; none owns repository tooling. Set skip_specs for this tooling-only adoption.

## Impact

OpenSpec guidance, record checker and development documentation only. No global installation, dependency update or publication.

## Open questions

None.
