# Syncloom Q7: rendering 1080x1350 PNG and PDF carousels from Rust

Study `main969-q7-render` on Phrona 5.0.12 (phrona.v2), Claude Research primary, ChatGPT Deep research challenger. Question file `questions/q7-render-images-carousels-rust.md` passed unchanged, SHA256 `df615de9099ec8d95f838d07d31f18d05a1e14f4a4f000003b0b6bc6592f6f0c` (verified against `question-hashes.txt` before start).

## Provenance (read this first)

| Provider | Label | Conversation | Size | SHA256 (word for word) | Sources |
|---|---|---|---|---|---|
| Claude (primary) | NATIVE | https://claude.ai/chat/6ff2a228-1b2e-408e-9571-5d34c0f9218c | 18393 chars / 18734 B | `2cb770179892784949e8d77444a7c3eefdfd433c72da36e91dcf60b41f050093` | 77 citations |
| ChatGPT (challenger) | HAND | https://chatgpt.com/c/6ae4dcc6-6cf4-83ea-90aa-f0d0a8b57610 | 9965 chars / 10075 B | `4d5b4b7e7d814c05f220de9c01844215770ce9d87c0096782edc119cf0fd7e0b` | sourcesScanned 0; panel: Citations 0 ("No sources detected"); 3 links inside the report body |

