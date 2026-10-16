#!/usr/bin/env python3
"""Frozen end-to-end guard for the Syncloom Thought Studio (promotion authority).

Runs ``ThoughtContentPipeline.create_content_set`` on the frozen thoughts in ``cases.jsonl``
over text, carousel and image channels, and judges the whole content set a reviewer would open.
Every criterion is absolute and satisfied by the epoch baseline, so a failing run means the
candidate regressed the whole flow even if its own part improved. Prints one JSON object with
``"passed"`` and exits 0 on pass, 1 on failure. Rendered files go to a temporary directory.

Environment from evalloop (``EVALLOOP_REPO``, ``EVALLOOP_COMPONENT``, run/variant paths) is
accepted but the verdict depends only on the repository code and the frozen cases.
"""

from __future__ import annotations

import asyncio
import json
import os
import re
import sys
import tempfile
from pathlib import Path
from typing import Any

REPO = Path(os.environ.get("EVALLOOP_REPO") or Path(__file__).resolve().parents[2]).resolve()
sys.path.insert(0, str(REPO / "src"))

from holus.agents.marketing.thought_pipeline import ThoughtContentPipeline  # noqa: E402

CHANNELS = [
    "linkedin_text",
    "threads_text",
    "twitter_x_thread",
    "facebook_text",
    "linkedin_carousel",
    "instagram_image",
]
PLATFORM_LIMITS = {
    "linkedin": 3000,
    "threads": 500,
    "instagram": 2200,
    "facebook": 63206,
    "twitter_x": 280,  # per thread segment
}
MIN_TEXT = 20
MAX_THESIS = 200
REQUIRED_CHECKLIST = {
    "source_evidence",
    "source_context",
    "channel_plan.transformation_job",
    "quality_evaluation.platform_fit_summary",
    "approval_workflow.publish_gate",
}
_SEGMENT_RE = re.compile(r"^\d+/\s")


def _clean(text: str) -> str:
    return " ".join(str(text or "").split())


def _segments(text: str) -> list[str]:
    """Thread segments: blank-line separated paragraphs grouped under their 'n/' heading."""
    segments: list[str] = []
    for paragraph in text.split("\n\n"):
        if _SEGMENT_RE.match(paragraph) or not segments:
            segments.append(paragraph)
        else:
            segments[-1] = segments[-1] + "\n\n" + paragraph
    return segments


def _check_text_limits(record: dict[str, Any], failures: list[str]) -> None:
    text = str(record.get("text") or "")
    platform = str(record.get("platform"))
    piece = str(record.get("piece_id"))
    if len(_clean(text)) < MIN_TEXT:
        failures.append(f"{piece}: text shorter than {MIN_TEXT} characters")
    limit = PLATFORM_LIMITS.get(platform)
    if limit is None:
        failures.append(f"{piece}: unknown platform {platform}")
        return
    if platform == "twitter_x":
        segments = _segments(text)
        if len(segments) < 2:
            failures.append(f"{piece}: thread has fewer than 2 segments")
        for index, segment in enumerate(segments, start=1):
            if len(segment) > limit:
                failures.append(
                    f"{piece}: thread segment {index} has {len(segment)} > {limit} characters"
                )
    elif len(text) > limit:
        failures.append(f"{piece}: text has {len(text)} > {limit} characters")


def _check_thesis(record: dict[str, Any], thought: str, failures: list[str]) -> None:
    piece = str(record.get("piece_id"))
    essence = record.get("thought_essence") or {}
    thesis = _clean(str(record.get("topic") or ""))
    if not thesis:
        failures.append(f"{piece}: empty thesis")
        return
    if len(thesis) > MAX_THESIS:
        failures.append(f"{piece}: thesis longer than {MAX_THESIS} characters")
    if essence.get("role_map"):
        return  # hand-written thesis family
    haystack = _clean(thought).casefold()
    position = haystack.find(thesis.casefold())
    if position < 0:
        failures.append(f"{piece}: thesis is not a span of the thought")
        return
    after = haystack[position + len(thesis) : position + len(thesis) + 1]
    if after.isalnum():
        failures.append(f"{piece}: thesis ends inside a word")


