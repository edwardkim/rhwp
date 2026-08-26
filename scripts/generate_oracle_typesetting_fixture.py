#!/usr/bin/env python3
"""Generate the deterministic public HWPX fixture for Issue #4963 W5."""

from __future__ import annotations

import argparse
import html
import itertools
import re
import zipfile
from pathlib import Path
from typing import Any

from oracle_stage2_common import (
    ROOT,
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    read_contract,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_json,
)


ZIP_TIMESTAMP = (1980, 1, 1, 0, 0, 0)


def replace_once(value: str, before: str, after: str, label: str) -> str:
    if value.count(before) != 1:
        raise OracleStage2Error(f"{label} marker must occur exactly once")
    return value.replace(before, after, 1)


def zip_info(name: str, compression: int) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(name, ZIP_TIMESTAMP)
    info.compress_type = compression
    info.create_system = 3
    info.external_attr = 0o100644 << 16
    return info


def lineseg(text_length: int, vertical_position: int = 0, width: int = 42520) -> str:
    return (
        '<hp:linesegarray><hp:lineseg textpos="0" '
        f'vertpos="{vertical_position}" vertsize="1000" textheight="1000" '
        f'baseline="850" spacing="600" horzpos="0" horzsize="{width}" '
        f'flags="393216" data-text-length="{text_length}"/></hp:linesegarray>'
    )


def paragraph(
    paragraph_id: int,
    char_property_id: int,
    text: str,
    *,
    stored_line_seg: bool,
    width: int = 42520,
) -> str:
    escaped = html.escape(text, quote=False)
    lane = "stored-line-lane" if stored_line_seg else "fresh-candidate-lane"
    line = lineseg(len(text), width=width) if stored_line_seg else ""
    return (
        f'<hp:p id="{paragraph_id}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
        f'columnBreak="0" merged="0" data-oracle-lane="{lane}">'
        f'<hp:run charPrIDRef="{char_property_id}"><hp:t>{escaped}</hp:t></hp:run>'
        f"{line}</hp:p>"
    )


def table_host(paragraph_id: int, cell_paragraphs: str) -> str:
    cell = (
        '<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" '
        'borderFillIDRef="2"><hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" '
        'vertAlign="CENTER" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" '
        f'textHeight="0" hasTextRef="0" hasNumRef="0">{cell_paragraphs}</hp:subList>'
        '<hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
        '<hp:cellSz width="30000" height="5400"/>'
        '<hp:cellMargin left="510" right="510" top="141" bottom="141"/></hp:tc>'
    )
    table = (
        '<hp:tbl id="49630001" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" '
        'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" '
        'rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
        '<hp:sz width="30000" widthRelTo="ABSOLUTE" height="5400" heightRelTo="ABSOLUTE" '
        'protect="0"/><hp:pos treatAsChar="1" affectLSpacing="1" flowWithText="1" '
        'allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="COLUMN" '
        'vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
        '<hp:outMargin left="283" right="283" top="283" bottom="283"/>'
        '<hp:inMargin left="510" right="510" top="141" bottom="141"/>'
        f"<hp:tr>{cell}</hp:tr></hp:tbl>"
    )
    return (
        f'<hp:p id="{paragraph_id}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
        'columnBreak="0" merged="0" data-oracle-context="table-cell">'
        f'<hp:run charPrIDRef="0">{table}</hp:run>{lineseg(1)}</hp:p>'
    )


