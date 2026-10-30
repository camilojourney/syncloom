# Q7: rendering 1080x1350 images and multi-page PDF carousels from Rust

Status: open

## Question

How syncloom.v2 should render 1080x1350 PNG images and multi-page PDF carousels from Rust: headless Chromium over CDP (R1), Typst (R2), resvg with a PDF writer (R3) or WKWebView (R4), judged on pixel stability, typography, gradients, determinism, packaging, PDF size and accessibility, LinkedIn's document limits, and crate maintenance. The question as sent is [`questions/q7-render-images-carousels-rust.md`](../2026-10-30-social-api-research-questions/questions/q7-render-images-carousels-rust.md) (sha256 `df615de9...`, verified unchanged by the operator before the study started).

## Why asked

Slice 3, visual assets (the plan's Q-RENDER): the renderer choice decides whether the old HTML/CSS templates carry over or are rewritten. Recorded now, built later; nothing in slice 2 depends on it.

## What would change the answer

The question's condition (Typst or resvg reaching pixel-stable output with acceptable typography on the existing gradient and font design) was not tested by either report. The Claude report names the test that would settle it: a gated two-week Typst spike on about ten of the 49 rendering tests, judged on a perceptual diff, byte-identical repeat runs, no conic-gradient divergence between PDF and PNG, selectable tagged text and latency. A Typst release that ships the conic-gradient PDF fix, or resvg gaining text wrapping, would also move it.

## How collected

Phrona 5.0.12 study `main969-q7-render`, run by the phrona-lab home on 2026-10-30: Claude Research primary (collected natively, 77 citations, after one `research-not-finished`) and ChatGPT Deep research challenger (collected by hand; sources panel "Sources scanned 0", so model output without citations, lesson 18). The study ended at its coordinator call limit (`original-limits`), so Phrona marks it incomplete and not default-eligible; both reports are complete documents. Langfuse trace `6653da1959815016f5886aee4b92fd38`, 208 events, known cost USD 0.120. `q7-report.md` is the operator's summary; `q7-post.json` the result record.

## Status

Open: the production choice is recorded in [decision 0003](../../decisions/0003-visual-renderer.md): keep headless Chromium, with a gated later Typst spike; resvg-only and WKWebView are not the main path. Browser distribution, cross-machine equivalence and the Typst spike remain later work; neither report establishes those outcomes.

## Decision

[Decision 0003](../../decisions/0003-visual-renderer.md) records the slice 3 choice. Local Rust rendering is implemented on the visual slice branch; no distribution or platform upload is claimed.

## Files

Verbatim copies; the repository never edits them. Origins are paths on the machine that ran the study.

| File | Origin | sha256 |
|---|---|---|
| `q7-report.md` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q7-report.md` | `83412befb00e0a107b66a46b8e85b5735202059f03b7515df95bea6b4d6aba16` |
| `q7-claude-NATIVE-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q7-claude-NATIVE-report.txt` | `2cb770179892784949e8d77444a7c3eefdfd433c72da36e91dcf60b41f050093` |
| `q7-chatgpt-HAND-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q7-chatgpt-HAND-report.txt` | `4d5b4b7e7d814c05f220de9c01844215770ce9d87c0096782edc119cf0fd7e0b` |
| `q7-post.json` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q7-post.json` | `355736d4f5458c47cb67300f087dc53a95aa3ff1d3ff8ebd579d994f2aabf781` |