- Report files: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q7-chatgpt-HAND-report.txt`; `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q7-claude-NATIVE-report.txt`. Native Claude original also at `/Users/camiloslaptop/.local/state/phrona/durable-study-5.0.12-candidate/studies/main969-q7-render/reports/claude-2cb770179892784949e8d77444a7c3eefdfd433c72da36e91dcf60b41f050093.txt`.
- Outcome: `original-limits` ("Original coordinator call limit reached: 40 of 40 calls used"). Phase `incomplete`, `defaultEligible: False`. Coordinator calls 40/40, native calls 0, known cost USD 0.120. Langfuse trace `6653da1959815016f5886aee4b92fd38`, 208 events received, 0 pending.
- Collection failure codes recorded by 5.0.12: {"chatgpt": {"report-model-only-no-sources": 1, "report-unfinished-or-ambiguous": 7}, "claude": {"research-not-finished": 1}}.
- HAND reads (ChatGPT) were zero-click and zero-input (inputEvents {cg.get('inputEvents','n/a')}, clicks {cg.get('clicks','n/a')}) after browser-identity, exact-URL, question-association and frame-lineage checks, taken from the Deep research iframe. HAND reports never count toward native acceptance.
- ChatGPT Sources and activity panel (5.0.12 opened it natively; I made no click): Sources scanned 0, Citations 0 ("No sources detected"), connector sources scanned 0. Panel text: `Sources | Activity · 5m |  | Citations |  | No sources detected |  | Sources scanned · 0 |  | No sources scanned |  | Connector sources scanned |  | No connector sources scanned`.


## Answer tied to the DECISION (renderer for 1080x1350 PNG and multi-page PDF carousels from Rust)

Claude (NATIVE, 77 citations): keep R1, headless Chromium, as the production renderer, and run a gated two-week Typst (R2) spike on about 10 of the 49 rendering tests. Drop Chromium only if the Typst port passes a perceptual diff the founder accepts, byte-identical repeat runs, no PDF versus PNG conic-gradient divergence, selectable tagged text and acceptable latency. R3 (resvg) and R4 (WKWebView) are not the main path.

ChatGPT (HAND, uncited): R1 is the safe fallback for complex web designs, but it leans toward R3 (SVG templates, resvg, printpdf) "if you can translate your templates into SVG" and calls R3 worth pursuing. R4 not recommended. Typst only if designs fit its model.

So both agree on R4 (no) and on keeping R1 as the fallback. They differ on R3 and on how confident to be about pixel stability.

Your "WOULD CHANGE THE ANSWER" condition (Typst or resvg reaches pixel-stable output with acceptable typography on the existing gradient and font design): neither report tested it. Documented evidence in the Claude report says the condition is not met today for the existing design, and names the spike that would test it.

## Evidence in the Claude report that bears on the decision

- Gradients: Typst has linear, radial and conic, but after the PDF backend moved to krilla, conic gradients in PDF differ from PNG (Typst issue #7890, fixes on main, not confirmed shipped in 0.15.1). SVG has no conic gradient, so R3 needs wedge emulation (Satori does this). No CSS-style blend modes or filters in Typst; blur-filtered SVG took about 600 ms to compile to PDF versus about 50 ms before the krilla switch (#6244).
- Determinism: Typst 0.15 states floating-point calculations are consistently deterministic across platforms; with a fixed document date and bundled fonts, byte-identical PDFs are reported by downstream projects. resvg/krilla CPU rasterization is deterministic for a fixed crate version and font set, but resvg 0.48 (Jul 2026) changed its text stack (skrifa plus harfrust) and warns of small rendering changes, so expect re-baselining on minor upgrades. Chromium: stable on one pinned binary and machine with `--font-render-hinting=none`, software rendering, DPR 1, but snapshot tests need a tolerance; Puppeteer maintainers say screenshots are platform dependent. PDF metadata (CreationDate, ModDate, ID) needs normalising.
- Chromium PDF gradients: the peitho project measured macOS Preview taking 14.2 s on one gradient slide (Skia type 4 shadings plus soft masks), cut to 33 ms by pre-rasterising gradients. Relevant because the founder reviews on Mac and phone.
- Packaging: chrome-headless-shell (mac-arm64) about 86 MB, full Chrome for Testing about 151 MB; pin the version, download on first run, verify checksum (Gatekeeper can mark browser-downloaded zips damaged). Check redistribution terms before bundling Chrome for Testing binaries. Typst and resvg compile into the binary. R4: wry has no PDF export API (open since 2022), WebKit cannot be pinned, so snapshot tests drift across macOS updates.
- PDF size and accessibility: print all slides as one HTML document with `@page { size: 1080px 1350px; margin: 0 }` rather than merging per-slide PDFs. Chromium PDFs are tagged by default but Gotenberg reports tagging toggles misbehaving in newer builds. Typst is tagged by default since 0.14 with PDF/A and PDF/UA. A third-party test measured a 10-slide 1080x1350 PDF at 280 KB.
- LinkedIn: one PDF up to 100 MB and 300 pages, all pages the same size; 1080x1350 is a tool convention, not an official LinkedIn spec; LinkedIn displays pages as images (a 2019 engineering post also describes a selectable text layer); links are clickable only after download. PDF/X is a print standard and not required. LinkedIn help pages were read from search snippets, so test selectable text in the current feed manually.
- Crate maintenance (Oct 2026): chromiumoxide 0.9.1 (25 Feb 2026; 0.7 to 0.9 dropped async-std and feature flags), headless_chrome 1.0.21, typst 0.15.1 (17 Jul 2026, API breaks between minors), resvg 0.49.0 (24 Oct 2026), krilla 0.8.2, svg2pdf archived on 10 Jul 2026, lopdf 0.46. Open-issue counts not collected.
- No published head-to-head latency benchmark exists for this workload; latency numbers are expectations to measure.

## Where the ChatGPT report is weaker or conflicts (HAND, uncited)

- It says resvg "guarantees pixel-reproducible output across platforms" and handles gradients and blend modes, and calls R3 attractive. Claude's evidence contradicts the strong form: determinism holds for a pinned crate version and font set, SVG has no conic gradients, and there is no automatic text wrapping in SVG, so R3 means building a text layout engine for multi-line slide copy.
- LinkedIn: it says LinkedIn "recommends about a 4:5 ratio, for example 1200x1500". Claude found no official pixel spec. Treat the 1200x1500 figure as unsourced. Limits of 100 MB and 300 pages agree between the two.
- Its Typst statements are marked "no direct citation" by the report itself, and it omits the conic PDF versus PNG issue. Its R4 dismissal is brief and omits the wry PDF gap.
- It cites only three links inside the body (chromiumoxide docs, Typst issue #2988, printpdf docs); the sources panel found none (0 scanned, 0 citations).

## What I would not conclude

That R1 or Typst wins on measured fidelity. Both reports say no published benchmark covers this workload, and neither saw the actual 49 templates. The two-gate spike is the evidence that would settle it.

## Collection provenance and 5.0.11 / 5.0.12 behaviour seen live

- Claude was the primary and was sent first, as configured (`primary: claude`). It collected natively as `complete-loaded-claude-document` after one `research-not-finished` (no 4 minute wait wasted). The native Claude path for a report with 77 citations worked.
- 5.0.12 opened ChatGPT's Sources and activity panel by itself and read "Sources scanned 0", then recorded `report-model-only-no-sources` (1x). It then recorded `report-unfinished-or-ambiguous` 7x on a page that, when I read it afterwards, held one complete report document, no busy nodes, the correct question and the panel still open. My read: after the model-only refusal, further native reads kept refusing until the study hit the 40/40 coordinator limit. Hypothesis, not proven: the panel left open changes the layout the reader expects.
- The study ended `original-limits` (40/40 coordinator calls), not a native full pair, so it is not default-eligible.
- HAND applies to ChatGPT: the report is complete (4 H1 sections, ends with its Sources line, 0 busy nodes), and 5.0.12 refused it. Claude needed no HAND.
- Q7 is the study whose parallel start with Q8 caused the Claude draft leak into Q8's page (Q7's own Send was clean; see the Q8 report).

All findings are model research from two reports. Two providers agreeing is not independent evidence. Default stays 4.3.23.
