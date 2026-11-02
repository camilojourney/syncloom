# Design

## Context

See [proposal](proposal.md). Existing documentary owners are research/decision READMEs and their Python checker. The current Decide interface permits domain-qualified decisions; legacy records keep flat identities.

## Goals / Non-Goals

Keep immutable report bytes separate from editable custody/navigation notes. Do not turn recommendations into accepted product choices or recover reports from summaries.

## Decisions

Use the existing card/index and source-hash tables, not a search service or second catalog, per [research/0001](../../../docs/decisions/research/0001-research-custody-and-retrieval.md). This is also `docs/decisions/research/0001-research-custody-and-retrieval.md` for the record checker backlink.

Extend the existing checker to resolve decision links from each record's directory and number decisions per domain; reject a competing checker or mass migration. Code Verify alone owns the method-record claims/trigger schema.

Kept: original provider reports and historical operator summaries, because independent authors own them and unchanged custody is required; annotations belong only in cards. Existing historical study directories retain their layout.

References: security, data. Integrations, llm and ui were inspected for evidence boundaries, but this change makes no provider calls, prompt or rendered-screen changes. The OpenSpec context's Code 5.0.0 mention is historical; actual loaded owner is Code 5.4.2.

## Risks / Trade-offs

- Missing source result -> mark unknown; never infer no study or completed research.
- HAND text beside a native report -> retain original native incomplete outcome and zero-source challenger limitation.
- New domain path -> test both valid traversal and broken evidence/backlink through the existing checker.
- Source/report URLs -> navigation only, not remote reachability or factual verification.

## Security boundaries

Only the authorized worker copies named approved public-question research bytes. No credentials, private transcripts or browser state are imported. Local record checker reads repository files, logs paths/errors only and performs no network calls. Task-scoped trace read keeps credentials in memory. Runtime guards and secret stores are untouched.

## Migration Plan

Add omitted historical cards/reports and link current records. No report move or rewrite. Revert this local commit to roll back navigation/checker changes; original sources remain retained.

## Open Questions

External catalog IDs and the missing hosting result await Phrona's owner. Neither changes independent preservation.
