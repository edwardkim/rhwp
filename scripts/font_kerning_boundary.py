#!/usr/bin/env python3
"""Freeze public GPOS/legacy kern capability boundaries for Issue #4968 W9-Q2."""

from __future__ import annotations

import argparse
from io import BytesIO
from pathlib import Path
from typing import Any

from fontTools.feaLib.builder import addOpenTypeFeaturesFromString
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont, newTable
from fontTools.ttLib.tables._k_e_r_n import KernTable_format_0

from oracle_stage2_common import (
    ROOT,
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    regular_input,
    sha256_bytes,
    write_json,
)


MAX_FONT_BYTES = 32 * 1024 * 1024
PUBLIC_FONT = "ttfs/opensource/NotoSansKR-Regular.ttf"
PUBLIC_FONT_SHA256 = "6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74"
PAIRS = ("AV", "To", "WA", "HH")


def _value_record(value: Any) -> dict[str, int]:
    fields = ("XPlacement", "YPlacement", "XAdvance", "YAdvance")
    return {field: int(getattr(value, field, 0) or 0) for field in fields}


def _add(left: dict[str, int], right: dict[str, int]) -> dict[str, int]:
    return {key: left[key] + right[key] for key in left}


def _zero_pair() -> dict[str, dict[str, int]]:
    zero = {field: 0 for field in ("XPlacement", "YPlacement", "XAdvance", "YAdvance")}
    return {"first": dict(zero), "second": dict(zero)}


def _script_feature_indices(gpos: Any, script_tag: str) -> list[int]:
    scripts = {record.ScriptTag: record.Script for record in gpos.ScriptList.ScriptRecord}
    script = scripts.get(script_tag) or scripts.get("DFLT")
    if script is None or script.DefaultLangSys is None:
        return []
    language = script.DefaultLangSys
    values = list(language.FeatureIndex)
    if language.ReqFeatureIndex != 0xFFFF:
        values.append(language.ReqFeatureIndex)
    return sorted(set(int(value) for value in values))


def _kern_lookup_indices(gpos: Any, script_tag: str) -> list[int]:
    allowed = set(_script_feature_indices(gpos, script_tag))
    lookups: list[int] = []
    for index, record in enumerate(gpos.FeatureList.FeatureRecord):
        if index in allowed and record.FeatureTag == "kern":
            lookups.extend(int(value) for value in record.Feature.LookupListIndex)
    return list(dict.fromkeys(lookups))


def _pair_pos_value(subtable: Any, first: str, second: str) -> dict[str, Any] | None:
    if getattr(subtable, "LookupType", None) == 2 and hasattr(subtable, "ExtSubTable"):
        subtable = subtable.ExtSubTable
    if type(subtable).__name__ != "PairPos" or first not in subtable.Coverage.glyphs:
        return None
    if subtable.Format == 1:
        pair_set = subtable.PairSet[subtable.Coverage.glyphs.index(first)]
        record = next(
            (item for item in pair_set.PairValueRecord if item.SecondGlyph == second), None
        )
        if record is None:
            return None
    elif subtable.Format == 2:
        class1 = subtable.ClassDef1.classDefs.get(first, 0)
        class2 = subtable.ClassDef2.classDefs.get(second, 0)
        record = subtable.Class1Record[class1].Class2Record[class2]
    else:
        return None
    return {
        "first": _value_record(record.Value1),
        "second": _value_record(record.Value2),
    }


def _gpos_pair(font: TTFont, first: str, second: str, script_tag: str) -> tuple[bool, dict[str, Any]]:
    if "GPOS" not in font:
        return False, _zero_pair()
    gpos = font["GPOS"].table
    lookups = _kern_lookup_indices(gpos, script_tag)
    if not lookups:
        return False, _zero_pair()
    total = _zero_pair()
    for lookup_index in lookups:
        lookup = gpos.LookupList.Lookup[lookup_index]
        for subtable in lookup.SubTable:
            value = _pair_pos_value(subtable, first, second)
            if value is not None:
                total["first"] = _add(total["first"], value["first"])
                total["second"] = _add(total["second"], value["second"])
                break
    return True, total


def _legacy_pair(font: TTFont, first: str, second: str) -> tuple[bool, int]:
    if "kern" not in font:
        return False, 0
    supported = False
    value = 0
    for subtable in font["kern"].kernTables:
        if not isinstance(subtable, KernTable_format_0) or not (subtable.coverage & 1):
            continue
        supported = True
        value += int(subtable.kernTable.get((first, second), 0))
    return supported, value


