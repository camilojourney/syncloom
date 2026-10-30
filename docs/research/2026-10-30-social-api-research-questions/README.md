# Research questions for the social platform integrations (and Q-RENDER)

Status: open

## Question

What do the seven adapters of the old Python Holus Social API actually do today (operations, auth, limits, known failures), and which exact, self-contained questions must each platform's API research answer before that integration is rebuilt in Rust inside syncloom.v2?

## Why asked

Decision D2 (decision 0001): the captain asked for the social media integrations to be rebuilt "to make deep research on them too". Each integration slice is specified only after its research answer is saved here (vision 3.2). This study is the inventory of the old adapters and the eight questions in caller form, ready to run through Phrona.

## What would change the answer

The questions are written from the old adapters and notes dated March 2026, treated as unverified seeds. A question is rewritten if its grouping proves too dense for one answer (the report flags Q1 and Q3), or if the captain trims Q5 (TikTok, Reddit, Mastodon). Running a question sends its text to an outside provider and needs the owner's approval of the exact rendered text first.

## How collected

Read-only scout by a crewmate agent over the old Social API repository (README, `src/platforms/*.py`, routes, platform docs, decisions, roadmap), the old studio repository (domain research index, the post skill, the architecture), the plan report and the vision draft. No web research, no `phrona` command, nothing sent. The report and the question files are the crewmate's own output, copied without edits.

## Status

Open: all eight questions are written. Q2, Q3, Q7 and Q8 ran through Phrona on 2026-10-30 and have their own study folders, cards and index rows ([q2](../2026-10-30-q2-linkedin-publishing/README.md), [q3](../2026-10-30-q3-meta-facebook-instagram/README.md), [q7](../2026-10-30-q7-render-images-carousels-rust/README.md), [q8](../2026-10-30-q8-local-first-auth-and-scheduling/README.md)); Q1 ran the same day and is not imported yet; Q4, Q5 and Q6 have not run. The "Pending studies" table of the index lists what remains.

## Decision

[0001 syncloom.v2 is rebuilt in Rust as a separate repository, with decisions D1-D7](../../decisions/0001-syncloom-v2-rebuild-decisions.md) (D2).

## Files

Verbatim copies; the repository never edits them. Origins are paths on the machine that ran the study.

| File | Origin | sha256 |
|---|---|---|
| `report.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/report.md` | `473c6f26942ebac3f9d335ff078c5616858f1fea708814eb4e88cb79250b3b57` |
| `questions/q1-x-threads-bluesky.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/questions/q1-x-threads-bluesky.md` | `285735365f261f18051e8a92c3b0aacca6577933a7871500151bbdb919ed80aa` |
| `questions/q2-linkedin.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/questions/q2-linkedin.md` | `dc08cbb4ca38f3720635068d934130b30488673bf7181ec463decc355ca8ef29` |
| `questions/q3-meta-facebook-instagram.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/questions/q3-meta-facebook-instagram.md` | `0376181b391f348548d182fc20f50a55e289f8b8dde58a1865dc82948b98c1d0` |
| `questions/q4-youtube.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/questions/q4-youtube.md` | `0361cdae3af233cfa1339582c306333cc90323c28071e0eddd2e25dd5b7c296e` |
| `questions/q5-tiktok-reddit-mastodon-scope.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/questions/q5-tiktok-reddit-mastodon-scope.md` | `36aaccf55bb61b8021a98ce3f94fd694c2222546b3621a35aefaead1e1c46c8d` |
| `questions/q6-reading-platforms-for-research.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/questions/q6-reading-platforms-for-research.md` | `f0600ba8d822367a44ad0bd5bec490c206f4d01866feb20f517034fff792d2ad` |
| `questions/q7-render-images-carousels-rust.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/questions/q7-render-images-carousels-rust.md` | `df615de9099ec8d95f838d07d31f18d05a1e14f4a4f000003b0b6bc6592f6f0c` |
| `questions/q8-local-first-auth-and-scheduling.md` | `/Users/camiloslaptop/github/firstmate/data/syncloom-v2-research-questions-20261030/questions/q8-local-first-auth-and-scheduling.md` | `3bd00183162a71cf2aeeb9345d5e812c8b2a7b4f098c01a1725d801635e06874` |
