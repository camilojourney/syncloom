# syncloom.v2 in Rust: evidence-backed build plan

Scout task `syncloom-v2-rust-plan-20261029`, 2026-10-29. Read-only. Nothing outside this folder was edited, nothing was pushed, and no research or network call was made.
Paths below are relative to the Syncloom repository at local main `3efcd35` (worktree `/Users/camiloslaptop/.treehouse/syncloom-b30920/2/syncloom`) unless they start with `/`. `FM` = `/Users/camiloslaptop/github/firstmate`.

## One-page summary

**What exists.** No `syncloom.v2` repository exists. The foundation is the Python repo `camilojourney/syncloom`. Its in-repo identity is still "Holus": `README.md:1`, and no file in the repo mentions Syncloom (`FM/data/intake-holus/report.md:73`). It holds 36,153 lines of `src/` and 34,546 lines of tests. **The suite passes: 1892 tests, 0 failures, 1 skipped** (`pytest-summary.txt`, `pytest-junit.xml`, this folder). The core loop is real and well guarded: thought, content set, revision-bound review, then a contained publish intent in a durable outbox. Three things are weak:
- **Draft generation is fake.** It is a deterministic template with copy hard-coded for one specific thought (`src/holus/agents/marketing/thought_pipeline.py:972-1003, 1036-1160`).
- **The Social API contract has drifted.** It still does at `3efcd35` (`src/holus/integrations/holus_social_api/client.py:137,238-240,254`; `src/holus/api/routes/content.py:717-723`).
- **No spec owns the shipped core loop.** Intake, capture, review, outbox and lineage have no spec (section 2.3). The spec index also disagrees with spec headers.

**Recommended path.**
- Create a new local repo `syncloom.v2` beside the old one, following the invoz.v2 pattern. Use stock OpenSpec with Fleet baseline 1.0.0 copied verbatim, plus a Syncloom context block. `openspec/specs/` is the only spec authority.
- Build one approved behavior at a time.
- **Slice 1 does not depend on the platform choice:** a Rust core library, a loopback HTTP API and a CLI. It covers one text thought, one model-drafted LinkedIn text draft, revision-bound approval and a contained publish intent. The existing `/post` agent workflow already uses an HTTP API on loopback (`.agents/skills/post/SKILL.md:100-146`). Any later UI uses the same core.
- Rendering (images and carousels) and the operator UI come after the matching captain decisions and one research answer.

**Outstanding captain decisions** (none blocks slice 1 except D4, which blocks only its live-model check):

| # | Decision | What the evidence settles | Options |
|---|---|---|---|
| D1 | Operator frontend: browser, native desktop, or both | Settled: a browser surface exists for the public demo (`ARCHITECTURE.md:53-58`). Phone review matters: `/post` must return "phone-openable preview links" (`.agents/skills/post/SKILL.md:21-23`), and the operator "occasionally checks on his phone" (`docs/target.md:9`). **Not settled:** the form of the operator studio itself. | browser only; desktop only (phone review then needs another path); both on one core |
| D2 | Does "Rust, all of it" include rewriting Holus Social API (Python FastAPI, 7 platform adapters, 258 tests)? | The Syncloom charter covers both repos (`FM/data/syncloom/brief.md:4`). Holus treats the API as an external boundary that owns posting (`ARCHITECTURE.md:43-46,168-186`). | keep it as an external HTTP boundary for now (recommended); rewrite later as its own v2; fold it into syncloom.v2 |
| D3 | Is the public portfolio demo (`holus.camilomartinez.co`, the `holus.generation.v1` contract, the Genpeli BFF seam) in v2 scope? | Today it is a Next.js site on Vercel (`observatory/frontend/vercel.json`), and the live deploy is stale (`FM/data/intake-holus/report.md:25-38`). | leave it on the old app; rebuild later; drop |
| D4 | Model provider and spend for draft generation | ADR 0002 and the vision say Claude-only (`docs/vision.md:182`). Generation calls over $5/day need approval (`AGENTS.md:186`). | Claude (which model); multi-provider; deterministic-only for now |
| D5 | Product identity for v2: founder-thought-first studio, or autonomous strategist with 44-45 agents | The studio is the later, consistent direction (`README.md:16-27`, `ARCHITECTURE.md:350-374`, `.agents/skills/post/SKILL.md:26-27`). `AGENTS.md:28-38,73` and specs 010/031/035 still describe the autonomous strategist. | studio first, defer the agent fleet (recommended); carry both |
| D6 | Landing authority for syncloom.v2 | Syncloom has `yolo off` and needs an explicit merge word (`FM/data/changes-folder-syncloom-20261024/dispatch-rationale.md:3`). The captain's standing "yes and yes" was given for invoz.v2 and phrona.v2 only (`FM/data/invoz-v2-20261029/captain-landing-answer.txt`). | same standing landing as invoz.v2; per-slice approval |
| D7 | Change records in v2: OpenSpec `openspec/changes/` only, or also a root `changes/` folder | The captain asked that Syncloom get `changes/` (`FM/data/changes-folder-syncloom-20261024/brief.md:5`), but the invoz.v2 precedent uses OpenSpec changes only. | OpenSpec changes only (recommended, one record system); both |

Research to prepare, not yet run (section 5): **Q-RENDER** (carousel and image rendering in Rust), needed before the visual slice whatever D1 says. **Q-UI** and **Q-ACCESS** apply only after D1.

---

## What I did

