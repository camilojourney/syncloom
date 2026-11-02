# Proposal

## Request

> To collect all the research that we did, take decisions on it, and start building. Remember that I need you to go and check all the Prona research that we did. These have to be remembered. We have to have a connection between the links and the place where they actually belong. Remember that we updated Prona to have a domain specific for that research so we can look for it faster and check the composition for each research. We need to have this properly set up. In the future I can just look at it faster and find them directly.
>
> For this domain: list every Phrona study this home requested or used (home data and the repository's docs/research), with its links; for each check whether its findings were decided through /decide and acted on; decide any finding not yet decided and build what it supports, in parallel slices where independent; make sure each report is saved word for word in docs/research with provider links and linked from its decision record. Phrona's owner is building a domain catalog under Main1308; add its catalog id to each entry once delivered. Return one table: study, domain, decision (id or none), built (yes / in progress / no with reason). No Claude workers.

## Approval

pending

## Why

The existing index omits retained YouTube reports and the failed original auth run. It also repeats Q1 as pending and obscures native incomplete outcomes behind recovered reports.

## What Changes

- Preserve missing original reports and receipt provenance in existing research cards.
- Link study composition, provider URLs, actual implementation and genuine decision-method gaps.
- Keep catalog IDs pending until Phrona's owner delivers them.
- Extend the existing record checker for domain-qualified decision locations without changing historical identities.

## Capabilities

### New Capabilities

None. Documentation and its existing checker only; `skip_specs: true`.

### Modified Capabilities

None. Checked the stock capability list; `provenance-log` owns product events, not documentary research. Research/decision READMEs remain the format owners.

## Impact

Research cards/index, decision index/new archive decision, existing Python checker/tests. This slice changes no runtime or frontend design. Follow-on findings need prospective decisions and dispatched/built/deferred outcomes; this archive alone does not fulfill the whole intent.

## Reuse inventory

Timing: before first edit; inventory receipt retained in task data before this proposal.
Search: research/decision owners, all home data report references, named original batch receipts, checker/tests and stock capability inventory.
Candidates: existing cards, `check-research-records.py`, Code Verify's method-record check. No separate catalog or validator.
Decision: extend existing owners; new cards represent omitted original studies, not duplicate reports.

## Open questions

Where is the original media-hosting result? Which real catalog IDs will Main1308 deliver? Product privacy, budget and live-service choices remain outside this change.
