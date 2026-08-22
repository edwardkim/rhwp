#!/usr/bin/env python3
"""Bounded, deterministic PDF font/glyph/advance observation for Issue #4963."""

from __future__ import annotations

import argparse
import json
import math
import re
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any

from oracle_stage2_common import (
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    read_contract,
    regular_input,
    run_bounded,
    sha256_bytes,
    sha256_file,
    write_json,
)


def _first_line(value: bytes) -> str:
    lines = value.decode("utf-8", "replace").splitlines()
    return lines[0].strip() if lines else "unknown"


def _number(value: str | None, default: float = 0.0) -> float:
    try:
        result = float(value) if value is not None else default
    except ValueError as error:
        raise OracleStage2Error("PDF trace contains a non-numeric coordinate") from error
    if not math.isfinite(result):
        raise OracleStage2Error("PDF trace contains a non-finite coordinate")
    return result


def _round(value: float) -> float:
    return float(f"{value:.6f}")


def _matrix(value: str | None) -> tuple[float, float, float, float, float, float]:
    parts = (value or "1 0 0 1 0 0").split()
    if len(parts) == 4:
        parts.extend(("0", "0"))
    if len(parts) != 6:
        raise OracleStage2Error("PDF trace matrix must have four or six values")
    return tuple(_number(part) for part in parts)  # type: ignore[return-value]


def _transform_point(
    inner: tuple[float, float, float, float, float, float],
    outer: tuple[float, float, float, float, float, float],
    x: float,
    y: float,
) -> tuple[float, float]:
    ia, ib, ic, id_, ie, if_ = inner
    oa, ob, oc, od, oe, of = outer
    ix = ia * x + ic * y + ie
    iy = ib * x + id_ * y + if_
    return oa * ix + oc * iy + oe, ob * ix + od * iy + of


def _transform_vector(
    inner: tuple[float, float, float, float, float, float],
    outer: tuple[float, float, float, float, float, float],
    x: float,
    y: float,
) -> tuple[float, float]:
    ia, ib, ic, id_, _, _ = inner
    oa, ob, oc, od, _, _ = outer
    ix = ia * x + ic * y
    iy = ib * x + id_ * y
    return oa * ix + oc * iy, ob * ix + od * iy


def parse_pdffonts(value: bytes) -> list[dict[str, Any]]:
    lines = value.decode("utf-8", "replace").splitlines()
    fonts = []
    for line in lines[2:]:
        if not line.strip():
            continue
        match = re.match(
            r"^(\S+)\s{2,}(.+?)\s{2,}(\S+)\s+(yes|no)\s+(yes|no)\s+"
            r"(yes|no)\s+(\d+)\s+(\d+)$",
            line.strip(),
        )
        if match is None:
            raise OracleStage2Error("pdffonts output schema drift")
        (
            name,
            font_type,
            encoding,
            embedded,
            subset,
            unicode_map,
            object_number,
            generation_number,
        ) = match.groups()
        fonts.append(
            {
                "name": name,
                "type": font_type,
                "encoding": encoding,
                "embedded": embedded == "yes",
                "subset": subset == "yes",
                "unicodeMap": unicode_map == "yes",
                "objectId": {
                    "object": int(object_number),
                    "generation": int(generation_number),
                },
            }
        )
    return fonts


def parse_stext(value: bytes, maximum_glyphs: int) -> dict[str, Any]:
    try:
        payload = json.loads(value)
    except json.JSONDecodeError as error:
        raise OracleStage2Error("mutool stext JSON parse failed") from error
    pages = payload.get("pages")
    if not isinstance(pages, list):
        raise OracleStage2Error("mutool stext JSON lacks pages")
    spans = []
    visual_lines: set[tuple[int, float]] = set()
    for page_index, page in enumerate(pages, start=1):
        for block in page.get("blocks", []):
            if block.get("type") != "text":
                continue
            for line in block.get("lines", []):
                text = line.get("text", "")
                if not isinstance(text, str):
                    raise OracleStage2Error("mutool stext line text is invalid")
                x = _number(str(line.get("x", 0)))
                y = _number(str(line.get("y", 0)))
                bbox = line.get("bbox", {})
                font = line.get("font", {})
                spans.append(
                    {
                        "page": page_index,
                        "text": text,
                        "origin": {"x": _round(x), "y": _round(y)},
                        "bbox": {
                            key: _round(_number(str(bbox.get(key, 0))))
                            for key in ("x", "y", "w", "h")
                        },
                        "font": {
                            "name": str(font.get("name", "")),
                            "size": _round(_number(str(font.get("size", 0)))),
                            "weight": str(font.get("weight", "")),
                            "style": str(font.get("style", "")),
                        },
                    }
                )
                visual_lines.add((page_index, _round(y)))
                if len(spans) > maximum_glyphs:
                    raise OracleStage2Error("PDF text span limit exceeded")
    return {
        "pageCount": len(pages),
        "visualLineCount": len(visual_lines),
        "textSpanCount": len(spans),
        "textSpans": spans,
    }


