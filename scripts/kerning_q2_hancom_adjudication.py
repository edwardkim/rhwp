#!/usr/bin/env python3
"""Project local Hancom 2020 kerning evidence into a public Q2 adjudication."""

from __future__ import annotations

import argparse
import json
import re
import xml.etree.ElementTree as ET
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from oracle_stage2_common import (
    ROOT,
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    read_contract,
    read_json,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_json,
)
from pdf_oracle_observe import analyze_pdf


MANIFEST = Path(
    "mydocs/tech/investigations/issue-4968/fixtures/"
    "kerning_pair_fixture.manifest.json"
)
BOUNDARY = Path(
    "mydocs/tech/investigations/issue-4968/kerning_capability_boundary.json"
)
PAIR_SAMPLE = "AV To WA HH 가나다"
BODY_PATTERN = re.compile(
    r"^BODY R(100|90|80) S(0|-5|-10) K([01]) L(stored|fresh) \| "
    + re.escape(PAIR_SAMPLE)
    + r"$"
)
CONTEXT_PATTERN = re.compile(r"^(TABLE|TEXTBOX) K([01]) \| " + re.escape(PAIR_SAMPLE) + r"$")
LANGUAGES = ("Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User")
PAIR_INDEXES = {"AV": (0, 1), "To": (3, 4), "WA": (6, 7), "HH": (9, 10)}


def _round(value: float) -> float | int:
    rounded = float(f"{value:.6f}")
    return int(rounded) if rounded.is_integer() else rounded


def _checked_canonical(value: dict[str, Any], label: str) -> dict[str, Any]:
    claimed = value.get("canonicalSha256")
    if not isinstance(claimed, str):
        raise OracleStage2Error(f"{label} canonical digest is missing")
    payload = dict(value)
    del payload["canonicalSha256"]
    if sha256_bytes(canonical_json_bytes(payload)) != claimed:
        raise OracleStage2Error(f"{label} canonical digest mismatch")
    return value


def _direct_text(paragraph: ET.Element) -> tuple[str, str]:
    text_nodes = [child for child in paragraph if child.tag == "TEXT"]
    if len(text_nodes) != 1:
        raise OracleStage2Error("fixture paragraph must have one direct TEXT node")
    char_shape = text_nodes[0].get("CharShape")
    if char_shape is None:
        raise OracleStage2Error("fixture HWPML TEXT lacks CharShape")
    return "".join(text_nodes[0].itertext()), char_shape


