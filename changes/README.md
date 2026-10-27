# Change history

Every orchestrated change has one `YYYY-MM-DD/<change-id>/record.md`.
This directory holds history, never the orchestrator's live to-do list.
The repository is adopted when this file exists. Outside orchestration, use
commit messages for small direct fixes only in non-adopted repositories.

Run the shared code skill's `scripts/change.py init` once to adopt.
Before the first edit, run `change.py start --id <slug> --request-file <instructions>`.
The default request is the exact body of `## Captain's intent`, or the whole
file when that heading is absent. Use `--section <heading>` to select another
section, or `--request <text>` outside orchestration. Never edit the copied request.
`start` commits only the new record, retaining other staged work.

Use the bundled `assets/change-record.md` template through `start`; do not
maintain another template here. Complete scope (in/out), requirement IDs,
findings, decisions with their approval, actual changes, proof and links as
work happens. Findings has an `Other owners` subsection for out-of-scope defects:
name the owner, violated requirement or rule, counterexample, evidence and handoff
or disposition. Write `None.` there when empty; do not invent a handoff file.
Links name orchestrator task ID, session ID and trace, or explain
why a link is unavailable. Record amendments in Decisions, never in Request.

Proof holds only the table, with columns `Requirement | Evidence | Result`.
Put explanatory narrative in Findings, never below the Proof table.
Use one current requirement
ID per row. Results: PASS, FAIL, NOT_RUN, NO_CHECK, MANUAL_PENDING, EVAL_PASS,
EVAL_FAIL, BLOCKED. Every requirement listed in Requirements needs a proof row.
Every unproven result needs remaining work in What's left. Write `None.` only
when no work remains. A revised requirement needs a Decisions line naming its
new ID and approval source. Put the code skill's final report in Report, and
keep that report as the final message too.

For example, a Decisions approval line names the full revised ID and source:

```text
req~insertion.at-start~2 approval: Captain's task instructions, section Captain's intent.
```

Pass `--change <slug>` to the shared `verify.py`. It writes `evidence.json` for
the tested tree; never write that file by hand. Complete the narrative and
provisional proof before verification, then finish the report afterward.
Logs and private receipts remain outside the repository. Requirement results
are derived from coverage tags and suite status, not individual test reports.

After rebasing onto a moved default branch, replace the old comparison base with
the new fork point. Rerun `verify.py --change <id> --base <new-fork-point>`, then
`change.py check --base <new-fork-point>`. Keep the record's original front-matter
`base_commit` as history; the new evidence binds to the base actually verified.

In adopted work, `--base <ref>` means `git merge-base <ref> HEAD`, resolved once
for all checks in the invocation. Thus `main` means its fork point with HEAD,
not a default-branch tip that can advance independently. An immutable fork-point
commit also works. An old base that includes another landed record is rejected
with the added record paths and the re-verification instructions.

Run `change.py check --base <comparison-ref>` before landing. It checks the
prospective commit tree, including untracked files. Every change outside this
directory requires exactly one added record. Missing, rejected or stale proof
blocks landing. Only the record's own folder may change after verification.
This check is local evidence validation, not a cryptographic attestation.
Repository CI and landing integration are the adopter's responsibility.
