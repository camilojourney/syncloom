# Design

References: security, data.

## Context

See proposal.md. The old owner used substring markers and unconditionally treated accents as Spanish; its founder handles remain the migration input. Existing drafts are model-backed and stay that way, rather than reviving a deterministic composer.

## Goals / Non-Goals

One pure Rust owner of source-derived review metadata; no model inference or external language service. No translation, generated copy replacement, automatic rendering, or credential/account switching.

## Decisions

- Extend `thought-drafting` rather than create a parallel routing capability: this contract belongs to thought intake, not channel bounds. Keep all extraction and routing in `routing.rs`, with plain text and Channel inputs and a typed ThoughtContract output; total functions have no error cases.
- Compare occurrence counts of whole Unicode words from common English/Spanish frequency vocabulary, with one additional Spanish vote per accented word and one for inverted punctuation. Fold Spanish accents including decomposed combining marks; strict greater-than selects Spanish, so unknown words and ties default to English. Reject substring and accent-veto alternatives: isolated lookalikes and borrowed words must not override stronger English evidence.
- Remove known intake labels before classification as well as extraction, avoiding English metadata bias. Extract the first sentence with an explicit abbreviation list, uppercase its first character, then share one word-prefix helper for the 200/72 Unicode-scalar budgets; an oversized first word yields empty rather than a broken word. Product mentions use whole Unicode words over the entire source, unique in catalog order.
- Persist an optional contract when creating pieces, rather than recompute historical routing on reads after vocabulary changes. Old pieces deserialize without routing; raw source, text revisions, approval binding and provenance schema are unchanged. Editing generated text does not reroute the original source.
- Preserve old founder targets `camiloexperience` (English) and `camilojourney` (Spanish), with platform derived only from Channel. The result explicitly requires approval and is review metadata, never authorization or a claim that a connection exists. Reject silently replacing the authenticated destination: current platform adapters retain their explicit sign-in and containment rules.

## Security boundaries

Only the existing local operator may create/read pieces through existing guarded endpoints; no new path or permission is introduced. Routing reads source text already accepted by intake; it performs no URL fetch and exposes no credentials. Source and derived text remain in the piece store, never provenance events or logs; secrets stay with existing credential owners.

Kept: crates/syncloom-core/src/piece.rs and crates/syncloom-core/src/routing.rs, because the 31-token enum derive/serde annotation clone is serialization boilerplate for independently owned Status and Language types, not duplicated domain policy. Merging these unrelated enums would couple review transitions to language selection.

## Risks / Trade-offs

- Vocabulary evidence is a deterministic EN/ES heuristic, not universal linguistic certainty. Unknown/mixed input uses documented English ties; corpus regression tests must accompany future vocabulary changes.
- Spec 038 object is absent from the local old repository. The brief's selected behaviors are tested; visual judging and the old deterministic composer are intentionally not ported.
- Review metadata does not implement dual-account sign-in. It must not be represented as a delivery receipt.

## Migration Plan

Add optional `routing` metadata only to newly drafted pieces; no data rewrite. Rollback ignores the additive field; old records round-trip unchanged. The migration ledger marks language routing migrated in Slice 6, while connected-platform account capabilities keep their existing owners.