def analyze_font_bytes(
    data: bytes, *, pairs: tuple[str, ...] = PAIRS, script_tag: str = "latn"
) -> dict[str, Any]:
    if len(data) > MAX_FONT_BYTES:
        return {
            "status": "fail-closed",
            "capability": "unsupported",
            "fallbackReason": "font-byte-limit-exceeded",
            "bytes": len(data),
        }
    try:
        font = TTFont(BytesIO(data), lazy=False, recalcTimestamp=False)
    except Exception:
        return {
            "status": "fail-closed",
            "capability": "unsupported",
            "fallbackReason": "malformed-sfnt",
            "bytes": len(data),
        }
    try:
        cmap = font.getBestCmap() or {}
        glyph_pairs: list[tuple[str, str, str]] = []
        for pair in pairs:
            if len(pair) != 2 or any(ord(char) not in cmap for char in pair):
                glyph_pairs.append((pair, "", ""))
            else:
                glyph_pairs.append((pair, cmap[ord(pair[0])], cmap[ord(pair[1])]))

        gpos_supported = False
        for _, first, second in glyph_pairs:
            if first and _gpos_pair(font, first, second, script_tag)[0]:
                gpos_supported = True
                break
        legacy_supported = any(
            first and _legacy_pair(font, first, second)[0]
            for _, first, second in glyph_pairs
        )
        capability = (
            "gpos-kern" if gpos_supported else "legacy-kern" if legacy_supported else "unsupported"
        )
        results = []
        for pair, first, second in glyph_pairs:
            if not first:
                results.append(
                    {
                        "text": pair,
                        "disposition": "fail-closed",
                        "fallbackReason": "missing-cmap-glyph",
                        "adjustment": _zero_pair(),
                    }
                )
                continue
            if capability == "gpos-kern":
                _, adjustment = _gpos_pair(font, first, second, script_tag)
            elif capability == "legacy-kern":
                _, value = _legacy_pair(font, first, second)
                adjustment = _zero_pair()
                adjustment["first"]["XAdvance"] = value
            else:
                adjustment = _zero_pair()
            total_x_advance = adjustment["first"]["XAdvance"] + adjustment["second"]["XAdvance"]
            disposition = (
                "fail-closed"
                if capability == "unsupported"
                else "applied"
                if total_x_advance != 0
                else "no-pair-adjustment"
            )
            results.append(
                {
                    "text": pair,
                    "glyphs": [first, second],
                    "disposition": disposition,
                    "fallbackReason": (
                        "pair-table-unsupported" if capability == "unsupported" else None
                    ),
                    "adjustment": adjustment,
                    "totalXAdvance": total_x_advance,
                }
            )
        return {
            "status": "complete",
            "capability": capability,
            "fallbackReason": None,
            "bytes": len(data),
            "sha256": sha256_bytes(data),
            "unitsPerEm": int(font["head"].unitsPerEm),
            "tables": sorted(font.keys()),
            "script": script_tag,
            "pairs": results,
        }
    finally:
        font.close()


def synthetic_font(*, gpos: bool, legacy: bool) -> bytes:
    glyph_order = [".notdef", "space", "A", "V", "T", "o", "W", "H"]
    builder = FontBuilder(1000, isTTF=True)
    builder.setupGlyphOrder(glyph_order)
    glyphs = {}
    for name in glyph_order:
        pen = TTGlyphPen(None)
        if name not in {".notdef", "space"}:
            pen.moveTo((50, 0))
            pen.lineTo((550, 0))
            pen.lineTo((550, 700))
            pen.lineTo((50, 700))
            pen.closePath()
        glyphs[name] = pen.glyph()
    builder.setupGlyf(glyphs)
    builder.setupHorizontalMetrics({name: (600, 50) for name in glyph_order})
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupCharacterMap(
        {32: "space", 65: "A", 86: "V", 84: "T", 111: "o", 87: "W", 72: "H"}
    )
    builder.setupNameTable(
        {
            "familyName": "RHWP Kerning Boundary",
            "styleName": "Regular",
            "uniqueFontIdentifier": "rhwp-4968",
            "fullName": "RHWP Kerning Boundary Regular",
            "psName": "RHWP-Kerning-Boundary",
        }
    )
    builder.setupOS2(
        sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200
    )
    builder.setupPost()
    builder.setupMaxp()
    font = builder.font
    font.recalcTimestamp = False
    # OpenType longDateTime is seconds since 1904. Use the 1970 epoch in that
    # scale so fontTools does not reinterpret a deliberately fixed low value.
    font["head"].created = 2_082_844_800
    font["head"].modified = 2_082_844_800
    if gpos:
        addOpenTypeFeaturesFromString(
            font,
            "languagesystem DFLT dflt; languagesystem latn dflt; "
            "feature kern { pos A V -80; pos T o -40; } kern;",
        )
    if legacy:
        table = newTable("kern")
        table.version = 0
        subtable = KernTable_format_0()
        subtable.version = 0
        subtable.coverage = 1
        subtable.kernTable = {("A", "V"): -70, ("T", "o"): -30}
        table.kernTables = [subtable]
        font["kern"] = table
    stream = BytesIO()
    font.save(stream, reorderTables=False)
    return stream.getvalue()


def generate_boundary() -> dict[str, Any]:
    public_path = regular_input(ROOT, PUBLIC_FONT, MAX_FONT_BYTES)
    public_bytes = public_path.read_bytes()
    if sha256_bytes(public_bytes) != PUBLIC_FONT_SHA256:
        raise OracleStage2Error("public kerning font hash drift")
    public = analyze_font_bytes(public_bytes)
    cases = []
    for name, gpos, legacy in (
        ("gpos-only", True, False),
        ("legacy-only", False, True),
        ("gpos-and-legacy", True, True),
        ("no-pair-table", False, False),
    ):
        result = analyze_font_bytes(synthetic_font(gpos=gpos, legacy=legacy))
        cases.append({"case": name, **result})
    cases.append({"case": "malformed", **analyze_font_bytes(b"not-an-sfnt")})
    cases.append(
        {
            "case": "oversized",
            **analyze_font_bytes(bytes(MAX_FONT_BYTES + 1)),
        }
    )
    output = {
        "schemaVersion": 1,
        "issue": 4968,
        "stage": "W9-Q2",
        "kind": "kerning-capability-boundary",
        "status": "complete",
        "maximumFontBytes": MAX_FONT_BYTES,
        "precedence": ["gpos-kern", "legacy-kern", "unsupported"],
        "publicFont": {"path": PUBLIC_FONT, **public},
        "syntheticCases": cases,
        "fontBytesTracked": False,
    }
    output["canonicalSha256"] = sha256_bytes(canonical_json_bytes(output))
    return output


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--output", default="kerning_capability_boundary.json")
    args = parser.parse_args()
    output = generate_boundary()
    write_json(output_path(args.output_root, args.output), output, mode=0o644)
    print(output["canonicalSha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