- Read the project records: `README.md`, `ARCHITECTURE.md`, `AGENTS.md`, `docs/{vision,target,roadmap,ux-state,lineage}.md`, `docs/decisions/*`, `specs/README.md` and every spec header, the key spec bodies (031, 035, and the unlanded 038), `specs/ux-content-studio.md` and `.agents/skills/post/SKILL.md`.
- Read the core source: the thought pipeline, content/capture routes, the Social API client and containment, the outbox, and the render engine.
- Read Main's records: `FM/data/syncloom/brief.md`, `FM/data/syncloom-evalloop-fable/` (notes, tool feedback, arbiter review), `FM/data/syncloom-editor-preservation-20261003/receipt.md`, `FM/data/syncloom-supervisor-closure-20261004/` (meta and handled inbox messages), `FM/data/changes-folder-syncloom-20261024/`, `FM/data/intake-holus/report.md` and `FM/data/intake-holus-api/report.md`.
- Read the Holus Social API clone at `/Users/camiloslaptop/.firstmate-homes/syncloom/projects/holus-social-api` (`6e9a37b`): README and layout only. Its tests were not run (it is not under `FM/projects/` as the brief assumed).
- Read the baseline: `/Users/camiloslaptop/.firstmate-homes/fleet-lab/data/shared-openspec-baseline-20261029/report.md`, plus `system/shared/openspec/baseline.yaml` and `docs/development.md` at Fleet `4501883843df70c50f0726120aca09387ca6c95d`.
- Read the invoz.v2 precedent: `FM/projects/invoz.v2/{README.md,openspec/config.yaml,openspec/specs,openspec/changes}` and `FM/data/invoz-v2-20261029/contract.md`.
- Ran the full Python suite once, read-only, with results in `pytest-summary.txt`. Offline `uv sync` failed because the playwright and mypy wheels are not cached, so I used the existing slot-1 `.venv` interpreter read-only with `PYTHONPATH` pointing at this worktree's `src`.
- Did not run the frontend tests: there is no `node_modules` and the task is offline. The last recorded result is 20 passed, a passing build, and E2E scenarios that passed but then hung (`FM/data/intake-holus/report.md:59-61`).

---

## 1. First user journey and target platforms

### 1.1 The intended first journey, from the evidence

**Thought in, reviewed content out, explicit publish:**

1. The founder supplies one raw thought: text or a URL (`ARCHITECTURE.md:125-128`). Later this extends to one attachment slot (`AGENTS.md:243`). The thought is the source, not final copy (`.agents/skills/post/SKILL.md:31`).
2. Syncloom normalizes it and plans a content set for the selected channels (`ARCHITECTURE.md:130-138`). The default is LinkedIn: text, carousel PDF, and an image where supported (`.agents/skills/post/SKILL.md:33-39`).
3. It generates platform drafts and renders visual assets (`ARCHITECTURE.md:140-142`).
4. The founder reviews previews: Content Studio plus direct PDF and image links that open on a phone (`.agents/skills/post/SKILL.md:21-23,72-84`). The review shows quality and platform-fit evidence (`ARCHITECTURE.md:144-148`).
5. The founder approves a specific revision. Publishing or scheduling is a separate, explicit action and is never silent (`.agents/skills/post/SKILL.md:26-27,44`; `AGENTS.md:241`).
6. Delivery goes through Holus Social API, which owns accounts and posting (`ARCHITECTURE.md:43-46`). Today it is contained unless a grant is set (`src/holus/integrations/holus_social_api/containment.py:16-32`).
7. Later, analytics are read back for learning (`ARCHITECTURE.md:156-158`).

The UX spec for the studio agrees: creation first, then thought, formats, assets, review and schedule (`specs/ux-content-studio.md:5-26`).

### 1.2 Target platforms

- **Social destinations:** LinkedIn first (`.agents/skills/post/SKILL.md:33`). The pipeline supports linkedin text/image/carousel, instagram image/carousel, threads, X thread and facebook (`src/holus/agents/marketing/thought_pipeline.py:50-59`). The vision's "13 platforms" (`docs/vision.md:39`) is not implemented.
- **Accounts:** an English and a Spanish brand handle (`thought_pipeline.py:29-32`).
- **Operator devices:** a MacBook during work and a phone sometimes (`docs/target.md:9`).
- **Hosting:** a Mac mini as infrastructure (`docs/vision.md:22`), and the public site on Vercel (`observatory/frontend/vercel.json`).

### 1.3 What the evidence already settles

- **Single operator; no multi-tenancy, auth roles or billing** (`docs/vision.md:178`; `docs/target.md:31`).
- **Human review is the default gate, and publishing is explicit** (`AGENTS.md:241`; `ARCHITECTURE.md:144-154`).
- **Holus Social API owns posting, accounts and analytics.** Analytics are not stored permanently (`ARCHITECTURE.md:43-49`; `AGENTS.md:239`).
- **Video (Genpeli) and AI images (Pilaster) are deferred adapters.** They must not block text, image and carousel (`README.md:29-32`; `ARCHITECTURE.md:192-204`).
- **Some browser surface exists:** the public demo (`ARCHITECTURE.md:51-72`). Phone access to previews matters (`.agents/skills/post/SKILL.md:21-23`).

### 1.4 Unanswered product choices

