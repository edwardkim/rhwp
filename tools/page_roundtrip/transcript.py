#!/usr/bin/env python3
"""페이지 왕복 재현 전사(transcript). CLI 출력을 데이터로 남긴다.

판정은 데이터다. 전사는 한 줄이 한 사건(명령·stdout·stderr·종료코드·쪽수)이다.
침묵 스킵 금지 — 실패도 행으로 남긴다.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

TRANSCRIPT_KIND = "pageRoundtripTranscript"
TRANSCRIPT_SCHEMA = 1
EVENT_KINDS = (
    "meta",
    "command",
    "stdout",
    "stderr",
    "exit",
    "pages",
    "ir_diff",
    "note",
    "verdict",
    "catalog",
    "pageMap",
)

VERIFY_FAIL_RE = re.compile(
    r"검증 실패\(--verify-pages\):\s*변환 전\s*(\d+)\s*쪽,\s*재파싱 후\s*(\d+)\s*쪽"
)
VERIFY_PASS_RE = re.compile(r"검증 통과\(--verify-pages\):\s*(\d+)\s*쪽")
IR_DIFF_RE = re.compile(r"\[차이\]\s*(.+)")
IR_COUNT_RE = re.compile(r"IR 차이\s*(\d+)\s*건")


@dataclass
class TranscriptEvent:
    kind: str
    payload: dict[str, Any]
    seq: int = 0

    def to_json(self) -> dict[str, Any]:
        return {"seq": self.seq, "kind": self.kind, **self.payload}


@dataclass
class Transcript:
    schema_version: int = TRANSCRIPT_SCHEMA
    kind: str = TRANSCRIPT_KIND
    issue: int | None = None
    doc: str = ""
    route: str = "hwpx"
    created: str = ""
    events: list[TranscriptEvent] = field(default_factory=list)

    def add(self, kind: str, **payload: Any) -> TranscriptEvent:
        if kind not in EVENT_KINDS:
            raise ValueError(f"알 수 없는 transcript kind: {kind}")
        event = TranscriptEvent(kind=kind, payload=payload, seq=len(self.events))
        self.events.append(event)
        return event

    def pages(self) -> tuple[int, int] | None:
        for event in reversed(self.events):
            if event.kind == "pages":
                try:
                    return int(event.payload["before"]), int(event.payload["after"])
                except (KeyError, TypeError, ValueError):
                    return None
        return None

    def verdict(self) -> str:
        for event in reversed(self.events):
            if event.kind == "verdict":
                return str(event.payload.get("verdict") or "")
        return ""

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": self.schema_version,
            "kind": self.kind,
            "issue": self.issue,
            "doc": self.doc,
            "route": self.route,
            "created": self.created,
            "events": [e.to_json() for e in self.events],
        }


def new_transcript(*, issue: int | None, doc: str, route: str = "hwpx") -> Transcript:
    created = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    t = Transcript(issue=issue, doc=doc, route=route, created=created)
    t.add("meta", tool="tools/page_roundtrip/transcript.py", issue=issue)
    return t


def ingest_cli_text(transcript: Transcript, stdout: str, stderr: str, rc: int) -> Transcript:
    if stdout:
        transcript.add("stdout", text=stdout[:4000], bytes=len(stdout.encode("utf-8")))
    if stderr:
        transcript.add("stderr", text=stderr[:4000], bytes=len(stderr.encode("utf-8")))
    transcript.add("exit", rc=rc)
    fail = VERIFY_FAIL_RE.search("\n".join((stdout or "", stderr or "")))
    passed = VERIFY_PASS_RE.search("\n".join((stdout or "", stderr or "")))
    if fail:
        before, after = int(fail.group(1)), int(fail.group(2))
        transcript.add("pages", before=before, after=after, identical=False)
    elif passed:
        n = int(passed.group(1))
        transcript.add("pages", before=n, after=n, identical=True)
    for m in IR_DIFF_RE.finditer(stdout or ""):
        transcript.add("ir_diff", text=m.group(1))
    return transcript


def write_jsonl(path: Path, transcript: Transcript) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    header = {
        "schemaVersion": transcript.schema_version,
        "kind": transcript.kind,
        "issue": transcript.issue,
        "doc": transcript.doc,
        "route": transcript.route,
        "created": transcript.created,
    }
    lines = [json.dumps(header, ensure_ascii=False)]
    lines.extend(json.dumps(e.to_json(), ensure_ascii=False) for e in transcript.events)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_json(path: Path, transcript: Transcript) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(transcript.to_json(), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
