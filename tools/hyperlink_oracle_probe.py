#!/usr/bin/env python3
"""Inspect actual PDF link annotations and optional paired HWPX fields (#6963).

Requires pypdf. Page numbers in the JSON are 1-based. This never opens a URI.
Example:
  python tools/hyperlink_oracle_probe.py pdf/basic/Textmail-2022.pdf \
    --source samples/basic/Textmail.hwp --expect-uri http://www.hancom.co.kr
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys
import xml.etree.ElementTree as ET
import zipfile


def fingerprint(path: Path) -> dict:
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"path": path.as_posix(), "sha256": digest}


def source_fields(path: Path) -> list[dict] | None:
    if path.suffix.lower() != ".hwpx":
        return None  # Binary HWP parsing is deliberately left to the document engine.
    result = []
    with zipfile.ZipFile(path) as archive:
        for part in sorted(archive.namelist()):
            if not (part.startswith("Contents/section") and part.endswith(".xml")):
                continue
            root = ET.fromstring(archive.read(part))
            for field in root.iter():
                if field.tag.rsplit("}", 1)[-1] != "fieldBegin":
                    continue
                if field.get("type") != "HYPERLINK":
                    continue
                result.append({
                    "part": part,
                    "id": field.get("id"),
                    "parameters": {
                        child.get("name", ""): child.text or ""
                        for child in field.iter()
                        if child.tag.rsplit("}", 1)[-1] == "stringParam"
                    },
                })
    return result


def probe(pdf_path: Path, source: Path | None) -> dict:
    from pypdf import PdfReader

    reader = PdfReader(pdf_path)
    page_numbers = {
        (page.indirect_reference.idnum, page.indirect_reference.generation): index
        for index, page in enumerate(reader.pages, 1)
        if page.indirect_reference is not None
    }
    links = []
    for index, page in enumerate(reader.pages, 1):
        for reference in page.get("/Annots", []):
            annotation = reference.get_object()
            if annotation.get("/Subtype") != "/Link":
                continue
            action = annotation.get("/A", {})
            if hasattr(action, "get_object"):
                action = action.get_object()
            if not isinstance(action, dict):
                action = {}
            destination = action.get("/D", annotation.get("/Dest"))
            destination_page = None
            if isinstance(destination, list) and destination:
                first = destination[0]
                if hasattr(first, "idnum"):
                    destination_page = page_numbers.get((first.idnum, first.generation))
            rect = [float(n) for n in annotation.get("/Rect", [])]
            links.append({
                "page": index,
                "rect": rect,
                "action": str(action.get("/S", "")),
                "uri": str(action.get("/URI", "")),
                "destinationPage": destination_page,
                "destinationResolved": destination_page is not None,
            })
    metadata = reader.metadata or {}
    result = {
        "pdf": fingerprint(pdf_path),
        "producer": str(metadata.get("/Producer", "")),
        "creator": str(metadata.get("/Creator", "")),
        "pageCount": len(reader.pages),
        "linkCount": len(links),
        "links": links,
    }
    if source is not None:
        result["source"] = fingerprint(source)
        result["sourceHyperlinkFields"] = source_fields(source)
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pdf", type=Path)
    parser.add_argument("--source", type=Path)
    parser.add_argument("--expect-uri", action="append", default=[])
    parser.add_argument("--expect-link-count", type=int)
    args = parser.parse_args()
    try:
        result = probe(args.pdf, args.source)
    except (ImportError, OSError, ValueError, ET.ParseError, zipfile.BadZipFile) as error:
        print(f"hyperlink probe failed: {error}", file=sys.stderr)
        return 2
    uris = {link["uri"] for link in result["links"] if link["action"] == "/URI"}
    missing = sorted(set(args.expect_uri) - uris)
    count_matches = args.expect_link_count is None or args.expect_link_count == result["linkCount"]
    result["checks"] = {"missingUris": missing, "linkCountMatches": count_matches}
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 1 if missing or not count_matches else 0


if __name__ == "__main__":
    raise SystemExit(main())