- **D1: desktop vs browser vs both.** Settled only to the extent of 1.3: a browser-reachable demo and phone previews exist today. Native desktop alone cannot serve phone previews without a second path.
- **Contradiction to resolve, not a new choice:** `docs/target.md:9,35` and `docs/vision.md:176` say the Observatory is read-only with no control. That premise was overtaken by the shipped Capture and Content Studio write paths (`AGENTS.md:243`; `specs/ux-content-studio.md:13-16`). Treat `docs/target.md` as stale.
- **D2, D3, D4, D5, D6, D7:** see the summary table.
- **Slice-level product questions** (for each slice's proposal "Open questions", not captain-level now):
  - English-only first, or EN/ES account routing? The routing exists with a weak heuristic (`thought_pipeline.py:40-53,1580-1616`).
  - Should capture "confirm" stay a one-click approve+publish? Today it does both (`src/holus/api/routes/capture.py:188-234`).
  - Should URL intake fetch pages in slice 1? It fetches today through a public-URL guard that checks every redirect (`thought_pipeline.py:504-517`; `src/holus/research/sources/base.py:115-123`).

---

## 2. Retain / rebuild / retire map, by capability

Legend:
- **Retain behavior:** the observable behavior is sound; reimplement it in Rust and port the scenarios as v2 spec scenarios.
- **Rebuild:** the need is real but the current behavior is wrong or weak.
- **Retire:** do not carry into v2 (the old repo keeps it).
- **Defer:** out of v2 until its product decision or prerequisite exists.

Test counts come from this run's `pytest-junit.xml`, grouped by test module.

### 2.1 The studio loop (first journey)

| Capability | What works today (evidence) | Verdict for v2 |
|---|---|---|
| Thought intake (text/URL, provenance) | `POST /api/v1/content/from-thought` (`src/holus/api/routes/content.py:381`). Source type, URL and raw input are kept (`ARCHITECTURE.md:275`). Thoughts shorter than 8 characters are refused (`thought_pipeline.py:464-466`). Tests: thought_pipeline 14, content_routes 32, intake simulation 10. | **Retain behavior.** URL fetching (today behind `validate_public_http_url`, `src/holus/research/sources/base.py:115-123`) gets its own slice and security review. |
| Draft generation | Deterministic. One family of thoughts (mentions "workflow" plus 3 of claude/codex/cursor/deepseek/agy/skills) gets hard-coded paragraphs (`thought_pipeline.py:972-1003, 1007-1010, 1036-1160`). Any other thought gets its first sentence inside fixed filler copy that ends "That is the product I want Holus to become" (`thought_pipeline.py:1062-1070`). Evalloop baseline dev match rates: thesis 6/19, copy relevance 8/18, visual copy 6/17 (`FM/data/syncloom-evalloop-fable/notes/arbiter-review.md:17-19`). The agent names on records are metadata only; no model runs (`thought_pipeline.py:432-433`), yet records say e.g. "drafted LinkedIn post in Juan's voice" (`thought_pipeline.py:61-62`). | **Rebuild** as model-backed generation behind a provider trait. **Retire** the hard-coded copy and agent labels that claim work that never happened. |
| Platform rules (limits, shapes) | Three disagreeing tables (`arbiter-review.md:20-21`). Fit uses 3000 characters for every platform, X (280) and Threads (500) included (`thought_pipeline.py:1659-1667` vs `client.py:30-37`). | **Rebuild** with one owner capability. |
| Language/account routing | Substring markers such as `" sistema"` and `"flujo"` (`thought_pipeline.py:40-53`). Dev match 9/21 (`arbiter-review.md:17`). | **Rebuild.** Spec 038's behavior statements are sound inputs (section 2.3). |
| Review workflow | Status transition table (`content.py:507-517`). Approval is bound to a content revision via `review_decision_id`; a stale revision returns 409 (`content.py:107-118,523-526`). Tests: content_routes 32, review 13, e2e_publish 17. | **Retain behavior.** |
| Dispatch outbox and containment | One intent per (operation, piece, revision, scheduled_at), reserved atomically under a lock before any side effect (`src/holus/lineage/outbox.py:60-104`). Fails closed by default; `HOLUS_PERSONAL_DELIVERY_GRANT` opts in (`containment.py:16-32`). Retries do not duplicate (`FM/data/intake-holus/report.md:124-128`). Tests: outbox 3, capture_routes 7, personal_delivery_grant 3, social contract 7. | **Retain behavior.** |
| Social API client | Drift is still present at `3efcd35`: unversioned `/api/publish/{id}` (`client.py:254`); `schedule_id` where the API returns `scheduled_id` (`client.py:137`); fields the API drops silently (`client.py:238-240`); the item is marked `published` when the API only accepted the job (`content.py:717-723`). Cause and API side: `FM/data/intake-holus-api/report.md:17-22`. The API's real contract: `POST /api/v1/publish` returns accepted and async; status lives at `GET /api/v1/posts/{job_id}` (Social API `README.md`, "API behavior"). | **Rebuild** against the API's actual contract, keeping its terminal status and result mode truthful (`FM/data/intake-holus-api/report.md:74-82` is a ready acceptance list). |
| Capture (suggest/preview/confirm) | Works on text plus attachment metadata. Binary upload is blocked by design (`docs/decisions/011-capture-binary-intake-contract-gap.md:6-25`). Confirm does approve and publish in one call (`capture.py:188-234`). Tests: capture_routes 7, suggest 5. | **Rebuild later**, after D1. The confirm semantics need a product answer. |
| Lineage (provenance JSONL) | Append-only and privacy-safe. Its failure never breaks the artifact (`docs/lineage.md:1-30`; `thought_pipeline.py:484-489`). Tests: store 5, pipeline 1, routes 1. | **Retain behavior**, likely simpler: in v2 the review history can live in the same store. |
| Visual rendering (PNG/PDF) | HTML/CSS templates rendered by headless Chromium through Playwright (`src/holus/visual/engine.py:1-60,119-165,232-290`). Themes, layouts and SVG charts (`docs/roadmap.md:60-71`). Tests: carousel_rendering 49, spec_converter 60, charts 22, engine 18. | **Rebuild after Q-RENDER.** Keep the design system (templates, palettes, fonts) as input material. |
| Visual judge and quality score | `quality_score.py` (672 lines, 86 tests). Ordinary visuals carry REDO noise (spec 038 problem statement, `git show 16890bd:specs/038-...md` lines 22-28). | **Rebuild later** as an evaluated part, not slice 1. |

### 2.2 Everything else

| Capability | Evidence | Verdict |
|---|---|---|
| Research Radar | Spec 037 is implemented. Tests: radar 10, sources 9, api 3, routes 5. Approval routes a candidate into the studio without publishing (`README.md:43-57`). | **Defer.** Retain the behavior when an intake slice needs it. |
| Observatory dashboard (agents, trajectories, evaluations, knowledge, health) | API tests 66. Next.js 16 frontend (`observatory/frontend/package.json:14-21`). Built around the agent fleet (`docs/target.md:11`). | **Defer or retire with D5.** The studio needs review screens, not a fleet dashboard. |
| Public demo and `holus.generation.v1` contract | `src/holus/generation/*` and the frontend lib (`ARCHITECTURE.md:60-72`). Tests 7. Live site is stale (`FM/data/intake-holus/report.md:25-38`). | **D3.** |
| Agent registry, 3-layer prompt loader, 45 prompt files | Tests: registry 70, prompt_loader 16. The counts disagree: 32 (`docs/vision.md:62`), 38 (`specs/ux-content-studio.md:16`), 44 (`AGENTS.md:73`), and 45 `.md` files under `agentic/agents/`. | **Retire as a runtime.** Keep the KERNEL prompts as reference text for writing v2 generation prompts. |
| Autonomous marketing agent (LangGraph ReAct), auto-publish, orchestrator, specialist dispatch, Thompson bandit, prompt evolution, DSPy bridge, learning loop, diagnostician | Large and tested (marketing_agent 51, auto_publish 46, base_agent 49, specialist_dispatch 17, strategy_bandit 14). Not on the studio path. The old plan itself says not to build Phase 2/3 before Phase 1 content is good (`ARCHITECTURE.md:373-374`). | **Defer (D5).** Revisit once analytics data exists. |
| Redis event bus, mem0, launchd scheduling, kill switch | `src/holus/core/events.py`, `kill_switch.py`, `memory/mem0_client.py`. Tests: events 42, kill_switch 5, launchd 5. | **Defer.** A stop control comes back only with autonomy. |
| Genpeli video and Pilaster image clients | Deferred by the old architecture itself (`README.md:31-32`). Tests: video 47, image 40. | **Retire from v2** until an adapter slice. |
| Company OS evaluation code | `src/holus/evaluation/company_os_contract.py` (1378 lines) and tests 67. This is skill-evaluation infrastructure (`AGENTS.md`, "Company OS Domain Skills"), not product. | **Retire from the product repo.** It stays in the old repo. |
| Telegram gate | Never wired (`docs/AUDIT-2026-03-23.md:56,145`). | **Retire.** |
| MCP server | `src/holus/mcp` (8 tests). The real integration is direct HTTP (`FM/data/intake-holus-api/report.md:15`). | **Defer.** |
| Repo hygiene practices | 548 committed `node_modules` files under `docs/reference` (`git ls-files docs/reference \| grep -c node_modules`). 89 tracked `data/` files, including runtime queue YAML and `data/personal-context.json`. **Tests are not hermetic:** this run created `data/lineage/events.jsonl`, `data/lineage/.lineage.lock` and `data/.niche-research-state.json` in the worktree (`pytest-summary.txt`). | **Retire these practices.** v2 tests must leave the tree unchanged. |

### 2.3 Old specifications: which are wrong, stale or sound

**Missing owners.** Nothing in `specs/` owns the shipped core loop. A grep for `from-thought`, `capture/`, `outbox`, `lineage`, `holus.generation`, `PERSONAL_DELIVERY` and `expected_revision` finds only spec 037 (Research Radar handing off to from-thought). The de facto owners are prose and code: `ARCHITECTURE.md:120-159` (loop), `ARCHITECTURE.md:51-72` (public generation boundary), `docs/lineage.md` (lineage and outbox) and `docs/decisions/011-...` (capture binary gap).

**Index vs header disagreements.**

| Spec | `specs/README.md` says | Header says |
|---|---|---|
| 028 | Implemented (line 64) | `planned` (`028:3`) |
| 030 | Implemented (line 71) | `planned` (`030:3`) |
| 032 | Implemented (line 78) | `Not Started` (`032:3`) |
| 033 | Implemented (line 79) | `planned` (`033:3`) |

Spec 036 says `Not Started`, but the diagnostician exists (`FM/data/intake-holus/report.md:101`). The index's "how to write specs" link points outside the repo (`specs/README.md:7`).

| Spec | Judgment | Why |
|---|---|---|
| 001 core infrastructure | stale | Redis bus, kill switch and config for the agent fleet; partial. |
| 003, 011 | already deprecated | `specs/README.md:18,36`. |
| 009 autonomous build system | retire | Describes the development process, not product behavior. |
| 010 marketing agent, 027 resilient loop, 012 knowledge, 013 scheduling, 030 registry, 036 diagnostician | stale for v2 (D5) | The autonomous strategist path. |
| 014 Genpeli, 015 Pilaster | stale | The architecture now says deferred (`ARCHITECTURE.md:192-237`). |
| 016 social media v2 | **wrong** | Describes old MCP tools and says there is no wrapper, but a direct HTTP client exists (`FM/data/intake-holus-api/report.md:51`). |
| 017 authority engine | partly sound | Brand voice config is useful evidence for generation prompts. |
| 028/029 Observatory | wrong premise | "Read-only" was contradicted by capture and approve. 029 is partial. |
| 031, 035 LinkedIn pipelines | wrong journey | Analytics-driven agent decides and Telegram approval (`specs/031-...md:19-26`; `specs/035-...md:27-33`). The current journey is thought-first (section 1.1). |
| 032, 033, 034 | not implemented or drafts | Defer. |
| 037 research radar | sound | Implemented and tested; migrate when needed. |
| `ux-content-studio.md` | sound direction | Use it as input for the UI slice. |
| **038 (unlanded, branch `fm/syncloom-evalloop-fable` at `16890bd`)** | **partly sound** | Its user-visible requirements are good: correct account, no mid-word cuts, labels removed, per-platform and per-segment limits, drafts name only products the thought names (`git show 16890bd:specs/038-...md` lines 32-37, 136-162). Its algorithms (marker words, 200/72-character cuts) belong to the deterministic composer and should not be copied. The 7 evalloop components' dev cases (`evals/parts/*/cases/dev.jsonl` at `4855b13`) are reusable behavior fixtures. |

---

## 3. Smallest end-to-end first slice

**It does not depend on D1.** The slice is a Rust core library plus two thin adapters: a loopback HTTP API and a CLI. The HTTP API is justified by an existing consumer, not by a browser assumption: the `/post` agent workflow already drives the studio over loopback HTTP (`.agents/skills/post/SKILL.md:100-146`). A browser UI, a Tauri shell or a native toolkit can later call the same core.

**It depends on D4 only for its one live-model acceptance check.** invoz.v2 did the same: it landed with a fake provider, and the live Groq check stayed pending (`FM/projects/invoz.v2` commit `860cdfd`).

**Scope.** One text thought, then one `linkedin_text` draft generated through a provider trait, then revision-bound approval, then a contained publish intent.

Out of scope:
- URL fetching, other channels, images and carousels.
- Social API HTTP (that is slice 2).
- Capture suggestions, the UI, analytics.
- EN/ES account routing (an open question in the proposal).

**Acceptance criteria.** Each one is to become a SHALL requirement with scenarios in the slice's change.

1. **Loopback only.** The server binds only `127.0.0.1`. Any other bind address is a startup error. A request whose Host header is not `127.0.0.1:<port>` gets 403 `host_not_allowed`. No CORS headers are sent. This copies the invoz.v2 rules (`FM/data/invoz-v2-20261029/contract.md:12-16`). The current Python API has no auth and sends permissive dev CORS (`src/holus/api/app.py:76-80`).
2. **Create a draft.** `POST /v1/thoughts {text, channels:["linkedin_text"]}` returns 201 `{group_id, pieces:[{piece_id, channel, text, revision, status:"pending_review", source:{type:"text", raw_input}}]}`.
3. **Input errors.** Fewer than 8 non-space characters gives 422 `thought_too_short` (retained behavior, `thought_pipeline.py:464-466`). An unknown channel gives 422 `unsupported_channel` (retained, `capture.py:154-159`). Every error uses one envelope with a fixed message per code and never echoes request content (invoz pattern, `contract.md:22`).
4. **Generation boundary.** The draft comes from the `ModelProvider` trait. Tests use a fake provider and cover success, empty output, timeout and provider error. A failure returns 502 `provider_error` or 504 `provider_timeout` and persists nothing.
5. **Platform fit.** It is computed by the single platform-rules owner. A LinkedIn draft over 3000 characters is marked `REVIEW`, not `PASS`.
6. **Durable records.** Records survive a restart. Writes are atomic: a crash mid-write leaves the old record or the new one, never a torn file.
7. **Review transitions.** `PATCH /v1/pieces/{id} {status, expected_revision}` follows the retained transition table (`content.py:507-517`). An invalid transition gives 409 `invalid_status_transition`. A stale revision gives 409 `revision_conflict`. Editing the text creates a new revision and voids any approval.
8. **Publish rules.** `POST /v1/pieces/{id}/publish {expected_revision, dry_run}`:
   - unapproved gives 409 `approval_required`;
   - `dry_run` returns the payload and records nothing;
   - otherwise, exactly one outbox intent is reserved per (piece, revision, operation), marked `contained`, with zero outbound HTTP;
   - a repeated call returns the same request id (retained, `outbox.py:68-104`; `FM/data/intake-holus/report.md:124-128`).
9. **Provenance.** Each state change appends a provenance event that holds ids and hashes, not the thought text (retained intent, `docs/lineage.md:1-6`).
10. **CLI parity.** The CLI's `thought`, `show`, `approve` and `publish --dry-run` call the same core functions with the same errors.
11. **Hermetic tests.** `cargo test` uses temp directories, and `git status --porcelain --ignored` is unchanged after a run. This fixes the pollution observed in section 2.2.
12. **Checks.** `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` and `openspec validate --all --strict --no-interactive` all pass. The proposal follows baseline 1.0.0. Task 1.1 loads `/code` and records test-first evidence (`baseline.yaml:29-31`).
13. **Live check (manual, gated on D4).** One real thought produces a draft through the chosen provider, with the key read only from the environment and never logged.

**Slices after it**, each its own OpenSpec change:
- **Slice 2: truthful Social API dispatch.** Run it against a locally started Social API in sandbox mode, using `FM/data/intake-holus-api/report.md:74-82` as acceptance. Gated on D2 only for ownership, not for the client.
- **Slice 3: visual assets.** After Q-RENDER.
- **Slice 4: operator review UI.** After D1, plus Q-UI and Q-ACCESS if they apply.
- **Slice 5: more channels and EN/ES routing,** using the 038 behaviors and the ported evalloop cases.
- **Then:** capture suggestions, URL intake, Research Radar.

---

## 4. Rust architecture alternatives per platform option

**Common core, regardless of D1.** One Cargo workspace:
- `syncloom-core`: domain types, review state machine, platform rules, outbox rules; no I/O.
- Ports: `ModelProvider`, `Store`, `SocialApi`, `Renderer`, `Clock`.
- Adapters: store, provider over HTTP, Social API client, renderer (later).
- `syncloom-server`: Axum, loopback.
- `syncloom-cli`.
- `contracts/`: JSON fixtures both sides test against, as invoz.v2 does (`FM/projects/invoz.v2/README.md:5-10`).

**Local evidence for this core.** invoz.v2 already uses and verifies Axum 0.8, Tokio, reqwest with rustls, tracing and tower-http in this fleet (`FM/projects/invoz.v2/backend/Cargo.toml`, `[dependencies]`). Model calls can follow its raw-reqwest provider pattern; I did not check for an official Anthropic Rust SDK (no network).

**Storage** is a slice-1 design decision, not a captain call. SQLite would turn revision checks and outbox uniqueness into transactions and constraints. Today the code uses fcntl locks and scans every queue file on each request (`outbox.py:60-104`; `content.py:494-506`). Per-record YAML files keep today's easy inspection.

**Toolchain fact.** No `cargo` or `rustc` is on this shell's PATH. `cargo-1.98.1` exists in `/nix/store`, and the crate cache holds `axum-0.8.9`, `tokio-1.54.1`, `tauri-2.9.3`, `wry-0.53.5` and `tao-0.34.5`, so Tauri has been compiled on this machine before. v2 must pin its toolchain (`rust-toolchain.toml`) and state how `cargo` is reached.

The UI rows below are compared as options, not picked. Limitations marked **(local)** come from repo evidence. Those marked **(verify)** are general knowledge to confirm through Q-UI, not established facts.

| Platform option | Rust approach | Strengths | Concrete limitations |
|---|---|---|---|
| **Browser only** | A1: Axum serves HTML rendered in Rust (Askama, Maud or minijinja templates), with little or no client script | One process; reachable from a phone; no first-party JS | **(local)** Phone access means the operator API leaves loopback, which needs auth that does not exist today (`src/holus/api/app.py:76-80`). **(local)** The public demo is Vercel/Next.js today (`vercel.json`); a Rust server needs a different host (Q-ACCESS). **(verify)** Rich interactions (inline editing, slide-by-slide preview) need hand-written enhancement or a third-party script. |
| | A2: Rust compiled to WASM for the UI (for example Leptos or Dioxus web) on the same Axum core | Rich client UI written in Rust; shared types with the server | **(local)** Today's UI leans on React component libraries (Tremor, Recharts, `package.json:15-21`) with no Rust equivalent known locally. **(verify)** Framework API churn, WASM bundle size, compile times, accessibility support. Same phone/auth/hosting limits as A1. |
| **Native desktop only** | B1: Tauri 2 (Rust core plus system webview; UI from A1 or A2) | Native packaging, filesystem, tray; the UI is reusable in a browser later | **(local)** No phone path unless a server is also run: `/post` requires phone-openable links (`SKILL.md:21-23`). **(verify)** Webview differences per OS (WKWebView on macOS); signing and notarization for distribution; Tauri mobile maturity. **(local)** Spec 028 noted that filesystem reads work in Tauri but not in browsers (`specs/028-observatory-api.md:389`). |
| | B2: Pure-Rust native toolkit (for example egui, Iced, Slint, GPUI) | Everything in Rust, no webview | **(verify)** Rich text editing, PDF/PNG preview, accessibility, and licensing (for example Slint's licence terms) vary by toolkit. **(local)** Previewing carousels that are rendered as HTML would still need a separate renderer (section 2.1). No phone path. |
| **Both** | C1: one core plus Axum; a browser UI (A1/A2) and a Tauri shell loading the same UI | Most reuse; phone and desktop | Carries the limits of A plus B1: two packaging targets and auth for remote access. |
| | C2: one cross-target Rust UI framework (for example Dioxus web, desktop and mobile) | One UI codebase | **(verify)** Maturity of each target; same auth and hosting needs for phone access. |

**Rendering cuts across every option** and needs Q-RENDER:
- **R1:** keep the HTML/CSS templates and drive Chromium from Rust over CDP. This keeps visual parity, but a Chromium binary must ship or be located; today Playwright installs it (`AGENTS.md`, `just install` note).
- **R2:** Typst for PDF and PNG layout, in pure Rust. This means rewriting the design system.
- **R3:** SVG rasterization (resvg/tiny-skia) for single images.

A desktop webview might also render to PDF, but that is unverified.

---

## 5. Consequential evidence gaps, as Phrona-form questions (prepared, not run)

Public context for every question, with no private source or data:

> A single-operator, local-first content studio written entirely in Rust turns one short written thought into platform drafts, a 1080x1350 image, and a multi-page carousel PDF for LinkedIn document posts. The founder reviews and approves each revision before an explicit publish through a separate HTTP publishing service. A macOS laptop is the main device; previews must also open on a phone. Today's renderer is Python plus Playwright headless Chromium, turning HTML/CSS templates (gradients, web fonts, inline SVG charts) into PNG and per-slide PDFs merged into one document.

Rejected as not consequential:
- **Model access from Rust:** raw HTTP is already proven locally in invoz.v2.
- **LinkedIn carousel limits:** existing local research covers them (`docs/research/domain/carousel.md`, `linkedin-content-format-engagement.md`), refreshed at the carousel slice if stale.
- **Social API rewrite:** a captain decision (D2), not research.

**Q-RENDER (needed before slice 3, whatever D1 says).**
- **Decision:** how syncloom.v2 renders images and carousel PDFs.
- **Current evidence:** the Playwright/Chromium pipeline works, with 49 rendering tests (`src/holus/visual/engine.py`). No Rust rendering evidence exists in the fleet.
- **Alternatives:** (R1) drive headless Chromium from Rust over CDP with the existing HTML/CSS templates; (R2) Typst documents compiled to PDF and PNG; (R3) SVG templates rasterized with resvg/tiny-skia, with PDF from a Rust PDF library; (R4) a desktop webview's own PDF/PNG output where a desktop shell exists.
- **Constraints:** Rust for first-party code; a macOS host; deterministic output for snapshot tests; web-font and gradient fidelity; multi-page PDF; no paid service.
- **Question:** For producing 1080x1350 PNG cards and multi-page PDF carousels from designer templates in a Rust application, how do Chromium-over-CDP crates, Typst and resvg-based pipelines compare on visual fidelity (fonts, gradients, SVG), deterministic byte- or pixel-stable output, packaging and binary size on macOS, render latency, and maintenance activity, with published examples or benchmarks?
- **What would change the choice:** if Typst or resvg reaches pixel-stable output with acceptable typography, R1's Chromium dependency is dropped. If neither handles the existing gradient and font design, R1 stays and Chromium packaging becomes a slice-3 task.

**Q-UI (conditional: run only after D1, scoped to the chosen option).**
- **Decision:** which Rust UI approach implements the review studio for the platform the captain chose.
- **Current evidence:** none in the fleet beyond cached Tauri/wry crates. Today's UI is Next.js with React component libraries (`observatory/frontend/package.json`).
- **Alternatives:**
  - for browser: server-rendered Axum templates vs Leptos vs Dioxus web;
  - for desktop: Tauri 2 vs egui vs Iced vs Slint vs GPUI;
  - for both: Tauri plus a web UI vs Dioxus multi-target.
- **Constraints:** Rust for first-party code; a small single-operator app; needs inline text editing, image and PDF preview, accessible keyboard use, and dark mode; must stay maintainable by AI coding agents with strong tests.
- **Question:** For that platform, compare the named Rust UI approaches on stability of public APIs over the last two years, rich text editing and image/PDF preview support, accessibility, testing (component and end-to-end), build times, licensing, and how much generated-code churn agents face.
- **What would change the choice:** a framework without workable text editing or PDF preview drops out. If API churn is high for all WASM frameworks, server-rendered templates win for browser.

**Q-ACCESS (conditional: only if D1 includes a phone or browser path beyond loopback).**
- **Decision:** how the phone reaches previews and approval safely.
- **Current evidence:** the API has no auth today (`src/holus/api/app.py:76-80`). The `/post` skill forbids public tunnels without approval (`.agents/skills/post/SKILL.md:128-129`). The Mac mini is the intended host (`docs/vision.md:22`).
- **Alternatives:** private overlay network to the Mac (for example Tailscale); an authenticated reverse tunnel; a hosted Rust service with login; read-only signed preview links plus approval only on the laptop.
- **Constraints:** single user; no multi-tenant auth; publishing stays an explicit, revision-bound action; no secrets in URLs or logs.
- **Question:** For a single-user Rust web app on a home Mac that must let one person view previews and approve posts from a phone, compare overlay networks, authenticated tunnels, hosted deployment and signed read-only links on attack surface, setup and maintenance, latency, cost, and the auth each still requires.
- **What would change the choice:** if signed read-only links are judged enough, the phone never approves and the server stays loopback, with approval on the laptop only.

---

## 6. Spec and config migration plan on the approved baseline

### 6.1 Repository and identities

- **New local repository** `syncloom.v2` beside the preserved old repo, with no remote until the captain says so. invoz.v2 is local-only too: `git -C FM/projects/invoz.v2 remote -v` prints nothing.
- **Merge-back:** later, merge back into the original, as the captain said for Invoz (`FM/data/invoz-v2-20261029/captain-landing-answer.txt`).
- **Display identity:** "Syncloom". Keep the old repo's "Holus" identity untouched. Rename migration is out of scope (`FM/data/syncloom-supervisor-closure-20261004/syncloom.inbox/handled/053.msg`).

### 6.2 OpenSpec configuration

1. Run `openspec init --tools <tools>` with installed OpenSpec 1.15.0 (`/opt/homebrew/bin/openspec`), keeping stock `spec-driven`.
2. Copy `schema`, the common `context` lines, `rules` and `operations` **verbatim** from Fleet `system/shared/openspec/baseline.yaml` 1.0.0 at `4501883843df70c50f0726120aca09387ca6c95d` (procedure: `docs/development.md:33-56` there).
3. **Do not copy invoz.v2's config.** It predates the baseline and lacks several rules: the "including unmigrated project owners" proposal rule, the design security rule, the tasks test-first rule and the apply guidance; its archive text also differs (compare `FM/projects/invoz.v2/openspec/config.yaml:12-34` with `baseline.yaml:11-38`).
4. Record the baseline version and commit in `docs/development.md` of syncloom.v2.

**Project differences, as appended `context: |` lines only** (`docs/development.md:58-70` in Fleet):
- **Language:** Rust, with the toolchain pinned in `rust-toolchain.toml` and the way `cargo` is reached on this machine.
- **Layout:** the workspace crates from section 4, `contracts/`, `openspec/`.
- **Checks:** `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` plus `openspec validate --all --strict --no-interactive`.
- **Spec migration state:** "`openspec/specs/` is the only specification authority for syncloom.v2. Unmigrated owners for the old app are listed in `docs/migration.md`; read the overlapping ones before naming a capability."
- **Change concerns:** model calls and spend (`AGENTS.md:186`), external publishing stays contained and explicit (`AGENTS.md:241`), personal content and brand data, outbound URL fetches.
- **No baseline rule is changed.** invoz.v2's `scripts/check-change-shape.py` is Python. Either run it from its owner location or port it as a `cargo xtask`, to honor "Rust for all first-party implementation".

**Capability identities to adopt** (proposed; each proposal still runs `openspec list --specs` and requests approval):

| Capability | Owns |
|---|---|
| `studio-api` | loopback transport, Host rule, error envelope, request ids |
| `thought-drafting` | intake plus model drafting |
| `platform-rules` | channel limits and shapes, one owner |
| `draft-review` | states, revisions, edits |
| `dispatch-outbox` | contained and granted intents, idempotency |
| `social-api-client` | slice 2 |
| `provenance-log` | provenance events |
| `visual-assets` | slice 3 |
| `operator-ui` | after D1 |
| `capture-suggest`, `research-radar` | later |

### 6.3 One specification authority and migration of old specs

- **Do not edit or delete old specs now.** The old app is preserved and still runs.
- **`docs/migration.md` in syncloom.v2 is the ledger.** One row per old capability, pinned to old commit `3efcd35`, with columns:
  - old owner: spec, doc or code path;
  - verdict from section 2;
  - v2 capability, or retirement reason and approval;
  - state: unmigrated, migrated or retired.
- **How a capability migrates.** The OpenSpec change that rebuilds the capability names the old owner it read in its proposal (the baseline already requires reading unmigrated owners, `baseline.yaml:14`). Its design records which old behaviors were kept or changed, and why. The ledger row flips to `migrated` in the same branch.
- **How a capability retires.** It gets a ledger row with the captain's or reviewer's approval, not a spec delta. Retirement never deletes old code or data.
- **The old repo's specs are retired only once,** in one deliberate change made in the original repo at merge-back time, citing the ledger.
- **Unlanded spec 038 and the evalloop cases** stay on their branch. Their sound behaviors enter v2 through slice 5's proposal, with the dev cases ported as fixtures or evalloop cases and expectations rewritten to v2 requirements.

### 6.4 Where decisions live

- **Per change.** Each change's `design.md` holds its decisions, one or two sentences each, naming the rejected alternative (`baseline.yaml:26`). The proposal's `Request` quotes the captain verbatim, and `Approval` stays `pending` until a reviewer fills it (`baseline.yaml:13`). This is the invoz.v2 practice; for example, `FM/projects/invoz.v2/openspec/changes/archive/2026-10-29-add-transcription-api/design.md:9-30`.
- **Archived together.** Archiving keeps proposal, design and tasks in `openspec/changes/archive/<date>-<id>/` next to the spec they produced, in the same branch (`baseline.yaml:37-38`). That is the link from a spec to its decisions.
- **Structural captain decisions** (D1-D7) go into the proposal and design of the first change they shape, with the captain's exact words. For example, D1 lands in the `operator-ui` change and D4 in the `thought-drafting` change. The ledger points to them.
- **Research.** A Phrona or other research result is cited from the design by path and digest. A short summary goes in `docs/research/<topic>.md` only when more than one change relies on it.
- **No root `changes/` and no `docs/decisions/` folder** unless D7 says otherwise. Two record systems would compete. Code 5.0.0's repository-standard still names `changes/`, `specs/` and `.spec-check.toml` (`/Users/camiloslaptop/.agents/skills/code/references/repository-standard.md:18-24`), while the Fleet baseline says global `/specs` is retired and OpenSpec owns artifacts (Fleet `docs/development.md:10-13`). That inconsistency belongs to the baseline and Code owners, not to syncloom.v2.

---

## Preservation inventory (nothing below was touched)

- **Captain's copy** `/Users/camiloslaptop/github/syncloom`: branch `retire-graphify` at `0a3a44f`, open as https://github.com/camilojourney/syncloom/pull/33 (from this session's gh-axi start-up output). Its `data/` was not opened.
- **`fm/syncloom-evalloop-fable` at `16890bd`:** evalloop map, 7 components with dev cases, spec 038 draft (`FM/data/syncloom-evalloop-fable/notes/RESUME.md:41-58`). The campaign was stopped and must not be resumed.
- **`fm/changes-folder-syncloom-20261024` at `0c5e6ef`:** `changes/README.md`, captain-held pending a local merge approval (`FM/data/changes-folder-syncloom-20261024/position.md:3-5`).
- **Runtime data:** the `data/*` symlinks to `/Volumes/SSD/holus/*` (`AGENTS.md:225-232`), the local `data/content-queue`, and tracked `data/personal-context.json`. Importing any old content or voice data into v2 is a separate, explicit change.
- **Holus Social API** (`6e9a37b`) and its deployment gaps (`FM/data/intake-holus-api/report.md:26-41`).

## Work that should ship regardless of v2 (old repo; for firstmate to route)

1. The Social API client drift fix described in `FM/data/intake-holus-api/report.md:68-84`. It is still present at `3efcd35` (section 2.1).
2. Non-hermetic tests that write into `data/` (section 2.2).
3. Platform fit using 3000 characters for X and Threads (`thought_pipeline.py:1659-1667`).

## Evidence commands (selection)

- `git -C <worktree> log --oneline -5`; `git -C <worktree> branch -a`; `git -C <worktree> show --stat 16890bd 4855b13`.
- `pytest` as recorded in `pytest-summary.txt`: 1892 tests, 0 failures, 0 errors, 1 skipped, 99.6 s, exit 0.
- `grep -rli <term> specs` for each core-loop term (section 2.3); `git ls-files docs/reference | grep -c node_modules` gives 548; `git ls-files data | wc -l` gives 89.
- `git -C <fleet-system> show 4501883843df70c50f0726120aca09387ca6c95d:system/shared/openspec/baseline.yaml` and `:docs/development.md`.
- `openspec --version` gives 1.15.0; `ls ~/.cargo/registry/cache/*/` (crate versions above); `find / -maxdepth 6 -name cargo` finds `/nix/store/...-cargo-1.98.1/bin/cargo`.