def _check_visual(record: dict[str, Any], failures: list[str]) -> None:
    piece = str(record.get("piece_id"))
    job_plan = record.get("content_job_plan")
    if not isinstance(job_plan, dict) or "needs_visual" not in job_plan:
        failures.append(f"{piece}: content job plan missing")
        return
    if job_plan.get("needs_visual") is False:
        # The content-job classifier declined a visual for this draft; the reviewer sees
        # the reason in the job plan, so no asset is required.
        if not _clean(str(job_plan.get("reason") or "")):
            failures.append(f"{piece}: no-visual job plan lacks a reason")
        return
    spec = record.get("visual_spec") or {}
    path_key = (
        "rendered_pdf_path"
        if record.get("content_type") == "carousel_outline"
        else "rendered_image_path"
    )
    path = record.get(path_key)
    if not path or not Path(str(path)).is_file() or Path(str(path)).stat().st_size == 0:
        failures.append(f"{piece}: {path_key} missing or empty")
    hook = (
        spec.get("cover_hook")
        if record.get("content_type") == "carousel_outline"
        else spec.get("hook")
    )
    if not _clean(str(hook or "")):
        failures.append(f"{piece}: visual hook missing")
    if (
        record.get("content_type") == "carousel_outline"
        and len(spec.get("carousel_slides") or []) != 5
    ):
        failures.append(f"{piece}: carousel does not have 5 slides")
    if record.get("judge_verdict") not in {"PASS", "REDO"}:
        failures.append(f"{piece}: visual judge verdict missing")


def _check_case(case: dict[str, Any], content_set: Any, failures: list[str]) -> None:
    records = content_set.records
    package = content_set.package
    cid = case["id"]
    if len(records) != len(CHANNELS):
        failures.append(f"{cid}: expected {len(CHANNELS)} records, got {len(records)}")
    texts = [_clean(r.get("text")) for r in records]
    if len(set(texts)) != len(texts):
        failures.append(f"{cid}: drafts are not distinct across channels")
    approval = package.get("approval_workflow") or {}
    quality = package.get("quality_evaluation") or {}
    if approval.get("status") != "pending_review" or approval.get("approval_required") is not True:
        failures.append(f"{cid}: approval gate missing")
    checklist = approval.get("review_checklist") or []
    artifacts = {str(item.get("artifact")) for item in checklist}
    if not REQUIRED_CHECKLIST.issubset(artifacts):
        failures.append(f"{cid}: review checklist lacks {sorted(REQUIRED_CHECKLIST - artifacts)}")
    bad = [str(item.get("artifact")) for item in checklist if item.get("status") != "PASS"]
    if bad:
        failures.append(f"{cid}: checklist items not PASS: {bad}")
    if quality.get("ready_for_human_review") is not True:
        failures.append(f"{cid}: not ready for human review")
    for record in records:
        piece = str(record.get("piece_id"))
        destination = record.get("posting_destination") or {}
        if (
            destination.get("language") != case["language"]
            or destination.get("handle") != case["handle"]
        ):
            failures.append(
                f"{piece}: destination {destination.get('language')}/{destination.get('handle')} "
                f"!= {case['language']}/{case['handle']}"
            )
        fit = (record.get("quality") or {}).get("platform_fit") or {}
        if fit.get("verdict") != "PASS":
            failures.append(f"{piece}: platform fit {fit.get('verdict')}")
        if (record.get("quality") or {}).get("voice_check") != "PASS":
            failures.append(f"{piece}: voice check failed")
        _check_text_limits(record, failures)
        _check_thesis(record, case["thought"], failures)
        if record.get("content_type") in {"carousel_outline", "image_caption"}:
            _check_visual(record, failures)


async def _run(cases: list[dict[str, Any]]) -> dict[str, Any]:
    failures: list[str] = []
    errors: list[str] = []
    with tempfile.TemporaryDirectory(prefix="syncloom-e2e-") as tmp:
        root = Path(tmp)
        pipeline = ThoughtContentPipeline(
            queue_dir=root / "content-queue",
            rendered_dir=root / "rendered-content",
            lineage_dir=root / "lineage",
        )
        for case in cases:
            try:
                content_set = await pipeline.create_content_set(
                    thought=case["thought"], channels=list(CHANNELS), write_records=False
                )
            except Exception as exc:  # any crash is a whole-flow failure
                errors.append(f"{case['id']}: {type(exc).__name__}: {exc}")
                continue
            _check_case(case, content_set, failures)
    return {
        "passed": not failures and not errors,
        "cases": len(cases),
        "channels": CHANNELS,
        "failures": failures,
        "errors": errors,
        "component": os.environ.get("EVALLOOP_COMPONENT", ""),
    }


def main() -> int:
    cases = [
        json.loads(line)
        for line in (Path(__file__).resolve().parent / "cases.jsonl")
        .read_text(encoding="utf-8")
        .splitlines()
        if line.strip()
    ]
    report = asyncio.run(_run(cases))
    print(json.dumps(report, ensure_ascii=False))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