def project_hwpml(path: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    try:
        root = ET.parse(path).getroot()
    except ET.ParseError as error:
        raise OracleStage2Error("HWPML2X readback is not valid XML") from error
    if root.tag != "HWPML":
        raise OracleStage2Error("HWPML2X root identity mismatch")

    faces: dict[str, dict[str, str]] = {}
    for group in root.iter("FONTFACE"):
        language = group.get("Lang")
        if language not in LANGUAGES:
            continue
        faces[language] = {
            str(font.get("Id")): str(font.get("Name")) for font in group.findall("FONT")
        }
    if set(faces) != set(LANGUAGES):
        raise OracleStage2Error("HWPML2X language font tables are incomplete")

    shapes: dict[str, dict[str, Any]] = {}
    for shape in root.iter("CHARSHAPE"):
        shape_id = shape.get("Id")
        ratio = shape.find("RATIO")
        spacing = shape.find("CHARSPACING")
        font_id = shape.find("FONTID")
        if shape_id is None or ratio is None or spacing is None or font_id is None:
            raise OracleStage2Error("HWPML2X CHARSHAPE is incomplete")
        flag = shape.get("UseKerning", "").lower()
        if flag not in {"true", "false"}:
            raise OracleStage2Error("HWPML2X UseKerning value is invalid")
        shapes[shape_id] = {
            "kerning": flag == "true",
            "ratio": {language: int(ratio.get(language, "-1")) for language in LANGUAGES},
            "spacing": {
                language: int(spacing.get(language, "-2147483648"))
                for language in LANGUAGES
            },
            "faces": {
                language: faces[language].get(str(font_id.get(language)))
                for language in LANGUAGES
            },
        }

    paragraphs = {
        str(paragraph.get("InstId")): paragraph
        for paragraph in root.iter("P")
        if paragraph.get("InstId") is not None
    }
    matrix = {
        int(row["charPropertyId"]): row for row in manifest["semantic"]["matrix"]
    }
    rows = []
    for context in manifest["semantic"]["contexts"]:
        paragraph_id = str(context["paragraphId"])
        paragraph = paragraphs.get(paragraph_id)
        if paragraph is None:
            raise OracleStage2Error("HWPML2X fixture paragraph is missing")
        text, shape_id = _direct_text(paragraph)
        shape = shapes.get(shape_id)
        expected = matrix[int(context["charPropertyId"])]
        if shape is None:
            raise OracleStage2Error("HWPML2X referenced CHARSHAPE is missing")
        if shape["kerning"] is not bool(expected["kerning"]):
            raise OracleStage2Error("HWPML2X kerning flag changed during open/readback")
        if set(shape["ratio"].values()) != {int(expected["ratio"])}:
            raise OracleStage2Error("HWPML2X ratio changed during open/readback")
        if set(shape["spacing"].values()) != {int(expected["spacing"])}:
            raise OracleStage2Error("HWPML2X spacing changed during open/readback")
        if set(shape["faces"].values()) != {manifest["semantic"]["documentFace"]}:
            raise OracleStage2Error("HWPML2X fixture face changed during open/readback")
        expected_label = "K1" if expected["kerning"] else "K0"
        if expected_label not in text or PAIR_SAMPLE not in text:
            raise OracleStage2Error("HWPML2X fixture text/flag label mismatch")
        rows.append(
            {
                "context": context["context"],
                "kerning": bool(expected["kerning"]),
                "ratio": int(expected["ratio"]),
                "spacing": int(expected["spacing"]),
            }
        )

    context_counts = Counter(row["context"] for row in rows)
    flag_counts = Counter("on" if row["kerning"] else "off" for row in rows)
    return {
        "formatVersion": str(root.get("Version", "")),
        "subVersion": str(root.get("SubVersion", "")),
        "charShapeCount": len(shapes),
        "fixtureContextCount": len(rows),
        "contextCounts": dict(sorted(context_counts.items())),
        "fixtureFlagCounts": dict(sorted(flag_counts.items())),
        "allFixtureFlagsPreserved": True,
        "allFixtureAxesPreserved": True,
        "allFixtureFacesExact": True,
    }


def _sample_projection(glyphs: list[dict[str, Any]], text: str) -> dict[str, Any]:
    start = text.index(PAIR_SAMPLE)
    sample = glyphs[start : start + len(PAIR_SAMPLE)]
    if "".join(str(glyph["unicode"]) for glyph in sample) != PAIR_SAMPLE:
        raise OracleStage2Error("PDF trace sample/glyph alignment mismatch")
    base = float(sample[0]["position"]["x"])
    positions = [_round(float(glyph["position"]["x"]) - base) for glyph in sample]
    pair_gaps = {
        pair: _round(positions[right] - positions[left])
        for pair, (left, right) in PAIR_INDEXES.items()
    }
    return {"positions": positions, "pairGaps": pair_gaps}


def project_pdf_observation(observation: dict[str, Any]) -> dict[str, Any]:
    grouped: dict[tuple[int, float], list[dict[str, Any]]] = defaultdict(list)
    for glyph in observation.get("glyphObservations", []):
        unicode_value = glyph.get("unicode")
        if not isinstance(unicode_value, str) or len(unicode_value) != 1:
            raise OracleStage2Error("PDF trace fixture glyph must map to one Unicode scalar")
        position = glyph.get("position", {})
        grouped[(int(glyph["page"]), float(position["y"]))].append(glyph)

    body_rows = []
    supplementary: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for _, glyphs in sorted(grouped.items()):
        glyphs.sort(key=lambda glyph: float(glyph["position"]["x"]))
        text = "".join(str(glyph["unicode"]) for glyph in glyphs)
        body = BODY_PATTERN.fullmatch(text)
        context = CONTEXT_PATTERN.fullmatch(text)
        if body is not None:
            body_rows.append(
                {
                    "ratio": int(body.group(1)),
                    "spacing": int(body.group(2)),
                    "kerning": body.group(3) == "1",
                    "lane": (
                        "stored-line-lane" if body.group(4) == "stored" else "fresh-candidate-lane"
                    ),
                    **_sample_projection(glyphs, text),
                }
            )
        elif context is not None:
            name = "table-cell" if context.group(1) == "TABLE" else "text-box"
            supplementary[name].append(
                {"kerning": context.group(2) == "1", **_sample_projection(glyphs, text)}
            )

    if len(body_rows) != 18:
        raise OracleStage2Error("PDF trace must contain 18 BODY kerning rows")
    groups: dict[tuple[int, int, str], list[dict[str, Any]]] = defaultdict(list)
    for row in body_rows:
        groups[(row["ratio"], row["spacing"], row["lane"])].append(row)
    if len(groups) != 9:
        raise OracleStage2Error("PDF trace must contain nine controlled BODY groups")

    projected_groups = []
    maximum_delta = 0.0
    for (ratio, spacing, lane), rows in sorted(
        groups.items(), key=lambda item: (-item[0][0], -item[0][1], item[0][2])
    ):
        if len(rows) != 2 or {row["kerning"] for row in rows} != {False, True}:
            raise OracleStage2Error("PDF trace controlled group lacks one K0/K1 row")
        off = next(row for row in rows if not row["kerning"])
        on = next(row for row in rows if row["kerning"])
        deltas = [_round(right - left) for left, right in zip(off["positions"], on["positions"])]
        pair_deltas = {
            pair: _round(on["pairGaps"][pair] - off["pairGaps"][pair])
            for pair in PAIR_INDEXES
        }
        group_maximum = max((abs(value) for value in deltas), default=0.0)
        maximum_delta = max(maximum_delta, group_maximum)
        projected_groups.append(
            {
                "ratio": ratio,
                "spacing": spacing,
                "lineSegLane": lane,
                "onOffPositionsEqual": group_maximum == 0.0,
                "maximumAbsoluteDelta": group_maximum,
                "pairGapDeltas": pair_deltas,
                "offPositionsSha256": sha256_bytes(canonical_json_bytes(off["positions"])),
                "onPositionsSha256": sha256_bytes(canonical_json_bytes(on["positions"])),
            }
        )

    supplementary_equal: dict[str, bool] = {}
    for name, rows in sorted(supplementary.items()):
        if len(rows) != 2 or {row["kerning"] for row in rows} != {False, True}:
            raise OracleStage2Error("PDF trace supplementary context lacks one K0/K1 row")
        off = next(row for row in rows if not row["kerning"])
        on = next(row for row in rows if row["kerning"])
        supplementary_equal[name] = off["positions"] == on["positions"]

    return {
        "bodyRunCount": len(body_rows),
        "controlledGroupCount": len(projected_groups),
        "allControlledOnOffPositionsEqual": all(
            group["onOffPositionsEqual"] for group in projected_groups
        ),
        "maximumAbsoluteOnOffDelta": _round(maximum_delta),
        "groups": projected_groups,
        "supplementaryContextOnOffPositionsEqual": supplementary_equal,
        "supplementaryContextsAreNonCausal": True,
    }


def generate_adjudication(
    *, evidence_root: Path, interactive_name: str, hwpml_name: str, pdf_name: str
) -> dict[str, Any]:
    interactive_path = regular_input(evidence_root, interactive_name, 1024 * 1024)
    hwpml_path = regular_input(evidence_root, hwpml_name, 4 * 1024 * 1024)
    pdf_path = regular_input(evidence_root, pdf_name, 32 * 1024 * 1024)
    interactive = read_json(interactive_path)
    manifest = read_json(regular_input(ROOT, MANIFEST.as_posix(), 1024 * 1024))
    boundary = _checked_canonical(
        read_json(regular_input(ROOT, BOUNDARY.as_posix(), 1024 * 1024)), "Q2 boundary"
    )
    if (
        interactive.get("issue") != 4968
        or interactive.get("status") != "observed"
        or interactive.get("inputSha256") != manifest.get("inputSha256")
        or interactive.get("documentFaceSelectable") is not True
    ):
        raise OracleStage2Error("interactive Q2 envelope identity mismatch")
    if interactive.get("hwpmlReadback", {}).get("sha256") != sha256_file(hwpml_path):
        raise OracleStage2Error("interactive/HWPML2X digest mismatch")
    if interactive.get("export", {}).get("pdfSha256") != sha256_file(pdf_path):
        raise OracleStage2Error("interactive/PDF digest mismatch")

    observation = analyze_pdf(
        contract=read_contract(), pdf_root=evidence_root, relative_pdf=pdf_name
    )
    if observation["inputSha256"] != interactive["export"]["pdfSha256"]:
        raise OracleStage2Error("PDF observation digest mismatch")
    readback = project_hwpml(hwpml_path, manifest)
    layout = project_pdf_observation(observation)
    no_differential = layout["allControlledOnOffPositionsEqual"]
    result = {
        "schemaVersion": 1,
        "kind": "kerning-q2-hancom-adjudication",
        "issue": 4968,
        "stage": "W9-Q2",
        "status": "complete",
        "inputs": {
            "fixtureSha256": manifest["inputSha256"],
            "fontSha256": manifest["semantic"]["fontSource"]["sha256"],
            "hwpmlReadbackSha256": sha256_file(hwpml_path),
            "pdfSha256": observation["inputSha256"],
        },
        "environment": {
            "hancomVersion": interactive["environment"]["hancomVersion"],
            "documentFaceSelectable": True,
            "embeddedFontNames": [font["name"] for font in observation["fonts"]],
        },
        "hwpmlReadback": readback,
        "pdfLayout": layout,
        "openTypeReference": {
            "capability": boundary["publicFont"]["capability"],
            "unitsPerEm": boundary["publicFont"]["unitsPerEm"],
            "pairDesignUnits": {
                row["text"]: row["totalXAdvance"] for row in boundary["publicFont"]["pairs"]
            },
        },
        "adjudication": {
            "featureFlagSurvivesOpen": readback["allFixtureFlagsPreserved"],
            "featureFlagCreatesPdfLayoutDifferential": not no_differential,
            "classification": (
                "flag-preserved-no-pdf-layout-differential"
                if no_differential
                else "flag-preserved-pdf-layout-differential"
            ),
            "applicationOrderStatus": (
                "not-observable-because-on-off-differential-is-zero"
                if no_differential
                else "observable"
            ),
            "implementationTruthSource": "opentype-capability-contract",
            "hancomObservationRole": "version-scoped-negative-compatibility-observation",
            "causalLimits": [
                "This observation is limited to Hancom 2020 11.0.0.9136 PDF export.",
                "Equal K0/K1 output does not prove that both paths are always-kerned or always-unkerned.",
                "Supplementary table/text-box pairs use different LineSeg lanes and are not causal gates.",
            ],
        },
        "privacy": {
            "privateCorpusAccessed": False,
            "absolutePathIncluded": False,
            "rawVmIdentityIncluded": False,
            "rawHwpmlIncluded": False,
            "rawPdfIncluded": False,
        },
        "nextGate": "revise-Q2-expected-truth-before-product-mutation",
    }
    result["canonicalSha256"] = sha256_bytes(canonical_json_bytes(result))
    return result


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-root", type=Path, required=True)
    parser.add_argument("--interactive", default="kerning-q2.interactive.json")
    parser.add_argument("--hwpml", default="kerning-q2.readback.hml")
    parser.add_argument("--pdf", default="kerning-q2.pdf")
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    result = generate_adjudication(
        evidence_root=args.evidence_root,
        interactive_name=args.interactive,
        hwpml_name=args.hwpml,
        pdf_name=args.pdf,
    )
    write_json(output_path(args.output_root, args.output), result, mode=0o644)
    print(result["canonicalSha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