def text_box_host(paragraph_id: int, box_paragraphs: str) -> str:
    rectangle = (
        '<hp:rect id="49630002" zOrder="1" numberingType="PICTURE" textWrap="IN_FRONT_OF_TEXT" '
        'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" href="" groupLevel="0" '
        'instid="49630002" ratio="0"><hp:offset x="0" y="0"/>'
        '<hp:orgSz width="30000" height="5400"/><hp:curSz width="30000" height="5400"/>'
        '<hp:flip horizontal="0" vertical="0"/>'
        '<hp:rotationInfo angle="0" centerX="15000" centerY="2700" rotateimage="1"/>'
        '<hp:renderingInfo><hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
        '<hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
        '<hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
        '</hp:renderingInfo><hp:lineShape color="#000000" width="33" style="SOLID" '
        'endCap="FLAT" headStyle="NORMAL" tailStyle="NORMAL" headfill="1" tailfill="1" '
        'headSz="MEDIUM_MEDIUM" tailSz="MEDIUM_MEDIUM" outlineStyle="NORMAL" alpha="0"/>'
        '<hc:fillBrush><hc:winBrush faceColor="#FFFFFF" hatchColor="#000000" alpha="0"/>'
        '</hc:fillBrush><hp:shadow type="NONE" color="#B2B2B2" offsetX="0" offsetY="0" '
        'alpha="0"/><hp:drawText lastWidth="29434" name="W5 Oracle text box" editable="0">'
        '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" '
        'linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" '
        f'hasTextRef="0" hasNumRef="0">{box_paragraphs}</hp:subList>'
        '<hp:textMargin left="283" right="283" top="283" bottom="283"/></hp:drawText>'
        '<hc:pt0 x="0" y="0"/><hc:pt1 x="30000" y="0"/><hc:pt2 x="30000" y="5400"/>'
        '<hc:pt3 x="0" y="5400"/><hp:sz width="30000" widthRelTo="ABSOLUTE" height="5400" '
        'heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="1" affectLSpacing="1" '
        'flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" '
        'horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
        '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
        '<hp:shapeComment>W5 Oracle synthetic text box</hp:shapeComment></hp:rect>'
    )
    return (
        f'<hp:p id="{paragraph_id}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
        'columnBreak="0" merged="0" data-oracle-context="text-box">'
        f'<hp:run charPrIDRef="0">{rectangle}</hp:run>{lineseg(1)}</hp:p>'
    )


def header_or_footer(kind: str, control_id: int, char_property_id: int, text: str) -> str:
    vertical = "TOP" if kind == "header" else "BOTTOM"
    escaped = html.escape(text, quote=False)
    return (
        f'<hp:ctrl><hp:{kind} id="{control_id}" applyPageType="BOTH">'
        f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="{vertical}" '
        'linkListIDRef="0" linkListNextIDRef="0" textWidth="42520" textHeight="1800" '
        'hasTextRef="0" hasNumRef="0">'
        f'<hp:p id="{control_id}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
        f'columnBreak="0" merged="0" data-oracle-context="{kind}">'
        f'<hp:run charPrIDRef="{char_property_id}"><hp:t>{escaped}</hp:t></hp:run>'
        f'{lineseg(len(text))}</hp:p></hp:subList></hp:{kind}></hp:ctrl>'
    )


def build_char_properties(
    header: str,
    face: str,
    matrix: list[dict[str, Any]],
    substitution_face: str | None = None,
) -> str:
    escaped_face = html.escape(face, quote=True)
    header = header.replace('face="함초롬바탕"', f'face="{escaped_face}"')
    if substitution_face is not None:
        escaped_substitution = html.escape(substitution_face, quote=True)
        marker = f'<hh:font id="1" face="{escaped_face}" type="TTF" isEmbedded="0">'
        if header.count(marker) != 7:
            raise OracleStage2Error("exact font marker must occur once per language")
        header = header.replace(
            marker,
            marker
            + f'<hh:substFont face="{escaped_substitution}" type="TTF" '
            'isEmbedded="0" binaryItemIDRef=""/>',
        )
    match = re.search(r'(<hh:charPr id="0".*?</hh:charPr>)', header, flags=re.DOTALL)
    if match is None:
        raise OracleStage2Error("base charPr 0 is missing")
    base = match.group(1)
    additions: list[str] = []
    for entry in matrix:
        value = base.replace('id="0"', f'id="{entry["charPropertyId"]}"', 1)
        value = value.replace(
            'useKerning="0"', f'useKerning="{1 if entry["kerning"] else 0}"', 1
        )
        ratio = entry["ratio"]
        spacing = entry["spacing"]
        value = re.sub(
            r'<hh:ratio [^>]*/>',
            '<hh:ratio '
            + " ".join(
                f'{language}="{ratio}"'
                for language in ("hangul", "latin", "hanja", "japanese", "other", "symbol", "user")
            )
            + "/>",
            value,
            count=1,
        )
        value = re.sub(
            r'<hh:spacing [^>]*/>',
            '<hh:spacing '
            + " ".join(
                f'{language}="{spacing}"'
                for language in ("hangul", "latin", "hanja", "japanese", "other", "symbol", "user")
            )
            + "/>",
            value,
            count=1,
        )
        additions.append(value)
    header = replace_once(
        header,
        '<hh:charProperties itemCnt="7">',
        f'<hh:charProperties itemCnt="{7 + len(additions)}">',
        "charProperties count",
    )
    return replace_once(
        header,
        "</hh:charProperties>",
        "".join(additions) + "</hh:charProperties>",
        "charProperties end",
    )


