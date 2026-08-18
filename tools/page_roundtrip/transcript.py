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
from typing import Any, Iterable

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

    def ir_diffs(self) -> list[str]:
        return [
            str(e.payload.get("path") or e.payload.get("text") or "")
            for e in self.events
            if e.kind == "ir_diff"
        ]

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": self.schema_version,
            "kind": self.kind,
            "issue": self.issue,
            "doc": self.doc,
            "route": self.route,
            "created": self.created,
            "summary": {
                "events": len(self.events),
                "pages": (
                    {"before": self.pages()[0], "after": self.pages()[1]}
                    if self.pages()
                    else None
                ),
                "verdict": self.verdict(),
                "irDiffs": len(self.ir_diffs()),
            },
            "events": [e.to_json() for e in self.events],
        }

    def to_jsonl(self) -> str:
        head = {
            "schemaVersion": self.schema_version,
            "kind": self.kind,
            "issue": self.issue,
            "doc": self.doc,
            "route": self.route,
            "created": self.created,
        }
        lines = [json.dumps(head, ensure_ascii=False)]
        for event in self.events:
            lines.append(json.dumps(event.to_json(), ensure_ascii=False))
        return "\n".join(lines) + "\n"


def now_utc() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def new_transcript(*, doc: str, route: str = "hwpx", issue: int | None = None) -> Transcript:
    t = Transcript(issue=issue, doc=doc, route=route, created=now_utc())
    t.add("meta", tool="tools/page_roundtrip/transcript.py", issue=issue)
    return t


def ingest_cli_text(transcript: Transcript, stdout: str, stderr: str, rc: int) -> Transcript:
    """export-hwpx --verify --verify-pages 텍스트를 사건으로 분해한다."""
    if stdout:
        transcript.add("stdout", text=stdout[-8000:], bytes=len(stdout.encode("utf-8", "replace")))
    if stderr:
        transcript.add("stderr", text=stderr[-8000:], bytes=len(stderr.encode("utf-8", "replace")))
    transcript.add("exit", rc=rc)
    combined = "\n".join(part for part in (stdout, stderr) if part)
    fail = VERIFY_FAIL_RE.search(combined)
    if fail:
        before, after = int(fail.group(1)), int(fail.group(2))
        transcript.add("pages", before=before, after=after, identical=before == after)
    else:
        passed = VERIFY_PASS_RE.search(combined)
        if passed:
            n = int(passed.group(1))
            transcript.add("pages", before=n, after=n, identical=True)
    count = IR_COUNT_RE.search(combined)
    if count:
        transcript.add("note", irDiffCount=int(count.group(1)))
    for match in IR_DIFF_RE.finditer(combined):
        transcript.add("ir_diff", path=match.group(1).strip())
    return transcript


def classify_from_transcript(transcript: Transcript, *, cataloged: bool) -> str:
    pages = transcript.pages()
    if pages is None:
        return "ERROR"
    equal = pages[0] == pages[1]
    if cataloged and equal:
        return "UNEXPECTED_PASS"
    if cataloged and not equal:
        return "EXPECTED_FAIL"
    if equal:
        return "MATCH"
    return "MISMATCH"


def load_jsonl(path: Path) -> Transcript:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines:
        raise ValueError(f"빈 전사: {path}")
    head = json.loads(lines[0])
    t = Transcript(
        schema_version=int(head.get("schemaVersion") or TRANSCRIPT_SCHEMA),
        kind=str(head.get("kind") or TRANSCRIPT_KIND),
        issue=head.get("issue"),
        doc=str(head.get("doc") or ""),
        route=str(head.get("route") or "hwpx"),
        created=str(head.get("created") or ""),
    )
    for line in lines[1:]:
        if not line.strip():
            continue
        obj = json.loads(line)
        kind = str(obj.get("kind") or "note")
        payload = {k: v for k, v in obj.items() if k not in {"seq", "kind"}}
        t.add(kind, **payload)
    return t


def write_jsonl(path: Path, transcript: Transcript) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(transcript.to_jsonl(), encoding="utf-8")


def write_json(path: Path, transcript: Transcript) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(transcript.to_json(), ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def iter_transcripts(root: Path) -> Iterable[Path]:
    if root.is_file():
        yield root
        return
    for path in sorted(root.rglob("*.jsonl")):
        yield path
