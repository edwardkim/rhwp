#!/usr/bin/env python3
"""HWPX XML 에서 각주·미주 lineseg 를 추출한다. 실문서 바이트를 커밋하지 않는다.

정책연구용역 중간진도보고서 (#4882) 의 쪽수 드리프트는 각주 subList 의
후속 줄 vertpos=0 저장값이 재파싱에서 1172/2344 로 쌓이며 생긴다.
이 모듈은 그 저장 패턴을 ZIP/XML 에서 기계적으로 수집한다.
"""

from __future__ import annotations

import hashlib
import json
import re
import zipfile
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Iterable, Iterator

LINESEG_RE = re.compile(r"<hp:lineseg\b([^>]*)/?>", re.IGNORECASE)
ATTR_RE = re.compile(r'([A-Za-z0-9:]+)="([^"]*)"')
NOTE_OPEN_RE = re.compile(r"<(hp:)?(footNote|endNote)\b([^>]*)>", re.IGNORECASE)
NOTE_CLOSE_RE = re.compile(r"</(hp:)?(footNote|endNote)\s*>", re.IGNORECASE)
P_OPEN_RE = re.compile(r"<(hp:)?p\b([^>]*)>", re.IGNORECASE)
P_CLOSE_RE = re.compile(r"</(hp:)?p\s*>", re.IGNORECASE)
T_RE = re.compile(r"<(hp:)?t\b[^>]*>(.*?)</(hp:)?t\s*>", re.IGNORECASE | re.DOTALL)
TBL_OPEN_RE = re.compile(r"<(hp:)?tbl\b", re.IGNORECASE)
TBL_CLOSE_RE = re.compile(r"</(hp:)?tbl\s*>", re.IGNORECASE)
TC_OPEN_RE = re.compile(r"<(hp:)?tc\b", re.IGNORECASE)
TC_CLOSE_RE = re.compile(r"</(hp:)?tc\s*>", re.IGNORECASE)

NOTE_KINDS = ("footNote", "endNote")


@dataclass(frozen=True)
class LineSegRecord:
    textpos: int
    vertpos: int
    vertsize: int
    textheight: int
    baseline: int
    spacing: int
    horzpos: int
    horzsize: int
    flags: int

    @property
    def stacked_advance(self) -> int:
        return self.vertpos + self.vertsize + self.spacing

    def to_json(self) -> dict[str, int]:
        return asdict(self)


@dataclass
class NoteParagraph:
    para_index: int
    text: str
    segs: list[LineSegRecord] = field(default_factory=list)

    @property
    def vpos(self) -> list[int]:
        return [s.vertpos for s in self.segs]

    @property
    def all_zero_vpos(self) -> bool:
        return len(self.segs) > 1 and all(s.vertpos == 0 for s in self.segs)

    @property
    def trailing_zero_after_nonzero(self) -> bool:
        if len(self.segs) <= 1:
            return False
        return self.segs[0].vertpos != 0 and any(s.vertpos == 0 for s in self.segs[1:])

    def to_json(self) -> dict[str, Any]:
        return {
            "paraIndex": self.para_index,
            "text": self.text,
            "vpos": self.vpos,
            "allZeroVpos": self.all_zero_vpos,
            "hangulArtifact": self.trailing_zero_after_nonzero,
            "segs": [s.to_json() for s in self.segs],
        }


@dataclass
class NoteRecord:
    kind: str
    number: str
    inst_id: str
    in_table: bool
    table_depth: int
    xml_offset: int
    paragraphs: list[NoteParagraph] = field(default_factory=list)

    @property
    def path_hint(self) -> str:
        loc = "tbl." if self.in_table else ""
        return f"{loc}{self.kind}[instId={self.inst_id} number={self.number}]"

    @property
    def has_hwp5_zero_pattern(self) -> bool:
        return any(p.all_zero_vpos for p in self.paragraphs)

    def to_json(self) -> dict[str, Any]:
        return {
            "kind": self.kind,
            "number": self.number,
            "instId": self.inst_id,
            "inTable": self.in_table,
            "tableDepth": self.table_depth,
            "xmlOffset": self.xml_offset,
            "pathHint": self.path_hint,
            "hwp5ZeroPattern": self.has_hwp5_zero_pattern,
            "paragraphs": [p.to_json() for p in self.paragraphs],
        }


def parse_attrs(blob: str) -> dict[str, str]:
    return {m.group(1): m.group(2) for m in ATTR_RE.finditer(blob)}


def parse_int_attr(attrs: dict[str, str], name: str, default: int = 0) -> int:
    raw = attrs.get(name)
    if raw is None:
        return default
    try:
        return int(raw, 10)
    except ValueError:
        return default


def parse_lineseg_attrs(attr_blob: str) -> LineSegRecord:
    attrs = parse_attrs(attr_blob)
    return LineSegRecord(
        textpos=parse_int_attr(attrs, "textpos"),
        vertpos=parse_int_attr(attrs, "vertpos"),
        vertsize=parse_int_attr(attrs, "vertsize"),
        textheight=parse_int_attr(attrs, "textheight"),
        baseline=parse_int_attr(attrs, "baseline"),
        spacing=parse_int_attr(attrs, "spacing"),
        horzpos=parse_int_attr(attrs, "horzpos"),
        horzsize=parse_int_attr(attrs, "horzsize"),
        flags=parse_int_attr(attrs, "flags"),
    )


