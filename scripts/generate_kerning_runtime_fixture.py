#!/usr/bin/env python3
"""Generate the public exact-source runtime fixture for Issue #4968 R4E."""

from __future__ import annotations

import argparse
from pathlib import Path

from generate_kerning_pair_fixture import KerningFixtureSpec, generate_fixture
from oracle_stage2_common import OracleStage2Error


FONT = "tests/fixtures/fonts/RHWPExactKerningSmoke.ttf"
FONT_SHA256 = "775667d1980cd734e331f01e9390e02191bc35d669325291c842968cb0a4a9fc"
DOCUMENT_FACE = "RHWP Kerning Boundary"
PAIR_TEXT = "AV To WA HH"
CHAR_SHAPE_IDS = tuple(range(7, 25))

PROJECTION_CONTRACT = {
    "schemaVersion": 1,
    "kind": "kerning-runtime-canonical-projection",
    "comparison": "byte-exact-canonical-json",
    "rowKey": [
        "context",
        "charShapeId",
        "languageIndex",
        "lineSegLane",
    ],
    "allowedFields": [
        "context",
        "charShapeId",
        "languageIndex",
        "lineSegLane",
        "ratio",
        "spacing",
        "kerningRequested",
        "registration",
        "capability",
        "disposition",
        "fallbackReason",
        "paragraphRef",
        "measurement.totalWidth",
        "line.starts",
        "line.boundaries",
        "layout.positions",
        "layout.bboxWidth",
        "canvas.positions",
        "canvasKit.positions",
        "svg.sha256",
        "pageCount",
    ],
    "normalization": {
        "paragraphRef": {
            "native64Sentinel": "18446744073709551615",
            "wasm32Sentinel": "4294967295",
            "canonical": "para:MAX",
        }
    },
    "forbiddenFields": [
        "text",
        "fontBytes",
        "sourcePath",
        "sourceDocumentHash",
        "fileName",
        "absolutePath",
        "privateIdentity",
    ],
}

RUNTIME_SPEC = KerningFixtureSpec(
    contract_version="w9-q3-5-r4e-runtime-v1",
    manifest_kind="kerning-runtime-fixture-manifest",
    stage="W9-Q3-5R4E-0",
    font_path=FONT,
    font_sha256=FONT_SHA256,
    font_license="MIT",
    document_face=DOCUMENT_FACE,
    pair_text=PAIR_TEXT,
    pair_classes=(
        {"text": "AV", "kind": "adjusted", "gposXAdvance": -80},
        {"text": "To", "kind": "adjusted", "gposXAdvance": -40},
        {"text": "WA", "kind": "zero-adjustment", "gposXAdvance": 0},
        {"text": "HH", "kind": "no-pair", "gposXAdvance": 0},
    ),
    paragraph_id_start=49681000,
    body_labels=False,
    font_bytes_tracked=True,
    semantic_extension={
        "fontLicensePath": (
            "tests/fixtures/fonts/RHWPExactKerningSmoke.LICENSE.md"
        ),
        "visibleTextGlyphSet": ["space", "A", "V", "T", "o", "W", "H"],
        "exactSourceRegistration": {
            "nativeApi": "DocumentCore::register_exact_font_source_native",
            "wasmApi": "HwpDocument.registerExactFontSource",
            "languageIndex": 1,
            "language": "latin",
            "faceIndex": 0,
            "slots": [
                {"charShapeId": char_shape_id, "languageIndex": 1}
                for char_shape_id in CHAR_SHAPE_IDS
            ],
        },
        "canonicalProjectionContract": PROJECTION_CONTRACT,
    },
)


def generate_runtime_fixture(
    *, output_root: Path, output_relative: str, manifest_relative: str
) -> dict:
    return generate_fixture(
        output_root=output_root,
        output_relative=output_relative,
        manifest_relative=manifest_relative,
        spec=RUNTIME_SPEC,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--output", default="kerning_runtime_fixture.hwpx")
    parser.add_argument(
        "--manifest", default="kerning_runtime_fixture.manifest.json"
    )
    args = parser.parse_args()
    manifest = generate_runtime_fixture(
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
