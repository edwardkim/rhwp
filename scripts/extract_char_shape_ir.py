#!/usr/bin/env python3
"""Extract HWP5/HWPX char_shapes IR dumps used as #3500 fixtures.

Reads compound-file HWP5 and ZIP HWPX samples, then writes compact JSONL
dumps of DocInfo CHAR_SHAPE tables and paragraph PARA_CHAR_SHAPE runs.
This is a fixture builder, not a production parser.
"""

from __future__ import annotations

import json
import struct
import sys
import zipfile
import zlib
from pathlib import Path
from xml.etree import ElementTree as ET

MAGIC = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"
ENDOFCHAIN = 0xFFFFFFFE
FREESECT = 0xFFFFFFFF
HWPTAG_BEGIN = 0x10
HWPTAG_CHAR_SHAPE = HWPTAG_BEGIN + 5
HWPTAG_PARA_HEADER = HWPTAG_BEGIN + 50
HWPTAG_PARA_TEXT = HWPTAG_BEGIN + 51
HWPTAG_PARA_CHAR_SHAPE = HWPTAG_BEGIN + 52
HWP_NS = {
    "hh": "http://www.hancom.co.kr/hwpml/2011/head",
    "hp": "http://www.hancom.co.kr/hwpml/2011/paragraph",
    "hs": "http://www.hancom.co.kr/hwpml/2011/section",
}


def u16(buf: bytes, off: int) -> int:
    return struct.unpack_from("<H", buf, off)[0]


def u32(buf: bytes, off: int) -> int:
    return struct.unpack_from("<I", buf, off)[0]


def i8(buf: bytes, off: int) -> int:
    return struct.unpack_from("<b", buf, off)[0]


def i32(buf: bytes, off: int) -> int:
    return struct.unpack_from("<i", buf, off)[0]


