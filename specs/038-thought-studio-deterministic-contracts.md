# Spec 038: Thought Studio Deterministic Contracts

**Status:** in-progress
**Phase:** Phase 1
**Author:** Camilo Martinez (drafted by claude-fable-5-1 in the evalloop epoch of 2026-10-16)
**Created:** 2026-10-16
**Updated:** 2026-10-16

## Scope

Kind: product

Owns: the observable behavior of the deterministic Thought Studio path
(`ThoughtContentPipeline` without model keys) as a reviewer sees it on a queue record.
That covers the account language route, the thesis, source intent and mentioned products,
the platform-fit verdict, product names in drafts, and the visible visual hook.
Does not own: agent-backed generation, URL extraction, rendering quality, or Holus
Social API delivery (contained; see [Spec 016](016-social-media-integration-v2.md)).

## Problem

A founder captures one thought and opens the review drawer. Today the drawer can show
the wrong brand account for an English thought that contains "de" or "los". The thesis
can be cut inside a word at 72 characters and then appear in every draft. An intake label
such as "Research note:" can sit inside the thesis and the carousel headline. A thread
segment longer than a post limit can be marked as fitting its platform. A draft can pitch
Holus for a thought about something else. Every ordinary visual can carry a REDO verdict
with violations. Each of these costs a manual correction before anything can be approved.

## Goals

- The account route is right for plain English, plain Spanish, English with Spanish-looking
  tokens, and accent-less Spanish.
- The thesis is a complete first sentence, label-free, never cut inside a word.
- Platform fit uses the platform's own limit, per thread segment for X.
- Drafts name only products the thought names.
- The visual hook is label-free and passes the local judge for ordinary thoughts.

Each goal is measured by an evalloop component under `evals/parts/` with sealed cases.

## Non-Goals

- Replacing the deterministic composer with agent-backed generation (roadmap "Next").
- Semantic rewriting of the thesis; the thesis stays a span of the thought.
- Changing `holus_social_api` limits or client behavior.

## Requirements

### Language route by evidence

`req~thought-studio.language-route~1`

WHEN a thought is routed to a posting destination, the pipeline SHALL choose the
language whose whole-word function-word and accent evidence is stronger, defaulting
to English on a tie.

Needs: impl, utest

### Spanish-looking tokens in English

`req~thought-studio.language-route-lookalikes~1`

WHEN an English thought contains isolated Spanish-looking tokens such as de, con, los,
las, una or por, the pipeline SHALL still route it to the English account.

Needs: impl, utest

### Accent-less Spanish

`req~thought-studio.language-route-plain-spanish~1`

WHEN a Spanish thought has no accented characters and none of the former marker words,
the pipeline SHALL still route it to the Spanish account.

Needs: impl, utest

### Thesis is the first sentence

`req~thought-studio.thesis-first-sentence~1`

The pipeline SHALL use the thought's first sentence as the thesis, with trailing sentence
punctuation removed. A sentence closes at ".", "!" or "?" followed by a space or the end
of the text.

Needs: impl, utest

### Intake label removed

`req~thought-studio.thesis-label~1`

WHEN the thought starts with a label of one to three words ending in a colon, the
pipeline SHALL remove the label before extracting the thesis and upper-case the thesis's
first character.

Needs: impl, utest

### Abbreviations do not end a sentence

`req~thought-studio.thesis-abbreviation~1`

The pipeline SHALL NOT close a sentence at a period that belongs to an abbreviation or
a single-letter initial. The abbreviations are e.g., i.e., vs., etc., Dr., Mr., Mrs., Ms.,
No. and St.

Needs: impl, utest

### Thesis length

`req~thought-studio.thesis-length~1`

WHEN the first sentence is longer than 200 characters, the pipeline SHALL cut the thesis
at the last space within the first 200 characters and strip trailing punctuation. A thesis
SHALL never end inside a word.

Needs: impl, utest

### Mentioned products

`req~thought-studio.product-mentions~1`

The pipeline SHALL list, sorted and in canonical spelling, every configured portfolio
product (Pilaster, Genpeli, Invoz) named as a whole word in the thought, case-insensitively.

Needs: impl, utest

### Intent precedence

`req~thought-studio.intent-precedence~1`

WHEN classifying a text thought, the pipeline SHALL return product_context when a product
is mentioned. Otherwise it SHALL return campaign_idea for campaign, launch or series,
then research_note for research, study, paper or benchmark, then raw_thought.

Needs: impl, utest

### Platform fit uses platform limits

`req~thought-studio.platform-fit-limits~1`

WHEN evaluating platform fit, the pipeline SHALL return REVIEW when the text is shorter
than 20 characters or longer than the platform's inclusive character limit in
`platform_config`. Otherwise it SHALL return PASS.

Needs: impl, utest

### Thread segments

`req~thought-studio.platform-fit-thread~1`

WHEN the platform is twitter_x, the pipeline SHALL apply the 280-character limit to each
thread segment. A segment is a blank-line-separated paragraph group starting with "n/".

Needs: impl, utest

### Drafts name only the thought's products

`req~thought-studio.copy-products~1`

A generated draft SHALL name no product among Holus, Pilaster, Genpeli and Invoz that
the thought itself does not name.

Needs: impl, utest

### Visual hook

`req~thought-studio.visual-hook~1`

The visible hook of a carousel cover or image card SHALL be derived from the thesis
without any intake label. A hook longer than 72 characters SHALL be cut at a word boundary.

Needs: impl, utest

### Judge accepts the thesis as the hook

`req~thought-studio.visual-judge-thesis~1`

WHEN the visible copy's hook is the extracted thesis or its word-boundary shortening,
the local visual judge SHALL NOT flag it as repeating the raw thought opening.

Needs: impl, utest

## Acceptance Criteria

- Every component under `evals/parts/` reaches match_fraction 1.0 on its dev cases.
- Each component change earns an `accepted` evalloop decision with `evals/e2e/check.py` passing.
- `just check` passes.

## Decisions

- Thesis stays a span of the thought (case-insensitive) so review can trace it.
- The 200-character cut and 72-character hook cut are declared here, not tuned to cases.
- Ties in language evidence default to English because the English account is the
  configured default destination.
