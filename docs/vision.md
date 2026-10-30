# Syncloom vision

Owner: the captain (Syncloom's founder)
Last reviewed: 2026-10-30 (captain: "yes to all", recorded in [decision 0001](decisions/0001-syncloom-v2-rebuild-decisions.md))

*For anyone deciding what Syncloom should and should not become: its intent, outcomes, priorities and non-goals. How it is built goes in the OpenSpec specs and each change's `design.md`; how to run it goes in the README. Every statement carries one label:*

- **[A] Agreed** - decided, in the captain's own words or by the captain's "yes to all" of 2026-10-30 on the recommended text (decision 0001 holds the words and the decisions D1-D7).
- **[O] Open** - not decided yet.

*Requirements belong in the OpenSpec specs, not here.*

## 1. What Syncloom is and who it is for

1.1 **[A]** Syncloom is the thought-to-content studio for a solo founder's content portfolio: one thought from a person or an online source becomes platform-native text, images and carousels, reviewed, then published, then learned from. (Existing vision, formerly "Holus".)

1.2 **[A]** syncloom.v2 is rebuilt in Rust, all first-party code, on the sound parts of the existing foundation; the old specifications are not carried over as they are ("we have a good foundation but specs are trash ... build it with rust all of it").

1.3 **[A]** One operator: the founder. No multi-tenant accounts, user roles or billing.

1.4 **[A]** It is a studio first: the founder's own thought is the starting point and the founder decides what goes out. The autonomous strategist with 40-plus agents is deferred until real performance data exists (D5).

## 2. The core loop

2.1 **[A]** Thought in, reviewed content out, explicit publish:
1. The founder gives one raw thought (text first; a link or an attachment later). The thought is the source, not the final copy.
2. Syncloom plans a content set for the chosen channels; LinkedIn first.
3. It drafts each piece for its platform and, later, renders its images and carousel.
4. The founder reviews previews, edits, and approves one exact version.
5. Publishing or scheduling is a separate, explicit action, never silent.
6. Later, results come back and inform the next drafts.

2.2 **[A]** Human review is the gate. An edit after approval voids the approval; only the approved version can be published.

2.3 **[A]** Drafts are written by a real model from the founder's thought, in the founder's voice, and name only what the thought names. Nothing in the product claims work that did not happen (the old fixed-template drafts and agent labels are retired).

2.4 **[A]** Each platform's limits and formats have one owner, so a draft is never checked against the wrong limit (the old app checked X and Threads against LinkedIn's 3000 characters).

## 3. Publishing and the social platforms