class Cfb:
    def __init__(self, data: bytes) -> None:
        if data[:8] != MAGIC:
            raise ValueError("not cfb")
        self.data = data
        self.sector_size = 1 << u16(data, 30)
        self.mini_sector_size = 1 << u16(data, 32)
        self.dir_start = u32(data, 48)
        self.mini_cutoff = u32(data, 56)
        self.mini_fat_start = u32(data, 60)
        self.difat_start = u32(data, 68)
        fat_ids = [u32(data, 76 + 4 * i) for i in range(109)]
        difat = self.difat_start
        while difat < 0xFFFFFFFA:
            base = 512 + difat * self.sector_size
            for i in range((self.sector_size // 4) - 1):
                fat_ids.append(u32(data, base + 4 * i))
            difat = u32(data, base + self.sector_size - 4)
        self.fat: list[int] = []
        for sid in fat_ids:
            if sid >= 0xFFFFFFFA:
                continue
            base = 512 + sid * self.sector_size
            for i in range(self.sector_size // 4):
                self.fat.append(u32(data, base + 4 * i))
        dir_bytes = self._read_chain(self.dir_start)
        self.entries = []
        for off in range(0, len(dir_bytes), 128):
            entry = dir_bytes[off : off + 128]
            if len(entry) < 128:
                break
            name_len = u16(entry, 64)
            name = entry[: max(name_len - 2, 0)].decode("utf-16le", "replace")
            self.entries.append(
                {
                    "name": name,
                    "type": entry[66],
                    "left": u32(entry, 68),
                    "right": u32(entry, 72),
                    "child": u32(entry, 76),
                    "start": u32(entry, 116),
                    "size": struct.unpack_from("<Q", entry, 120)[0],
                }
            )
        root = next(e for e in self.entries if e["type"] == 5)
        self.mini_fat: list[int] = []
        if self.mini_fat_start < 0xFFFFFFFA:
            mini = self._read_chain(self.mini_fat_start)
            for i in range(0, len(mini), 4):
                self.mini_fat.append(u32(mini, i))
        self.mini_store = (
            self._read_chain(root["start"]) if root["start"] < 0xFFFFFFFA else b""
        )

    def _read_chain(self, start: int) -> bytes:
        out = bytearray()
        sid = start
        seen: set[int] = set()
        while sid < 0xFFFFFFFA and sid not in seen:
            seen.add(sid)
            base = 512 + sid * self.sector_size
            out.extend(self.data[base : base + self.sector_size])
            if sid >= len(self.fat):
                break
            sid = self.fat[sid]
        return bytes(out)

    def _read_mini(self, start: int, size: int) -> bytes:
        out = bytearray()
        sid = start
        seen: set[int] = set()
        while sid < 0xFFFFFFFA and sid not in seen and len(out) < size:
            seen.add(sid)
            base = sid * self.mini_sector_size
            out.extend(self.mini_store[base : base + self.mini_sector_size])
            if sid >= len(self.mini_fat):
                break
            sid = self.mini_fat[sid]
        return bytes(out[:size])

    def stream(self, path: str) -> bytes | None:
        parts = [p for p in path.replace("\\", "/").split("/") if p]
        idx = 0
        child = next((e["child"] for e in self.entries if e["type"] == 5), 0xFFFFFFFF)
        for part in parts:
            found = None
            stack = [child]
            seen: set[int] = set()
            while stack:
                i = stack.pop()
                if i >= len(self.entries) or i in seen:
                    continue
                seen.add(i)
                e = self.entries[i]
                if e["name"].casefold() == part.casefold():
                    found = e
                    break
                if e["left"] < 0xFFFFFFFA:
                    stack.append(e["left"])
                if e["right"] < 0xFFFFFFFA:
                    stack.append(e["right"])
            if found is None:
                return None
            if part is parts[-1]:
                if found["type"] != 2:
                    return None
                if found["size"] < self.mini_cutoff:
                    return self._read_mini(found["start"], found["size"])
                return self._read_chain(found["start"])[: found["size"]]
            child = found["child"]
        return None

    def list_streams(self, prefix: str = "") -> list[str]:
        names: list[str] = []

        def walk(idx: int, path: str) -> None:
            if idx >= len(self.entries):
                return
            e = self.entries[idx]
            if e["left"] < 0xFFFFFFFA:
                walk(e["left"], path)
            here = f"{path}/{e['name']}" if path else e["name"]
            if e["type"] == 2:
                names.append(here)
            if e["type"] == 1 and e["child"] < 0xFFFFFFFA:
                walk(e["child"], here)
            if e["right"] < 0xFFFFFFFA:
                walk(e["right"], path)

        root = next(e for e in self.entries if e["type"] == 5)
        if root["child"] < 0xFFFFFFFA:
            walk(root["child"], prefix)
        return names


def iter_records(data: bytes):
    pos = 0
    n = len(data)
    while pos + 4 <= n:
        header = u32(data, pos)
        pos += 4
        tag = header & 0x3FF
        level = (header >> 10) & 0x3FF
        size = header >> 20
        if size == 0xFFF:
            if pos + 4 > n:
                break
            size = u32(data, pos)
            pos += 4
        payload = data[pos : pos + size]
        pos += size
        yield tag, level, payload


def decode_char_shape(payload: bytes) -> dict:
    if len(payload) < 46:
        return {"raw_len": len(payload), "truncated": True}
    font_ids = list(struct.unpack_from("<7H", payload, 0))
    ratios = list(payload[14:21])
    spacings = [i8(payload, 21 + i) for i in range(7)]
    rel = list(payload[28:35])
    offsets = [i8(payload, 35 + i) for i in range(7)]
    base = i32(payload, 42)
    attr = u32(payload, 46) if len(payload) >= 50 else 0
    sx = i8(payload, 50) if len(payload) >= 51 else 0
    sy = i8(payload, 51) if len(payload) >= 52 else 0
    colors = []
    off = 52
    for _ in range(4):
        if off + 4 <= len(payload):
            colors.append(u32(payload, off))
        else:
            colors.append(0)
        off += 4
    border = u16(payload, 68) if len(payload) >= 70 else 0
    strike = u32(payload, 70) if len(payload) >= 74 else 0
    return {
        "font_ids": font_ids,
        "ratios": ratios,
        "spacings": spacings,
        "relative_sizes": rel,
        "char_offsets": offsets,
        "base_size": base,
        "attr": attr,
        "italic": bool(attr & 1),
        "bold": bool(attr >> 1 & 1),
        "underline": attr >> 2 & 3,
        "underline_shape": attr >> 4 & 15,
        "outline": attr >> 8 & 7,
        "shadow": attr >> 11 & 3,
        "emboss": bool(attr >> 13 & 1),
        "engrave": bool(attr >> 14 & 1),
        "super": bool(attr >> 15 & 1),
        "sub": bool(attr >> 16 & 1),
        "emphasis": attr >> 21 & 15,
        "use_font_space": bool(attr >> 25 & 1),
        "strike_shape": attr >> 26 & 15,
        "kerning": bool(attr >> 30 & 1),
        "shadow_offset": [sx, sy],
        "text_color": colors[0],
        "underline_color": colors[1],
        "shade_color": colors[2],
        "shadow_color": colors[3],
        "border_fill_id": border,
        "strike_color": strike,
        "payload_len": len(payload),
    }


def decode_para_char_shape(payload: bytes) -> list[list[int]]:
    out = []
    for off in range(0, len(payload) - 7, 8):
        out.append([u32(payload, off), u32(payload, off + 4)])
    return out


def maybe_decompress(raw: bytes, compressed: bool) -> bytes:
    if not compressed or not raw:
        return raw
    try:
        return zlib.decompress(raw, -15)
    except zlib.error:
        try:
            return zlib.decompress(raw)
        except zlib.error:
            return raw


def file_header_compressed(cfb: Cfb) -> bool:
    header = cfb.stream("FileHeader") or b""
    if len(header) < 40:
        return False
    flags = u32(header, 36)
    return bool(flags & 0x01)


def extract_hwp5(path: Path) -> dict:
    data = path.read_bytes()
    cfb = Cfb(data)
    compressed = file_header_compressed(cfb)
    doc_info = maybe_decompress(cfb.stream("DocInfo") or b"", compressed)
    shapes = [decode_char_shape(p) for t, _, p in iter_records(doc_info) if t == HWPTAG_CHAR_SHAPE]
    paragraphs = []
    streams = [n for n in cfb.list_streams() if n.replace("\\", "/").upper().startswith("BODYTEXT/")]
    streams.sort()
    for si, name in enumerate(streams):
        raw = maybe_decompress(cfb.stream(name) or b"", compressed)
        para_i = -1
        pending_text_len = 0
        for tag, level, payload in iter_records(raw):
            if tag == HWPTAG_PARA_HEADER and level == 0:
                para_i += 1
                pending_text_len = 0
            elif tag == HWPTAG_PARA_TEXT and level == 1:
                pending_text_len = len(payload) // 2
            elif tag == HWPTAG_PARA_CHAR_SHAPE and level == 1:
                refs = decode_para_char_shape(payload)
                if refs:
                    paragraphs.append(
                        {
                            "section": si,
                            "para": para_i,
                            "text_units": pending_text_len,
                            "refs": refs,
                        }
                    )
    return {
        "file": path.name,
        "kind": "hwp5",
        "char_shape_count": len(shapes),
        "char_shapes": shapes,
        "paragraphs": paragraphs,
    }


def local(tag: str) -> str:
    if "}" in tag:
        return tag.rsplit("}", 1)[1]
    if ":" in tag:
        return tag.split(":", 1)[1]
    return tag


def extract_hwpx(path: Path) -> dict:
    shapes = []
    paragraphs = []
    with zipfile.ZipFile(path) as zf:
        names = zf.namelist()
        header_name = next((n for n in names if n.lower().endswith("header.xml")), None)
        if header_name:
            root = ET.fromstring(zf.read(header_name))
            for i, pr in enumerate(root.iter()):
                if local(pr.tag) != "charPr":
                    continue
                lang = {}
                for child in list(pr):
                    ln = local(child.tag)
                    if ln in {"fontRef", "ratio", "spacing", "relSz", "offset"}:
                        lang[ln] = {k.split("}")[-1]: v for k, v in child.attrib.items()}
                shapes.append(
                    {
                        "id": pr.attrib.get("id", str(i)),
                        "height": pr.attrib.get("height"),
                        "textColor": pr.attrib.get("textColor"),
                        "shadeColor": pr.attrib.get("shadeColor"),
                        "useFontSpace": pr.attrib.get("useFontSpace"),
                        "useKerning": pr.attrib.get("useKerning"),
                        "symMark": pr.attrib.get("symMark"),
                        "borderFillIDRef": pr.attrib.get("borderFillIDRef"),
                        "lang": lang,
                    }
                )
        section_names = sorted(
            n for n in names if "/section" in n.replace("\\", "/").lower() and n.lower().endswith(".xml")
        )
        for si, sname in enumerate(section_names):
            try:
                root = ET.fromstring(zf.read(sname))
            except ET.ParseError:
                continue
            for pi, p in enumerate(root.iter()):
                if local(p.tag) != "p":
                    continue
                refs = []
                pos = 0
                for child in list(p):
                    if local(child.tag) != "run":
                        continue
                    cid = int(child.attrib.get("charPrIDRef", "0") or 0)
                    refs.append([pos, cid])
                    text_len = 0
                    for node in child.iter():
                        if node.text:
                            text_len += len(node.text)
                    pos += text_len
                if refs:
                    paragraphs.append(
                        {
                            "section": si,
                            "para": pi,
                            "text_units": pos,
                            "refs": refs,
                        }
                    )
    return {
        "file": path.name,
        "kind": "hwpx",
        "char_shape_count": len(shapes),
        "char_shapes": shapes,
        "paragraphs": paragraphs,
    }


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    samples = root / "samples"
    out_dir = root / "tests" / "fixtures" / "char_shapes"
    out_dir.mkdir(parents=True, exist_ok=True)
    files = sorted(samples.rglob("*.hwp")) + sorted(samples.rglob("*.hwpx"))
    para_rows = []
    shape_rows = []
    target = None
    errors = []
    for path in files:
        rel = path.relative_to(samples).as_posix()
        try:
            if path.suffix.lower() == ".hwp":
                rec = extract_hwp5(path)
            else:
                rec = extract_hwpx(path)
        except Exception as exc:  # noqa: BLE001
            errors.append({"file": rel, "error": str(exc)})
            continue
        rec["file"] = rel
        if "re-multisize-10-10-empty-hancom.hwp" in rel:
            target = rec
        same_id_paras = []
        for p in rec["paragraphs"]:
            refs = p["refs"]
            collapsed = []
            for start, sid in refs:
                if not collapsed or collapsed[-1][1] != sid:
                    collapsed.append([start, sid])
            if len(collapsed) != len(refs):
                same_id_paras.append(p)
            para_rows.append(
                {
                    "file": rel,
                    "kind": rec["kind"],
                    "section": p["section"],
                    "para": p["para"],
                    "text_units": p["text_units"],
                    "refs": refs,
                    "same_id_extra": len(refs) - len(collapsed),
                }
            )
        shape_rows.append(
            {
                "file": rel,
                "kind": rec["kind"],
                "count": rec["char_shape_count"],
                "char_shapes": rec["char_shapes"],
            }
        )
        rec["same_id_paragraphs"] = same_id_paras
    same_id_rows = [r for r in para_rows if r["same_id_extra"] > 0]
    compact_shapes = []
    for row in shape_rows:
        compact = []
        for i, cs in enumerate(row["char_shapes"][:48]):
            if row["kind"] == "hwp5" and not cs.get("truncated"):
                compact.append(
                    {
                        "id": i,
                        "base_size": cs.get("base_size"),
                        "attr": cs.get("attr"),
                        "font_ids": cs.get("font_ids"),
                        "text_color": cs.get("text_color"),
                        "shade_color": cs.get("shade_color"),
                        "italic": cs.get("italic"),
                        "bold": cs.get("bold"),
                    }
                )
            elif row["kind"] == "hwpx":
                compact.append(
                    {
                        "id": cs.get("id"),
                        "height": cs.get("height"),
                        "textColor": cs.get("textColor"),
                        "shadeColor": cs.get("shadeColor"),
                    }
                )
        compact_shapes.append(
            {"file": row["file"], "kind": row["kind"], "count": row["count"], "preview": compact}
        )
    (out_dir / "corpus_same_id_para_char_shapes.jsonl").write_text(
        "\n".join(json.dumps(r, ensure_ascii=False, separators=(",", ":")) for r in same_id_rows)
        + ("\n" if same_id_rows else ""),
        encoding="utf-8",
    )
    (out_dir / "corpus_char_shape_tables.jsonl").write_text(
        "\n".join(json.dumps(r, ensure_ascii=False, separators=(",", ":")) for r in compact_shapes)
        + ("\n" if compact_shapes else ""),
        encoding="utf-8",
    )
    summary = {
        "sample_files": len(files),
        "ok_files": len(shape_rows),
        "errors": errors[:40],
        "error_count": len(errors),
        "paragraph_rows": len(para_rows),
        "same_id_rows": len(same_id_rows),
        "target": target,
    }
    (out_dir / "extract_summary.json").write_text(
        json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    if target:
        (out_dir / "issue_3500_re_multisize.json").write_text(
            json.dumps(target, ensure_ascii=False, indent=2), encoding="utf-8"
        )
    print(
        json.dumps(
            {
                "files": len(files),
                "ok": len(shape_rows),
                "paras": len(para_rows),
                "same_id": summary["same_id_rows"],
                "errors": len(errors),
                "target_paras": None if not target else len(target["paragraphs"]),
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