def parse_trace(value: bytes, maximum_glyphs: int) -> list[dict[str, Any]]:
    try:
        # Hancom PDF can preserve a legacy Korean byte sequence in a subset
        # font name while MuPDF emits the rest of the trace as UTF-8. XML has
        # no legal mixed-encoding representation, so retain the structural and
        # numeric evidence and replace only undecodable name bytes. The raw PDF
        # identity remains separately fixed by inputSha256.
        root = ET.fromstring(value.decode("utf-8", "replace"))
    except ET.ParseError as error:
        raise OracleStage2Error("mutool trace XML parse failed") from error
    glyphs = []
    for page_index, page in enumerate(root.findall("page"), start=1):
        for fill in page.iter("fill_text"):
            outer = _matrix(fill.get("transform"))
            for span in fill.findall("span"):
                inner = _matrix(span.get("trm"))
                vertical = span.get("wmode") == "1"
                for glyph in span.findall("g"):
                    advance = _number(glyph.get("adv"))
                    x = _number(glyph.get("x"))
                    y = _number(glyph.get("y"))
                    # MuPDF trace reports g@x/y after the span text matrix but
                    # before the enclosing fill_text transform.  The span trm
                    # still scales the normalized advance vector.
                    position = _transform_point((1, 0, 0, 1, 0, 0), outer, x, y)
                    vector = _transform_vector(
                        inner,
                        outer,
                        0.0 if vertical else advance,
                        advance if vertical else 0.0,
                    )
                    glyphs.append(
                        {
                            "page": page_index,
                            "font": span.get("font", ""),
                            "unicode": glyph.get("unicode", ""),
                            "glyphName": glyph.get("glyph", ""),
                            "fontNormalizedAdvance": _round(advance),
                            "position": {"x": _round(position[0]), "y": _round(position[1])},
                            "pdfObservedAdvance": {
                                "dx": _round(vector[0]),
                                "dy": _round(vector[1]),
                                "distance": _round(math.hypot(*vector)),
                                "unit": "pdf-user-space",
                            },
                        }
                    )
                    if len(glyphs) > maximum_glyphs:
                        raise OracleStage2Error("PDF glyph observation limit exceeded")
    return glyphs


def analyze_pdf(
    *,
    contract: dict[str, Any],
    pdf_root: Path,
    relative_pdf: str,
) -> dict[str, Any]:
    policy = contract["pdfAnalysis"]
    pdf = regular_input(pdf_root, relative_pdf, policy["maximumBytes"])
    run_options = {
        "timeout_seconds": policy["timeoutSeconds"],
        "maximum_output_bytes": policy["maximumToolOutputBytes"],
    }
    run_bounded([policy["qpdf"], "--check", str(pdf)], **run_options)
    page_stdout, _ = run_bounded(
        [policy["qpdf"], "--show-npages", str(pdf)], **run_options
    )
    try:
        page_count = int(page_stdout.decode("ascii").strip())
    except ValueError as error:
        raise OracleStage2Error("qpdf page count is invalid") from error
    if page_count <= 0 or page_count > policy["maximumPages"]:
        raise OracleStage2Error("PDF page limit exceeded")

    xref_stdout, _ = run_bounded(
        [policy["qpdf"], "--show-xref", str(pdf)], **run_options
    )
    object_count = sum(
        bool(re.match(rb"^\d+/\d+:", line)) for line in xref_stdout.splitlines()
    )
    if object_count <= 0 or object_count > policy["maximumObjects"]:
        raise OracleStage2Error("PDF object limit exceeded")

    fonts_stdout, _ = run_bounded([policy["pdffonts"], str(pdf)], **run_options)
    stext_stdout, _ = run_bounded(
        [policy["mutool"], "draw", "-q", "-F", "stext.json", "-o", "-", str(pdf)],
        **run_options,
    )
    trace_stdout, _ = run_bounded(
        [policy["mutool"], "draw", "-q", "-F", "trace", "-o", "-", str(pdf)],
        **run_options,
    )
    stext = parse_stext(stext_stdout, policy["maximumGlyphs"])
    glyphs = parse_trace(trace_stdout, policy["maximumGlyphs"])
    if stext["pageCount"] != page_count:
        raise OracleStage2Error("qpdf and mutool page counts disagree")

    tool_versions = {}
    for name, command in {
        "qpdf": [policy["qpdf"], "--version"],
        "pdffonts": [policy["pdffonts"], "-v"],
        "mutool": [policy["mutool"], "-v"],
    }.items():
        stdout, stderr = run_bounded(command, **run_options)
        tool_versions[name] = _first_line(stdout or stderr)

    result = {
        "schemaVersion": 1,
        "kind": "font-oracle-pdf-observation",
        "issue": 4963,
        "inputSha256": sha256_file(pdf),
        "inputBytes": pdf.stat().st_size,
        "pageCount": page_count,
        "objectCount": object_count,
        "fonts": parse_pdffonts(fonts_stdout),
        "visualLineCount": stext["visualLineCount"],
        "textSpanCount": stext["textSpanCount"],
        "textSpans": stext["textSpans"],
        "glyphObservationCount": len(glyphs),
        "glyphObservations": glyphs,
        "toolVersions": tool_versions,
        "advanceSemantics": {
            "fontNormalizedAdvance": "mutool trace g@adv before text/PDF transforms",
            "pdfObservedAdvance": "advance vector after span trm and fill_text transform",
            "sfntHmtxIncluded": False,
        },
    }
    result["canonicalSha256"] = sha256_bytes(canonical_json_bytes(result))
    return result


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pdf-root", type=Path, required=True)
    parser.add_argument("--pdf", required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    result = analyze_pdf(
        contract=read_contract(), pdf_root=args.pdf_root, relative_pdf=args.pdf
    )
    write_json(output_path(args.output_root, args.output), result)
    print(result["canonicalSha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
