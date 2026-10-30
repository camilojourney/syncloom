# syncloom.v2 in Rust: evidence-backed build plan

Status: complete

## Question

What does the old Syncloom (Holus) repository hold at commit `3efcd35`, which capabilities should a Rust syncloom.v2 retain, rebuild or retire, what is the smallest end-to-end first slice, which captain decisions remain open, and which research questions are consequential?

## Why asked

The captain asked for a from-scratch Rust rebuild of Syncloom on the sound parts of the old foundation. Without a map, the rebuild would either carry over the old specs (which the captain rejected) or lose the behaviors that work (revision-bound review, the durable outbox, lineage). The report's seven decisions D1-D7 became Main's recommendations, accepted by the captain on 2026-10-30, and its section 3 is the acceptance list of slice 1.

## What would change the answer

The retain/rebuild/retire verdicts rest on the old code at `3efcd35` and on one run of its test suite; a later commit of the old repository could move a verdict. D2's recommendation ("keep the Social API external") was already changed by the captain's words of 2026-10-30 (decision 0001). The research questions Q-RENDER, Q-UI and Q-ACCESS are prepared, not run; their answers shape slices 3 and 4.

## How collected

Read-only scout by a crewmate agent over the old Syncloom repository (worktree at `3efcd35`), the old Holus Social API clone (README and layout), Main's records, the Fleet OpenSpec baseline and the invoz.v2 precedent. The old Python test suite was run once, read-only, with its summary and JUnit result kept here. No network access and no edits outside the task folder. The report is the crewmate's own output, copied without edits.

## Status

Delivered 2026-10-29. Complete: every open point it lists was decided in decision 0001 or is a pending study in the index (Q7, Q-UI, Q-ACCESS).

## Decision

[0001 syncloom.v2 is rebuilt in Rust as a separate repository, with decisions D1-D7](../../decisions/0001-syncloom-v2-rebuild-decisions.md).

## Files

Verbatim copies; the repository never edits them. Origins are paths on the machine that ran the study.

| File | Origin | sha256 |
|---|---|---|
| `report.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-rust-plan-20261029/report.md` | `9a2232f84cacddc0cf22656b1eb034c16e06730abda519199f738d906bdec415` |
| `pytest-summary.txt` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-rust-plan-20261029/pytest-summary.txt` | `d880d945343d7f80e35c4d7c5d38ed5d77d010e870b9a1ba8277dcc516dfbacd` |
| `pytest-junit.xml` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-rust-plan-20261029/pytest-junit.xml` | `b39cbd7e047363b1cf56a763e8108a362a7864be976d0b839a1d2d9e6834f6d7` |