def generate_fixture(
    *,
    contract: dict[str, Any],
    output_root: Path,
    output_relative: str,
    manifest_relative: str,
    document_face: str,
    substitution_face: str | None = None,
) -> dict[str, Any]:
    fixture = contract["fixture"]
    sources = contract["fontInventory"]["sources"]
    ranks = {entry["documentFace"]: entry["queueRank"] for entry in sources}
    if document_face not in ranks:
        raise OracleStage2Error("fixture face is not in the frozen W4 queue")
    template = regular_input(ROOT, fixture["sourceTemplate"], 4 * 1024 * 1024)
    if sha256_file(template) != fixture["sourceTemplateSha256"]:
        raise OracleStage2Error("fixture source template hash drift")

    with zipfile.ZipFile(template) as source:
        names = source.namelist()
        if names[0] != "mimetype" or len(names) != len(set(names)):
            raise OracleStage2Error("fixture source ZIP inventory is invalid")
        entries = {name: source.read(name) for name in names}

    matrix = []
    for index, (ratio, spacing, kerning) in enumerate(
        itertools.product(fixture["ratios"], fixture["spacings"], fixture["kerning"]),
        start=7,
    ):
        matrix.append(
            {
                "charPropertyId": index,
                "ratio": ratio,
                "spacing": spacing,
                "kerning": kerning,
            }
        )

    header = entries["Contents/header.xml"].decode("utf-8")
    entries["Contents/header.xml"] = build_char_properties(
        header, document_face, matrix, substitution_face
    ).encode("utf-8")

    context_records: list[dict[str, Any]] = []
    body: list[str] = []
    paragraph_id = 49631000
    for index, entry in enumerate(matrix):
        stored = index % 2 == 0
        label = (
            f'BODY R{entry["ratio"]} S{entry["spacing"]} '
            f'K{1 if entry["kerning"] else 0}'
        )
        text = f"{label} | {fixture['text']}"
        body.append(
            paragraph(
                paragraph_id,
                entry["charPropertyId"],
                text,
                stored_line_seg=stored,
            )
        )
        context_records.append(
            {
                "context": "body",
                "paragraphId": paragraph_id,
                "charPropertyId": entry["charPropertyId"],
                "lineSegLane": "stored-line-lane" if stored else "fresh-candidate-lane",
            }
        )
        paragraph_id += 1

    representative = [matrix[0], matrix[9], matrix[-1]]
    table_paragraphs = []
    text_box_paragraphs = []
    for context_index, entry in enumerate(representative):
        stored = context_index % 2 == 0
        table_id = paragraph_id + context_index
        box_id = paragraph_id + 10 + context_index
        text = f"FIXED R{entry['ratio']} S{entry['spacing']} | {fixture['text']}"
        table_paragraphs.append(
            paragraph(
                table_id,
                entry["charPropertyId"],
                text,
                stored_line_seg=stored,
                width=28000,
            )
        )
        text_box_paragraphs.append(
            paragraph(
                box_id,
                entry["charPropertyId"],
                text,
                stored_line_seg=not stored,
                width=28000,
            )
        )
        context_records.extend(
            [
                {
                    "context": "table-cell",
                    "paragraphId": table_id,
                    "charPropertyId": entry["charPropertyId"],
                    "lineSegLane": "stored-line-lane" if stored else "fresh-candidate-lane",
                },
                {
                    "context": "text-box",
                    "paragraphId": box_id,
                    "charPropertyId": entry["charPropertyId"],
                    "lineSegLane": "fresh-candidate-lane" if stored else "stored-line-lane",
                },
            ]
        )
    paragraph_id += 20
    body.append(table_host(paragraph_id, "".join(table_paragraphs)))
    body.append(text_box_host(paragraph_id + 1, "".join(text_box_paragraphs)))

    extreme = matrix[-1]
    header_text = f"HEADER R80 S-10 | {fixture['text']}"
    footer_text = f"FOOTER R80 S-10 | {fixture['text']}"
    controls = header_or_footer(
        "header", 49630003, extreme["charPropertyId"], header_text
    ) + header_or_footer("footer", 49630004, extreme["charPropertyId"], footer_text)
    context_records.extend(
        [
            {
                "context": "header",
                "paragraphId": 49630003,
                "charPropertyId": extreme["charPropertyId"],
                "lineSegLane": "stored-line-lane",
            },
            {
                "context": "footer",
                "paragraphId": 49630004,
                "charPropertyId": extreme["charPropertyId"],
                "lineSegLane": "stored-line-lane",
            },
        ]
    )

    section = entries["Contents/section0.xml"].decode("utf-8")
    marker = '<hp:run charPrIDRef="0"><hp:t/></hp:run><hp:linesegarray>'
    replacement = (
        '<hp:run charPrIDRef="0"><hp:t/></hp:run>'
        f'<hp:run charPrIDRef="0">{controls}</hp:run><hp:linesegarray>'
    )
    section = replace_once(section, marker, replacement, "header/footer control insertion")
    section = replace_once(section, "</hs:sec>", "".join(body) + "</hs:sec>", "section end")
    entries["Contents/section0.xml"] = section.encode("utf-8")
    entries["Preview/PrvText.txt"] = fixture["text"].encode("utf-8")

    output = output_path(output_root, output_relative)
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        archive.writestr(zip_info("mimetype", zipfile.ZIP_STORED), entries.pop("mimetype"))
        for name in sorted(entries):
            archive.writestr(zip_info(name, zipfile.ZIP_DEFLATED), entries[name])

    semantic = {
        "contractVersion": (
            fixture["contractVersion"]
            if substitution_face is None
            else f'{fixture["contractVersion"]}-subst-v1'
        ),
        "documentFace": document_face,
        "queueRank": ranks[document_face],
        "sourceTemplateSha256": fixture["sourceTemplateSha256"],
        "textSha256": sha256_bytes(fixture["text"].encode("utf-8")),
        "matrix": matrix,
        "contexts": context_records,
        "fontBytesEmbedded": False,
    }
    if substitution_face is not None:
        semantic["substitutionFace"] = substitution_face
    lane_counts = {
        lane: sum(record["lineSegLane"] == lane for record in context_records)
        for lane in fixture["lineSegLanes"]
    }
    with zipfile.ZipFile(output) as archive:
        zip_entries = [
            {
                "name": info.filename,
                "bytes": info.file_size,
                "sha256": sha256_bytes(archive.read(info.filename)),
                "compression": "stored"
                if info.compress_type == zipfile.ZIP_STORED
                else "deflated",
            }
            for info in archive.infolist()
        ]
    manifest = {
        "schemaVersion": 1,
        "kind": "font-oracle-typesetting-fixture-manifest",
        "issue": 4963,
        "sourceFormat": "hwpx",
        "inputSha256": sha256_file(output),
        "inputBytes": output.stat().st_size,
        "semanticSha256": sha256_bytes(canonical_json_bytes(semantic)),
        "semantic": semantic,
        "lineSegLaneCounts": lane_counts,
        "zipEntries": zip_entries,
    }
    write_json(output_path(output_root, manifest_relative), manifest, mode=0o644)
    return manifest


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--output", default="oracle_typesetting_fixture.hwpx")
    parser.add_argument("--manifest", default="oracle_typesetting_fixture.manifest.json")
    parser.add_argument("--face")
    parser.add_argument("--subst-face")
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    contract = read_contract()
    face = args.face or contract["fixture"]["canaryDocumentFace"]
    manifest = generate_fixture(
        contract=contract,
        output_root=args.output_root,
        output_relative=args.output,
        manifest_relative=args.manifest,
        document_face=face,
        substitution_face=args.subst_face,
    )
    print(manifest["inputSha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