3.1 **[A]** Syncloom owns its own social platform integrations, rebuilt in Rust inside syncloom.v2 (D2, changed by the captain's words of 2026-10-30: "also the integrations wtih al lthe social media apis to make deep research onthem too"). The old Python Holus Social API, with its Bluesky, Facebook, Instagram, LinkedIn, Threads, X and YouTube adapters, is the reference for what each adapter did; it is not called from syncloom.v2 and is not rewritten as a separate service.

3.2 **[A]** Each platform integration is built only after deep research on that platform's current official API: what a single-user app may do, auth and token lifetimes, app review, rate limits and paid tiers, terms on automation and AI-generated content, breaking changes, test modes, and maintained Rust crates. The research questions are prepared and recorded in [`docs/research/2026-10-30-social-api-research-questions/`](research/2026-10-30-social-api-research-questions/README.md); each answer is saved in `docs/research/` before its integration is specified.

3.3 **[A]** The integrations also serve research: reading platforms for content research happens only through what the official APIs allow, settled by the same research before anything is built.

3.4 **[A]** Syncloom reports what actually happened: "accepted" is never shown as "published". Publishing stays contained until the founder explicitly enables it for a platform, a repeated publish request never posts twice, and a real post to a real account needs the captain's word for that exact action.

## 4. Where the founder uses it

4.1 **[A]** A MacBook during work, and sometimes the phone to look at previews.

4.2 **[A]** The operator studio is a browser page served by the Rust core on the laptop, phone previews included (D1). How the phone reaches it safely is decided by the prepared research question Q-ACCESS before that path is built; until then the server listens only on the laptop itself. A native desktop shell can be added on the same core if ever needed.

4.3 **[A]** The public portfolio demo (holus.camilomartinez.co) stays on the old app; syncloom.v2 does not rebuild it now (D3).

## 5. Quality priorities

5.1 **[A]** One core, thin surfaces: a Rust core library holds the rules; the loopback HTTP API, the command line and the browser page call the same core and return the same errors.

5.2 **[A]** Truthful records: every state change is recorded with ids and hashes, not the thought text; records survive a restart and are never half-written.

5.3 **[A]** Safe by default: the server listens only on the laptop itself (127.0.0.1), refuses other hosts, sends no cross-origin headers, and keeps keys only in the environment, never in logs.

5.4 **[A]** Tests leave the repository unchanged (the old tests wrote into `data/`).

5.5 **[A]** A fix counts as working only after a real end-to-end run shows the outcome the founder would see.

5.6 **[A]** Every model call and run is traced in the local Langfuse, with content kept out of the traces.

5.7 **[A]** Model spend for drafting stays within USD 5 a day without further approval, using Claude Sonnet (D4).

## 6. The build order

6.1 **[A]** One approved piece at a time, each its own OpenSpec change with a Request, an Approval, specs, a design and tasks, built with /code.

6.2 **[A]** Order:
1. Slice 1: one text thought, one model-written LinkedIn text draft, approval of an exact version, and a contained publish intent; loopback API plus command line (13 acceptance criteria, plan section 3).
2. Slice 2: the first platform integration in Rust, LinkedIn, after its research answer (question Q2) and the local-first credentials answer (Q8): truthful dispatch of an approved piece, tested against a fake of the platform's API before any real post.
3. Slice 3: images and carousels, after the rendering research question (Q7, formerly Q-RENDER) is answered.
4. Slice 4: the review screen, the browser page of 4.2.
5. Slice 5: more channels and their integrations (X, Threads, Bluesky, Facebook Pages, Instagram, YouTube as their research answers arrive), English/Spanish account routing, reusing the good behaviors from the unlanded spec 038 and its test cases.
6. Later: capture suggestions, link intake, Research Radar and platform reading for research (Q6).

6.3 **[A]** Records follow the invoz.v2 pattern (D7): OpenSpec specs are the only specification authority; decisions live in `docs/decisions/` and research in `docs/research/` with an index, linked both ways; no separate root `changes/` folder. The old app's capabilities and their migration state are in [`docs/migration.md`](migration.md).

6.4 **[A]** Landing works like invoz.v2 (D6): local-only, no remote; passing, reviewed, in-scope work lands locally without asking; destructive, irreversible, security-sensitive or failing work escalates; nothing is pushed without the captain's word.

6.5 **[A]** Once everything has moved from the old repository into syncloom.v2, the two are merged back into the original repository.

## 7. Out of scope for now

7.1 **[A]** The autonomous strategist, agent registry, prompt evolution, bandits, event bus and scheduler (deferred with D5).

7.2 **[A]** Video (Genpeli) and AI-generated images (Pilaster) as blockers: they stay later adapters.

7.3 **[A]** The Observatory fleet dashboard, the Telegram gate, the MCP server and the Company OS evaluation code (they stay in the old repository).

7.4 **[A]** Trading systems, a general chatbot, and custom model training.

7.5 **[O]** English-only first, or English and Spanish accounts from slice 1 (a slice-level question in the slice proposals; slice 1 is English-only).

## 8. How success is measured

8.1 **[A]** For the founder: a thought becomes a draft worth posting with little editing, approved and published in minutes, never posted by mistake.

8.2 **[O]** The numbers that define "worth posting" (edit distance, approval rate, time from thought to publish, engagement once analytics return) are set in the slice 1 and slice 5 proposals.