def decode_xml_text(raw: str) -> str:
    text = re.sub(r"<[^>]+>", "", raw)
    return (
        text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", '"')
        .replace("&apos;", "'")
        .replace("\n", " ")
        .strip()
    )


def extract_paragraphs_from_note_xml(body: str) -> list[NoteParagraph]:
    paras: list[NoteParagraph] = []
    # subList 안 문단만. 중첩 각주는 드물지만 같은 규칙으로 문단을 모은다.
    depth = 0
    start = None
    for match in re.finditer(r"</?hp:p\b[^>]*>", body):
        token = match.group(0)
        if token.startswith("</"):
            depth -= 1
            if depth == 0 and start is not None:
                chunk = body[start:match.end()]
                texts = [decode_xml_text(m.group(2)) for m in T_RE.finditer(chunk)]
                segs = [parse_lineseg_attrs(m.group(1)) for m in LINESEG_RE.finditer(chunk)]
                paras.append(
                    NoteParagraph(
                        para_index=len(paras),
                        text="".join(t for t in texts if t)[:240],
                        segs=segs,
                    )
                )
                start = None
        else:
            if depth == 0:
                start = match.start()
            depth += 1
    return paras


def iter_note_spans(xml: str) -> Iterator[tuple[str, dict[str, str], int, int]]:
    """(kind, attrs, body_start, body_end) — body 는 여는 태그 다음~닫는 태그 앞."""
    pos = 0
    while True:
        open_m = NOTE_OPEN_RE.search(xml, pos)
        if not open_m:
            return
        kind = open_m.group(2)
        attrs = parse_attrs(open_m.group(3) or "")
        depth = 1
        cursor = open_m.end()
        while depth > 0:
            nxt_open = NOTE_OPEN_RE.search(xml, cursor)
            nxt_close = NOTE_CLOSE_RE.search(xml, cursor)
            if nxt_close is None:
                return
            if nxt_open and nxt_open.start() < nxt_close.start():
                depth += 1
                cursor = nxt_open.end()
            else:
                depth -= 1
                if depth == 0:
                    yield kind, attrs, open_m.end(), nxt_close.start()
                    pos = nxt_close.end()
                    break
                cursor = nxt_close.end()


def table_depth_at(xml: str, offset: int) -> int:
    head = xml[:offset]
    opens = len(TBL_OPEN_RE.findall(head))
    closes = len(TBL_CLOSE_RE.findall(head))
    return max(0, opens - closes)


def extract_notes_from_section_xml(xml: str) -> list[NoteRecord]:
    notes: list[NoteRecord] = []
    for kind, attrs, start, end in iter_note_spans(xml):
        body = xml[start:end]
        depth = table_depth_at(xml, start)
        notes.append(
            NoteRecord(
                kind=kind,
                number=attrs.get("number", ""),
                inst_id=attrs.get("instId", ""),
                in_table=depth > 0,
                table_depth=depth,
                xml_offset=start,
                paragraphs=extract_paragraphs_from_note_xml(body),
            )
        )
    return notes


def hwpx_section_xml(path: Path, section: str = "Contents/section0.xml") -> str:
    with zipfile.ZipFile(path) as zf:
        return zf.read(section).decode("utf-8", errors="replace")


def hwpx_entry_names(path: Path) -> list[str]:
    with zipfile.ZipFile(path) as zf:
        return zf.namelist()


def sha256_file(path: Path, limit: int | None = None) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        remaining = limit
        while True:
            chunk = fh.read(1024 * 1024 if remaining is None else min(1024 * 1024, remaining))
            if not chunk:
                break
            h.update(chunk)
            if remaining is not None:
                remaining -= len(chunk)
                if remaining <= 0:
                    break
    return h.hexdigest()


def summarize_notes(notes: Iterable[NoteRecord]) -> dict[str, Any]:
    items = list(notes)
    zero = [n for n in items if n.has_hwp5_zero_pattern]
    artifact = [
        n
        for n in items
        if any(p.trailing_zero_after_nonzero for p in n.paragraphs)
    ]
    in_table = [n for n in items if n.in_table]
    return {
        "notes": len(items),
        "footnotes": sum(1 for n in items if n.kind.lower() == "footnote"),
        "endnotes": sum(1 for n in items if n.kind.lower() == "endnote"),
        "inTable": len(in_table),
        "hwp5ZeroPattern": len(zero),
        "hangulArtifact": len(artifact),
        "multiLine": sum(
            1 for n in items if any(len(p.segs) > 1 for p in n.paragraphs)
        ),
        "totalSegs": sum(len(p.segs) for n in items for p in n.paragraphs),
    }


def notes_to_json(notes: list[NoteRecord], *, source: str) -> dict[str, Any]:
    return {
        "schemaVersion": 1,
        "kind": "pageRoundtripNoteProbe",
        "source": source,
        "summary": summarize_notes(notes),
        "notes": [n.to_json() for n in notes],
    }
