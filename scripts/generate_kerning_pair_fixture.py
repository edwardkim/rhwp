#!/usr/bin/env python3
"""Generate the deterministic public HWPX pair fixture for Issue #4968 W9-Q2."""

from __future__ import annotations

import argparse
import itertools
import zipfile
from pathlib import Path
from typing import Any

from generate_oracle_typesetting_fixture import (
    build_char_properties,
    paragraph,
    replace_once,
    table_host,
    text_box_host,
    zip_info,
)
from oracle_stage2_common import (
    ROOT,
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_json,
)


TEMPLATE = "samples/hwpx/ref/ref_empty.hwpx"
TEMPLATE_SHA256 = "c58144645069f7d1258e91404730618ad568bc4d47680ad5f891d3050aa308c7"
FONT = "ttfs/opensource/NotoSansKR-Regular.ttf"
FONT_SHA256 = "6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74"
DOCUMENT_FACE = "Noto Sans KR"
PAIR_TEXT = "AV To WA HH 가나다"
RATIOS = (100, 90, 80)
SPACINGS = (0, -5, -10)
KERNING = (False, True)
LANES = ("stored-line-lane", "fresh-candidate-lane")


def generate_fixture(
    *, output_root: Path, output_relative: str, manifest_relative: str
) -> dict[str, Any]:
    template = regular_input(ROOT, TEMPLATE, 4 * 1024 * 1024)
    if sha256_file(template) != TEMPLATE_SHA256:
        raise OracleStage2Error("kerning fixture template hash drift")
    font = regular_input(ROOT, FONT, 32 * 1024 * 1024)
    if sha256_file(font) != FONT_SHA256:
        raise OracleStage2Error("public kerning font hash drift")

    with zipfile.ZipFile(template) as source:
        names = source.namelist()
        if names[0] != "mimetype" or len(names) != len(set(names)):
            raise OracleStage2Error("fixture source ZIP inventory is invalid")
        entries = {name: source.read(name) for name in names}

    matrix = [
        {
            "charPropertyId": index,
            "ratio": ratio,
            "spacing": spacing,
            "kerning": kerning,
        }
        for index, (ratio, spacing, kerning) in enumerate(
            itertools.product(RATIOS, SPACINGS, KERNING), start=7
        )
    ]
    header = entries["Contents/header.xml"].decode("utf-8")
    entries["Contents/header.xml"] = build_char_properties(
        header, DOCUMENT_FACE, matrix
    ).encode("utf-8")

    body: list[str] = []
    contexts: list[dict[str, Any]] = []
    paragraph_id = 49680000
    for matrix_index, entry in enumerate(matrix):
        # Matrix order is K0 then K1. Keep each on/off pair in the same lane so
        # pair positioning is not confounded by stored/fresh layout selection.
        stored = (matrix_index // 2) % 2 == 0
        lane = "stored-line-lane" if stored else "fresh-candidate-lane"
        label = (
            f'BODY R{entry["ratio"]} S{entry["spacing"]} '
            f'K{1 if entry["kerning"] else 0} '
            f'L{"stored" if stored else "fresh"}'
        )
        body.append(
            paragraph(
                paragraph_id,
                entry["charPropertyId"],
                f"{label} | {PAIR_TEXT}",
                stored_line_seg=stored,
            )
        )
        contexts.append(
            {
                "context": "body",
                "paragraphId": paragraph_id,
                "charPropertyId": entry["charPropertyId"],
                "lineSegLane": lane,
            }
        )
        paragraph_id += 1

    representatives = [matrix[0], matrix[1]]
    table_paragraphs: list[str] = []
    text_box_paragraphs: list[str] = []
    for index, entry in enumerate(representatives):
        table_stored = index == 0
        table_id = paragraph_id + index
        box_id = paragraph_id + 10 + index
        label = f'K{1 if entry["kerning"] else 0}'
        table_paragraphs.append(
            paragraph(
                table_id,
                entry["charPropertyId"],
                f"TABLE {label} | {PAIR_TEXT}",
                stored_line_seg=table_stored,
                width=28000,
            )
        )
        text_box_paragraphs.append(
            paragraph(
                box_id,
                entry["charPropertyId"],
                f"TEXTBOX {label} | {PAIR_TEXT}",
                stored_line_seg=not table_stored,
                width=28000,
            )
        )
        contexts.extend(
            [
                {
                    "context": "table-cell",
                    "paragraphId": table_id,
                    "charPropertyId": entry["charPropertyId"],
                    "lineSegLane": (
                        "stored-line-lane" if table_stored else "fresh-candidate-lane"
                    ),
                },
                {
                    "context": "text-box",
                    "paragraphId": box_id,
                    "charPropertyId": entry["charPropertyId"],
                    "lineSegLane": (
                        "fresh-candidate-lane" if table_stored else "stored-line-lane"
                    ),
                },
            ]
        )
    body.append(table_host(paragraph_id + 20, "".join(table_paragraphs)))
    body.append(text_box_host(paragraph_id + 21, "".join(text_box_paragraphs)))

    section = entries["Contents/section0.xml"].decode("utf-8")
    section = replace_once(section, "</hs:sec>", "".join(body) + "</hs:sec>", "section end")
    entries["Contents/section0.xml"] = section.encode("utf-8")
    entries["Preview/PrvText.txt"] = PAIR_TEXT.encode("utf-8")

    output = output_path(output_root, output_relative)
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        archive.writestr(zip_info("mimetype", zipfile.ZIP_STORED), entries.pop("mimetype"))
        for name in sorted(entries):
            archive.writestr(zip_info(name, zipfile.ZIP_DEFLATED), entries[name])

    semantic = {
        "contractVersion": "w9-q2-kerning-pair-v1",
        "documentFace": DOCUMENT_FACE,
        "fontSource": {
            "path": FONT,
            "sha256": FONT_SHA256,
            "license": "SIL-OFL-1.1",
            "embedded": False,
        },
        "sourceTemplateSha256": TEMPLATE_SHA256,
        "pairText": PAIR_TEXT,
        "pairClasses": [
            {"text": "AV", "kind": "adjusted", "gposXAdvance": -18},
            {"text": "To", "kind": "adjusted", "gposXAdvance": -76},
            {"text": "WA", "kind": "zero-adjustment", "gposXAdvance": 0},
            {"text": "HH", "kind": "no-pair", "gposXAdvance": 0},
            {"text": "가나", "kind": "non-latin-control", "gposXAdvance": 0},
        ],
        "matrix": matrix,
        "contexts": contexts,
        "fontBytesEmbedded": False,
    }
    lane_counts = {
        lane: sum(record["lineSegLane"] == lane for record in contexts) for lane in LANES
    }
    with zipfile.ZipFile(output) as archive:
        zip_entries = [
            {
                "name": info.filename,
                "bytes": info.file_size,
                "sha256": sha256_bytes(archive.read(info.filename)),
                "compression": (
                    "stored" if info.compress_type == zipfile.ZIP_STORED else "deflated"
                ),
            }
            for info in archive.infolist()
        ]
    manifest = {
        "schemaVersion": 1,
        "kind": "kerning-pair-fixture-manifest",
        "issue": 4968,
        "stage": "W9-Q2",
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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--output", default="kerning_pair_fixture.hwpx")
    parser.add_argument("--manifest", default="kerning_pair_fixture.manifest.json")
    args = parser.parse_args()
    manifest = generate_fixture(
        output_root=args.output_root,
        output_relative=args.output,
        manifest_relative=args.manifest,
    )
    print(manifest["inputSha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
