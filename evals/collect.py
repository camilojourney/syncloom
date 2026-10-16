#!/usr/bin/env python3
"""Frozen evalloop collector for the Syncloom Thought Studio components.

Runs the application's own deterministic functions on each case's ``inputs`` and writes one
``structured_replay`` observation per case (``{"case_id": ..., "actual": {...}}``) into
``evals/parts/<component>/variants/<variant>/observations/``. It is listed under
``[protected] frozen`` in ``evals/policy.toml`` so a candidate cannot change how outputs are
projected into observations. It never prints case contents, so it can run over the sealed
split without revealing it.

Usage (from the repository root, inside the project environment)::

    uv run python evals/collect.py <component> <variant> [--oracle]

``--oracle`` writes each dev case's declared ``expected`` (and ``required``) values as the
observation instead of running the application. It exists only for the ``reference`` variant
that ``evalloop prove-cases`` uses to show the cases are not vacuous; it never runs over the
sealed split and is never application evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tomllib
from collections.abc import Callable, Mapping
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO / "src"))

from holus.agents.marketing.thought_pipeline import (  # noqa: E402
    CAROUSEL_CHANNELS,
    _build_platform_text,
    _clean_thought,
    _extract_thought_essence,
    _infer_source_intent,
    _platform_fit,
    _prepare_judged_carousel_outline,
    _prepare_judged_visual_spec,
    _select_visual_brief,
    _source_context,
    detect_language_for_destination,
)

# Product names the studio may name in copy. Holus is the studio itself; the other three are
# the configured portfolio products (config/products.yaml). Canonical spelling is recorded.
PRODUCT_NAMES = ("Holus", "Pilaster", "Genpeli", "Invoz")
_PRODUCT_RE = re.compile(r"\b(" + "|".join(PRODUCT_NAMES) + r")\b", re.IGNORECASE)
_LABEL_PREFIX_RE = re.compile(r"^\s*(?:[A-Za-z][\w'-]*\s+){0,2}[A-Za-z][\w'-]*:\s")


def products_named(text: str) -> list[str]:
    """Sorted canonical product names that appear as whole words in ``text``."""
    found = {match.group(1).casefold() for match in _PRODUCT_RE.finditer(text)}
    return sorted(name for name in PRODUCT_NAMES if name.casefold() in found)


def repeats_raw_opening(thought: str, copy: str) -> bool:
    """True when the visible copy contains the first five words of the raw thought."""
    words = _clean_thought(thought).lower().split()
    if len(words) < 5:
        return False
    return " ".join(words[:5]) in copy.lower()


def has_label_prefix(copy: str) -> bool:
    """True when the copy starts with a one-to-three-word intake label such as 'Research note:'."""
    return bool(_LABEL_PREFIX_RE.match(copy))


def collect_language_route(inputs: Mapping[str, Any]) -> dict[str, Any]:
    return {"language": detect_language_for_destination(str(inputs["thought"]))}


def collect_source_intent(inputs: Mapping[str, Any]) -> dict[str, Any]:
    text = _clean_thought(str(inputs["thought"]))
    return {"intent": _infer_source_intent(text, str(inputs.get("source_type", "text")))}


def collect_product_mentions(inputs: Mapping[str, Any]) -> dict[str, Any]:
    text = _clean_thought(str(inputs["thought"]))
    intent = _infer_source_intent(text, str(inputs.get("source_type", "text")))
    return {"mentioned_products": sorted(_source_context(text, intent)["mentioned_products"])}


def collect_thesis_extraction(inputs: Mapping[str, Any]) -> dict[str, Any]:
    return {"thesis": _extract_thought_essence(str(inputs["thought"])).thesis}


def collect_platform_fit(inputs: Mapping[str, Any]) -> dict[str, Any]:
    record = {
        "text": str(inputs["text"]),
        "platform": str(inputs["platform"]),
        "content_type": str(inputs["content_type"]),
        "platform_job_plan": "frozen transformation job",
    }
    return {"verdict": _platform_fit(record)["verdict"]}


def collect_copy_relevance(inputs: Mapping[str, Any]) -> dict[str, Any]:
    text = _build_platform_text(_clean_thought(str(inputs["thought"])), str(inputs["channel"]))
    return {"products_in_copy": products_named(text)}


def collect_visual_copy(inputs: Mapping[str, Any]) -> dict[str, Any]:
    thought = _clean_thought(str(inputs["thought"]))
    channel = str(inputs["channel"])
    essence = _extract_thought_essence(thought)
    brief = _select_visual_brief(thought, channel, str(inputs.get("nonce", "evalloop")))
    if channel in CAROUSEL_CHANNELS:
        outline, _brief, judge = _prepare_judged_carousel_outline(thought, essence, channel, brief)
        hook = str(outline["slides"][0]["variables"].get("headline") or "")
    else:
        spec, _brief, judge = _prepare_judged_visual_spec(thought, essence, channel, brief)
        hook = str(spec.get("hook") or "")
    return {
        "judge_verdict": judge.verdict,
        "judge_score": judge.score,
        "redo_count": judge.redo_count,
        "hook": hook,
        "hook_repeats_raw_opening": repeats_raw_opening(thought, hook),
        "hook_has_label_prefix": has_label_prefix(hook),
    }


COLLECTORS: dict[str, Callable[[Mapping[str, Any]], dict[str, Any]]] = {
    "language-route": collect_language_route,
    "source-intent": collect_source_intent,
    "product-mentions": collect_product_mentions,
    "thesis-extraction": collect_thesis_extraction,
    "platform-fit": collect_platform_fit,
    "copy-relevance": collect_copy_relevance,
    "visual-copy": collect_visual_copy,
}


def _git_head() -> str:
    try:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True, text=True, check=True
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return ""


def _read_cases(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def _resolve_case_path(component_dir: Path, raw: str) -> Path:
    path = Path(raw).expanduser()
    if path.is_absolute():
        return path
    if (component_dir / path).exists():
        return component_dir / path
    return REPO / path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("component")
    parser.add_argument("variant")
    parser.add_argument(
        "--oracle", action="store_true", help="write declared expectations (dev only)"
    )
    args = parser.parse_args(argv)

    component_dir = REPO / "evals" / "parts" / args.component
    manifest = tomllib.loads((component_dir / "component.toml").read_text(encoding="utf-8"))
    if manifest["component"]["id"] != args.component:
        print(json.dumps({"error": "component id does not match directory"}))
        return 1
    collector = COLLECTORS.get(args.component)
    if collector is None:
        print(json.dumps({"error": f"no collector registered for {args.component}"}))
        return 1

    splits: list[tuple[str, Path]] = [
        ("dev", _resolve_case_path(component_dir, manifest["cases"]["dev"]["path"]))
    ]
    sealed = manifest["cases"].get("sealed")
    if sealed and not args.oracle:
        splits.append(("sealed", _resolve_case_path(component_dir, sealed["path"])))

    out_dir = component_dir / "variants" / args.variant / "observations"
    out_dir.mkdir(parents=True, exist_ok=True)
    collector_sha256 = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    head = _git_head()
    counts: dict[str, int] = {}
    errors: dict[str, int] = {}
    for split, path in splits:
        if not path.is_file():
            print(json.dumps({"error": f"{split} case file missing"}))
            return 1
        for case in _read_cases(path):
            if args.oracle:
                actual: dict[str, Any] = {**case.get("required", {}), **case["expected"]}
            else:
                try:
                    actual = collector(case["inputs"])
                except Exception as exc:  # an errored case is recorded, not hidden
                    errors[split] = errors.get(split, 0) + 1
                    actual = {"error": type(exc).__name__}
            record = {
                "case_id": case["id"],
                "actual": actual,
                "evidence_kind": "declared-oracle" if args.oracle else "replayed-pure-function",
                "source_revision": head,
                "collector_sha256": collector_sha256,
            }
            (out_dir / f"{case['id']}.json").write_text(
                json.dumps(record, indent=2, ensure_ascii=False, sort_keys=True) + "\n",
                encoding="utf-8",
            )
            counts[split] = counts.get(split, 0) + 1
    print(
        json.dumps(
            {
                "component": args.component,
                "variant": args.variant,
                "oracle": args.oracle,
                "observations": counts,
                "errored": errors,
                "source_revision": head,
                "output": str(out_dir.relative_to(REPO)),
            }
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
