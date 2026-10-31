## ADDED Requirements

### Requirement: Source language
The studio SHALL classify source thoughts as English or Spanish, retaining English for tied evidence, empty input and unknown language.

#### Scenario: English lookalikes
- **WHEN** an English sentence contains an isolated `de`, `los`, `con`, `las`, `una` or `por`
- **THEN** its language remains English
- **AND** `The café is open and the résumé is ready.` remains English

#### Scenario: Spanish without accents or former markers
- **WHEN** the thought is `Nuestros clientes necesitan herramientas simples y resultados claros.`
- **THEN** its language is Spanish
- **AND** `Necesitamos escribir mejor contenido cada dia.` is Spanish

#### Scenario: Accents and ties
- **WHEN** the thought is `¿Cómo diseñamos una solución más útil?`
- **THEN** its language is Spanish
- **AND** `the el`, empty text and product names alone default to English

### Requirement: Founder destination
The studio SHALL map source language to a founder review destination without granting delivery or changing the connected platform identity.

#### Scenario: English and Spanish accounts
- **WHEN** new pieces are drafted from English and Spanish thoughts
- **THEN** their destination accounts are respectively `camiloexperience` and `camilojourney`, with `@`-prefixed handles
- **AND** their platform follows `platform-rules`, language is `en` or `es`, and `approval_required` is true
- **AND** provider output language does not change the source destination

### Requirement: Deterministic thesis
The studio SHALL extract the source's first sentence as a capitalized thesis without leading intake labels or trailing punctuation.

#### Scenario: Labels and first sentence
- **WHEN** the thought is `Research note: better tools help teams. Another thought.`
- **THEN** the thesis is `Better tools help teams`
- **AND** `Campaign idea: launch Pilaster! Then Genpeli.` produces `Launch Pilaster`
- **AND** empty input or a label alone produces an empty thesis

#### Scenario: Abbreviations
- **WHEN** the first sentence contains `e.g.`, `i.e.`, `vs.`, `etc.`, `Dr.`, `Mr.`, `Mrs.`, `Ms.`, `No.` or `St.`
- **THEN** the abbreviation does not prematurely end the thesis
- **AND** `Dr. Smith builds tools. Next.` produces `Dr. Smith builds tools`

### Requirement: Thesis length
The studio SHALL limit the thesis to 200 Unicode scalar characters without splitting words.

#### Scenario: Word boundary
- **WHEN** adding the next word would exceed 200 characters
- **THEN** the thesis ends before that word, without trailing whitespace
- **AND** a single word exceeding 200 characters produces an empty thesis
- **AND** a word exactly 200 characters long is preserved

### Requirement: Portfolio mentions
The studio SHALL extract only named portfolio products from whole-word source mentions, uniquely and case-insensitively in catalog order.

#### Scenario: Product catalog
- **WHEN** the source is `invoz and PILASTER help Genpeli. Pilaster again.`
- **THEN** products are `Pilaster`, `Genpeli`, `Invoz` in that order
- **AND** `Pilasters Genpelis Invozip` produces no product mentions

### Requirement: Clean visual hook
The studio SHALL derive a label-free visual hook from the thesis, limited to 72 Unicode scalar characters without splitting words.

#### Scenario: Hook boundary
- **WHEN** a thesis exceeds 72 characters
- **THEN** the hook ends at its last complete word within 72 characters
- **AND** a first word exceeding 72 characters produces an empty hook
- **AND** a word exactly 72 characters long is preserved

### Requirement: Routing contract interface
The studio SHALL return and persist routing, thesis, visual_hook and products on newly created pieces, preserving compatibility with records lacking this metadata.

#### Scenario: HTTP and Rust interfaces
- **WHEN** `analyze_thought(text, channel)` is called with any text and supported Channel
- **THEN** it returns a ThoughtContract containing destination, thesis, visual_hook and products without errors or side effects
- **AND** POST /v1/thoughts pieces expose that contract as `routing`, preserved by GET /v1/pieces/{id}
- **AND** editing generated text preserves source-derived routing

#### Scenario: Legacy records
- **WHEN** an older piece without `routing` is loaded
- **THEN** it remains readable and no metadata is invented or written back
