#!/usr/bin/env python3
"""Path-free SFNT identity, hmtx, outline and embedding inventory for #4963."""

from __future__ import annotations

import argparse
import math
from pathlib import Path
from typing import Any

from fontTools.pens.recordingPen import RecordingPen
from fontTools.ttLib import TTCollection, TTFont

from oracle_stage2_common import (
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    read_contract,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_json,
)


NAME_IDS = {
    1: "family",
    2: "subfamily",
    4: "fullName",
    6: "postScriptName",
    16: "preferredFamily",
    17: "preferredSubfamily",
}


def _sort_text(values: set[str]) -> list[str]:
    return sorted(values, key=lambda value: value.encode("utf-8"))


def _normalize_pen_value(value: Any) -> Any:
    if isinstance(value, (tuple, list)):
        return [_normalize_pen_value(entry) for entry in value]
    if isinstance(value, float):
        if not math.isfinite(value):
            raise OracleStage2Error("non-finite glyph outline coordinate")
        return float(f"{value:.9g}")
    if isinstance(value, (str, int, bool)) or value is None:
        return value
    return str(value)


def embedding_flags(fs_type: int | None) -> list[str]:
    if fs_type is None:
        return ["os2-table-unavailable"]
    if fs_type == 0:
        return ["installable-embedding"]
    flags = []
    known = {
        0x0002: "restricted-license-embedding",
        0x0004: "preview-and-print-embedding",
        0x0008: "editable-embedding",
        0x0100: "no-subsetting",
        0x0200: "bitmap-embedding-only",
    }
    for bit, label in known.items():
        if fs_type & bit:
            flags.append(label)
    known_mask = 0
    for bit in known:
        known_mask |= bit
    unknown = fs_type & ~known_mask
    if unknown:
        flags.append(f"unknown-bits-0x{unknown:04x}")
    return flags


def collection_face_count(path: Path, maximum_faces: int) -> int:
    if path.suffix.lower() not in {".ttc", ".otc"}:
        return 1
    collection = TTCollection(path, lazy=True)
    try:
        count = len(collection.fonts)
    finally:
        collection.close()
    if count <= 0 or count > maximum_faces:
        raise OracleStage2Error("font collection face count exceeds the contract")
    return count


def inspect_font(
    path: Path,
    *,
    document_face: str,
    face_index: int,
    sample_codepoints: list[int],
    maximum_collection_faces: int,
) -> dict[str, Any]:
    face_count = collection_face_count(path, maximum_collection_faces)
    if face_index < 0 or face_index >= face_count:
        raise OracleStage2Error("font face index is outside the collection")
    try:
        font = TTFont(
            path,
            fontNumber=face_index,
            lazy=False,
            recalcBBoxes=False,
            recalcTimestamp=False,
        )
    except Exception as error:
        raise OracleStage2Error(f"font parse failed: {type(error).__name__}") from error
    try:
        if "name" not in font or "head" not in font or "maxp" not in font:
            raise OracleStage2Error("font lacks required SFNT identity tables")
        names: dict[str, list[str]] = {}
        for name_id, label in NAME_IDS.items():
            values: set[str] = set()
            for record in font["name"].names:
                if record.nameID != name_id:
                    continue
                try:
                    value = record.toUnicode().strip()
                except Exception:
                    continue
                if value:
                    values.add(value)
            names[label] = _sort_text(values)

        identity_values = {
            value
            for field in ("family", "fullName", "preferredFamily")
            for value in names[field]
        }
        cmap = font.getBestCmap() or {}
        hmtx = font["hmtx"].metrics if "hmtx" in font else {}
        glyph_set = font.getGlyphSet()
        samples = []
        for codepoint in sample_codepoints:
            glyph_name = cmap.get(codepoint)
            if glyph_name is None:
                samples.append(
                    {
                        "codepoint": codepoint,
                        "character": chr(codepoint),
                        "status": "unavailable",
                        "reason": "codepoint-not-in-best-cmap",
                    }
                )
                continue
            metric = hmtx.get(glyph_name)
            if metric is None:
                raise OracleStage2Error("cmap glyph is missing from hmtx")
            pen = RecordingPen()
            try:
                glyph_set[glyph_name].draw(pen)
            except Exception as error:
                raise OracleStage2Error(
                    f"glyph outline draw failed: {type(error).__name__}"
                ) from error
            recording = [
                [operator, _normalize_pen_value(arguments)]
                for operator, arguments in pen.value
            ]
            samples.append(
                {
                    "codepoint": codepoint,
                    "character": chr(codepoint),
                    "status": "observed",
                    "glyphName": glyph_name,
                    "hmtxAdvance": metric[0],
                    "leftSideBearing": metric[1],
                    "outlineSha256": sha256_bytes(canonical_json_bytes(recording)),
                }
            )

        fs_type = font["OS/2"].fsType if "OS/2" in font else None
        sfnt_version = font.sfntVersion
        if isinstance(sfnt_version, bytes):
            sfnt_version = sfnt_version.decode("latin-1")
        result = {
            "schemaVersion": 1,
            "kind": "font-oracle-sfnt-inventory",
            "issue": 4963,
            "documentFace": document_face,
            "sha256": sha256_file(path),
            "bytes": path.stat().st_size,
            "faceIndex": face_index,
            "collectionFaceCount": face_count,
            "sfntVersion": str(sfnt_version),
            "nameTable": names,
            "exactNameMatch": document_face in identity_values,
            "unitsPerEm": font["head"].unitsPerEm,
            "glyphCount": font["maxp"].numGlyphs,
            "cmapCodepointCount": len(cmap),
            "os2FsType": fs_type,
            "embeddingFlags": embedding_flags(fs_type),
            "tables": sorted(font.keys()),
            "sampleGlyphEvidence": samples,
        }
        result["canonicalSha256"] = sha256_bytes(canonical_json_bytes(result))
        return result
    finally:
        font.close()


def inventory_relative_font(
    *,
    contract: dict[str, Any],
    font_root: Path,
    relative_font: str,
    document_face: str,
    face_index: int = 0,
) -> dict[str, Any]:
    policy = contract["fontInventory"]
    if len(policy["sampleCodepoints"]) > policy["maximumSampleCodepoints"]:
        raise OracleStage2Error("sample codepoint inventory exceeds the contract")
    path = regular_input(font_root, relative_font, policy["maximumBytes"])
    return inspect_font(
        path,
        document_face=document_face,
        face_index=face_index,
        sample_codepoints=policy["sampleCodepoints"],
        maximum_collection_faces=policy["maximumCollectionFaces"],
    )


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--font-root", type=Path, required=True)
    parser.add_argument("--font", required=True)
    parser.add_argument("--document-face", required=True)
    parser.add_argument("--face-index", type=int, default=0)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    result = inventory_relative_font(
        contract=read_contract(),
        font_root=args.font_root,
        relative_font=args.font,
        document_face=args.document_face,
        face_index=args.face_index,
    )
    write_json(output_path(args.output_root, args.output), result)
    print(result["canonicalSha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
