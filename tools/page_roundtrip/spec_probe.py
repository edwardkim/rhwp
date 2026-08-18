#!/usr/bin/env python3
"""한글 스펙문서 HWPX 의 저장 pagination 신호를 추출한다.

#5128 은 IR 이 같고 쪽수만 69→68 로 줄어든다. 원인은 HWP5-origin HWPX 가
원본 HWP5 저장 LINE_SEG·RowBreak 분할을 native 전용 게이트로 건너뛴 것이다.
이 모듈은 실문서 바이너리를 커밋하지 않고 ZIP/XML 에서 표·줄·쪽나눔을 수집한다.
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
P_OPEN_RE = re.compile(r"<(hp:)?p\b([^>]*)>", re.IGNORECASE)
P_CLOSE_RE = re.compile(r"</(hp:)?p\s*>", re.IGNORECASE)
T_RE = re.compile(r"<(hp:)?t\b[^>]*>(.*?)</(hp:)?t\s*>", re.IGNORECASE | re.DOTALL)
TBL_OPEN_RE = re.compile(r"<(hp:)?tbl\b([^>]*)>", re.IGNORECASE)
TBL_CLOSE_RE = re.compile(r"</(hp:)?tbl\s*>", re.IGNORECASE)
PIC_OPEN_RE = re.compile(r"<(hp:)?pic\b([^>]*)>", re.IGNORECASE)
COLPR_RE = re.compile(r"<(hp:)?colPr\b([^>]*)/?>", re.IGNORECASE)
PAGEHIDE_RE = re.compile(r"<(hp:)?pageHiding\b([^>]*)/?>", re.IGNORECASE)
PAGEBREAK_RE = re.compile(r'pageBreak="([^"]*)"', re.IGNORECASE)
TREAT_AS_CHAR_RE = re.compile(r'treatAsChar="([^"]*)"', re.IGNORECASE)
TEXTWRAP_RE = re.compile(r'textWrap="([^"]*)"', re.IGNORECASE)
HEIGHT_RE = re.compile(r'\bheight="([^"]*)"', re.IGNORECASE)
ROWCNT_RE = re.compile(r'rowCnt="([^"]*)"', re.IGNORECASE)
COLCNT_RE = re.compile(r'colCnt="([^"]*)"', re.IGNORECASE)
SEC0_NAMES = (
    "Contents/section0.xml",
    "Contents/section1.xml",
    "Contents/section2.xml",
    "Contents/section3.xml",
    "Contents/section4.xml",
    "Contents/section5.xml",
    "Contents/section6.xml",
    "Contents/section7.xml",
)

SPEC_SAMPLE_HWP = "samples/한글문서파일형식_5.0_revision1.3.hwp"
ISSUE_5128 = 5128
PINNED_PAGES = 69
PINNED_SECTIONS = 6
PINNED_PARAS = 619
PINNED_P015 = ("partialTable", 73)
PINNED_P016 = ("partialParagraph", 84)
PINNED_SPLIT_TABLES = (73, 174, 193, 203, 284, 343, 363, 380)


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
    def first_of_page(self) -> bool:
        return bool(self.flags & 1)

    @property
    def first_of_column(self) -> bool:
        return bool(self.flags & 2)

    @property
    def stacked_advance(self) -> int:
        return self.vertpos + self.vertsize + self.spacing

    def to_json(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["firstOfPage"] = self.first_of_page
        payload["firstOfColumn"] = self.first_of_column
        payload["stackedAdvance"] = self.stacked_advance
        return payload


def parse_int(raw: str | None, default: int = 0) -> int:
    if raw is None or raw == "":
        return default
    try:
        return int(raw, 10)
    except ValueError:
        try:
            return int(float(raw))
        except ValueError:
            return default


def parse_attrs(blob: str) -> dict[str, str]:
    return {m.group(1): m.group(2) for m in ATTR_RE.finditer(blob or "")}


def parse_lineseg_attrs(attr_blob: str) -> LineSegRecord:
    attrs = parse_attrs(attr_blob)
    return LineSegRecord(
        textpos=parse_int(attrs.get("textpos") or attrs.get("textPos")),
        vertpos=parse_int(attrs.get("vertpos") or attrs.get("vertPos")),
        vertsize=parse_int(attrs.get("vertsize") or attrs.get("vertSize")),
        textheight=parse_int(attrs.get("textheight") or attrs.get("textHeight")),
        baseline=parse_int(attrs.get("baseline") or attrs.get("baseLine")),
        spacing=parse_int(attrs.get("spacing")),
        horzpos=parse_int(attrs.get("horzpos") or attrs.get("horzPos")),
        horzsize=parse_int(attrs.get("horzsize") or attrs.get("horzSize")),
        flags=parse_int(attrs.get("flags")),
    )


def extract_text(xml: str) -> str:
    parts = [m.group(2) for m in T_RE.finditer(xml)]
    text = "".join(parts)
    return (
        text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", '"')
        .replace("&apos;", "'")
    )


def strip_inner_tables(xml: str) -> str:
    """중첩 표 본문을 걷어 바깥 문단 텍스트만 남긴다."""
    out: list[str] = []
    i = 0
    depth = 0
    for m in re.finditer(r"</?hp:tbl\b[^>]*>", xml, re.IGNORECASE):
        if depth == 0:
            out.append(xml[i : m.start()])
        tag = m.group(0)
        if tag.startswith("</"):
            depth = max(0, depth - 1)
        else:
            depth += 1
            if tag.endswith("/>") and depth > 0:
                depth -= 1
        i = m.end()
    if depth == 0:
        out.append(xml[i:])
    return "".join(out)


@dataclass
class TableRecord:
    para_index: int
    control_index: int
    row_cnt: int
    col_cnt: int
    page_break: str
    treat_as_char: bool
    text_wrap: str
    height: int
    xml_offset: int

    @property
    def is_rowbreak(self) -> bool:
        return self.page_break.lower() in {"row", "rowbreak", "TABLE"}

    def to_json(self) -> dict[str, Any]:
        return {
            "paraIndex": self.para_index,
            "controlIndex": self.control_index,
            "rowCnt": self.row_cnt,
            "colCnt": self.col_cnt,
            "pageBreak": self.page_break,
            "treatAsChar": self.treat_as_char,
            "textWrap": self.text_wrap,
            "height": self.height,
            "xmlOffset": self.xml_offset,
            "isRowBreak": self.is_rowbreak,
        }


@dataclass
class ParagraphRecord:
    section_index: int
    para_index: int
    text: str
    page_break: str
    segs: list[LineSegRecord] = field(default_factory=list)
    tables: list[TableRecord] = field(default_factory=list)
    picture_count: int = 0
    xml_offset: int = 0

    @property
    def vpos(self) -> list[int]:
        return [s.vertpos for s in self.segs]

    @property
    def has_page_first_seg(self) -> bool:
        return any(s.first_of_page for s in self.segs)

    @property
    def has_vpos_reset(self) -> bool:
        if len(self.segs) < 2:
            return False
        return any(
            self.segs[i].vertpos > 0 and self.segs[i + 1].vertpos == 0
            for i in range(len(self.segs) - 1)
        )

    def to_json(self) -> dict[str, Any]:
        return {
            "sectionIndex": self.section_index,
            "paraIndex": self.para_index,
            "text": self.text[:120],
            "pageBreak": self.page_break,
            "vpos": self.vpos,
            "hasPageFirstSeg": self.has_page_first_seg,
            "hasVposReset": self.has_vpos_reset,
            "pictureCount": self.picture_count,
            "tableCount": len(self.tables),
            "xmlOffset": self.xml_offset,
            "segs": [s.to_json() for s in self.segs],
            "tables": [t.to_json() for t in self.tables],
        }


def iter_top_level_paragraphs(xml: str) -> Iterator[tuple[int, str, str]]:
    """섹션 XML 의 최상위 <hp:p> 를 순회한다. 표 안 문단은 건너뛴다."""
    depth_tbl = 0
    i = 0
    para_idx = 0
    while i < len(xml):
        p_open = P_OPEN_RE.search(xml, i)
        tbl_open = TBL_OPEN_RE.search(xml, i)
        tbl_close = TBL_CLOSE_RE.search(xml, i)
        next_hits = [m for m in (p_open, tbl_open, tbl_close) if m]
        if not next_hits:
            break
        hit = min(next_hits, key=lambda m: m.start())
        if hit is tbl_open:
            depth_tbl += 1
            i = tbl_open.end()
            continue
        if hit is tbl_close:
            depth_tbl = max(0, depth_tbl - 1)
            i = tbl_close.end()
            continue
        # p open
        attrs = hit.group(2) or ""
        start = hit.start()
        # matching close at same table depth
        j = hit.end()
        depth_p = 1
        depth_t = depth_tbl
        while j < len(xml) and depth_p > 0:
            nxt_p_open = P_OPEN_RE.search(xml, j)
            nxt_p_close = P_CLOSE_RE.search(xml, j)
            nxt_t_open = TBL_OPEN_RE.search(xml, j)
            nxt_t_close = TBL_CLOSE_RE.search(xml, j)
            cands = [m for m in (nxt_p_open, nxt_p_close, nxt_t_open, nxt_t_close) if m]
            if not cands:
                break
            nxt = min(cands, key=lambda m: m.start())
            if nxt is nxt_t_open:
                depth_t += 1
                j = nxt.end()
                continue
            if nxt is nxt_t_close:
                depth_t = max(0, depth_t - 1)
                j = nxt.end()
                continue
            if nxt is nxt_p_open:
                depth_p += 1
                j = nxt.end()
                continue
            depth_p -= 1
            j = nxt.end()
            if depth_p != 0:
                continue
            body = xml[start:j]
            if depth_tbl == 0:
                yield para_idx, attrs, body
                para_idx += 1
            break
        else:
            i = hit.end()
            continue
        i = j


def extract_tables_from_para(para_xml: str, para_index: int) -> list[TableRecord]:
    tables: list[TableRecord] = []
    depth = 0
    start = -1
    current_attrs = ""
    ctrl = 0
    i = 0
    while i < len(para_xml):
        op = TBL_OPEN_RE.search(para_xml, i)
        cl = TBL_CLOSE_RE.search(para_xml, i)
        if op and (not cl or op.start() < cl.start()):
            if depth == 0:
                start = op.start()
                current_attrs = op.group(2) or ""
            depth += 1
            i = op.end()
            continue
        if cl:
            depth = max(0, depth - 1)
            if depth == 0 and start >= 0:
                block = para_xml[start : cl.end()]
                attrs = parse_attrs(current_attrs)
                row_m = ROWCNT_RE.search(block)
                col_m = COLCNT_RE.search(block)
                pb = PAGEBREAK_RE.search(block)
                tac = TREAT_AS_CHAR_RE.search(block)
                wrap = TEXTWRAP_RE.search(block)
                height = HEIGHT_RE.search(block)
                tables.append(
                    TableRecord(
                        para_index=para_index,
                        control_index=ctrl,
                        row_cnt=parse_int(attrs.get("rowCnt") or (row_m.group(1) if row_m else "0")),
                        col_cnt=parse_int(attrs.get("colCnt") or (col_m.group(1) if col_m else "0")),
                        page_break=(pb.group(1) if pb else attrs.get("pageBreak") or ""),
                        treat_as_char=(tac.group(1) if tac else attrs.get("treatAsChar") or "0")
                        in {"1", "true", "TRUE"},
                        text_wrap=(wrap.group(1) if wrap else attrs.get("textWrap") or ""),
                        height=parse_int(height.group(1) if height else attrs.get("height")),
                        xml_offset=start,
                    )
                )
                ctrl += 1
                start = -1
            i = cl.end()
            continue
        break
    return tables


def extract_paragraphs(section_xml: str, section_index: int) -> list[ParagraphRecord]:
    out: list[ParagraphRecord] = []
    for para_idx, attrs, body in iter_top_level_paragraphs(section_xml):
        parsed = parse_attrs(attrs)
        outer = strip_inner_tables(body)
        segs = [parse_lineseg_attrs(m.group(1)) for m in LINESEG_RE.finditer(outer)]
        rec = ParagraphRecord(
            section_index=section_index,
            para_index=para_idx,
            text=extract_text(outer),
            page_break=parsed.get("pageBreak") or "",
            segs=segs,
            tables=extract_tables_from_para(body, para_idx),
            picture_count=len(list(PIC_OPEN_RE.finditer(outer))),
            xml_offset=0,
        )
        out.append(rec)
    return out


def hwpx_entry_names(path: Path) -> list[str]:
    with zipfile.ZipFile(path) as zf:
        return zf.namelist()


def hwpx_section_xmls(path: Path) -> list[tuple[str, str]]:
    found: list[tuple[str, str]] = []
    with zipfile.ZipFile(path) as zf:
        names = zf.namelist()
        section_names = sorted(
            n for n in names if re.search(r"Contents/section\d+\.xml$", n.replace("\\", "/"))
        )
        if not section_names:
            section_names = [n for n in SEC0_NAMES if n in names]
        for name in section_names:
            found.append((name, zf.read(name).decode("utf-8")))
    return found


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def summarize_paragraphs(paragraphs: Iterable[ParagraphRecord]) -> dict[str, Any]:
    items = list(paragraphs)
    tables = [t for p in items for t in p.tables]
    return {
        "paragraphs": len(items),
        "tables": len(tables),
        "rowBreakTables": sum(1 for t in tables if t.is_rowbreak),
        "pageBreakParas": sum(1 for p in items if p.page_break.lower() in {"page", "column", "section"}),
        "vposResetParas": sum(1 for p in items if p.has_vpos_reset),
        "pageFirstSegs": sum(1 for p in items if p.has_page_first_seg),
        "pictures": sum(p.picture_count for p in items),
        "lineSegs": sum(len(p.segs) for p in items),
    }


def probe_hwpx(path: Path) -> dict[str, Any]:
    sections = hwpx_section_xmls(path)
    all_paras: list[ParagraphRecord] = []
    section_summaries: list[dict[str, Any]] = []
    for idx, (name, xml) in enumerate(sections):
        paras = extract_paragraphs(xml, idx)
        all_paras.extend(paras)
        summary = summarize_paragraphs(paras)
        summary["name"] = name
        summary["bytes"] = len(xml.encode("utf-8"))
        summary["sha256"] = hashlib.sha256(xml.encode("utf-8")).hexdigest()
        section_summaries.append(summary)
    return {
        "schemaVersion": 1,
        "kind": "issue5128SpecProbe",
        "source": str(path).replace("\\", "/"),
        "zipEntries": hwpx_entry_names(path),
        "sections": section_summaries,
        "summary": summarize_paragraphs(all_paras),
        "paragraphs": [p.to_json() for p in all_paras],
    }


def pinned_contract() -> dict[str, Any]:
    return {
        "issue": ISSUE_5128,
        "sample": SPEC_SAMPLE_HWP,
        "pages": PINNED_PAGES,
        "sections": PINNED_SECTIONS,
        "paragraphs": PINNED_PARAS,
        "p015": {"kind": PINNED_P015[0], "paraIndex": PINNED_P015[1]},
        "p016": {"kind": PINNED_P016[0], "paraIndex": PINNED_P016[1]},
        "splitTables": list(PINNED_SPLIT_TABLES),
    }
