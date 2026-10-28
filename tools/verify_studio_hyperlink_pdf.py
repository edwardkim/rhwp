#!/usr/bin/env python3
"""#6963: independently compare Chromium PDF /Link annotations to print DOM rects.

Requires pypdf. Run after rhwp-studio/e2e/hyperlink-pdf-issue6963.test.mjs:
  python tools/verify_studio_hyperlink_pdf.py output/pdf/issue6963-stage5
"""
import argparse
import json
from pathlib import Path
from urllib.parse import urlsplit, urlunsplit

from pypdf import PdfReader
from hyperlink_oracle_probe import probe


def with_root_path(uri: str) -> str:
    """Chromium serializes a bare HTTP origin with '/', preserving query/fragment."""
    parts = urlsplit(uri)
    if parts.scheme in ("http", "https") and not parts.path:
        return urlunsplit(parts._replace(path="/"))
    return uri


def verify(directory: Path) -> dict:
    evidence = json.loads((directory / "browser-evidence.json").read_text())
    results = {}
    for name, expected in evidence["pdfs"].items():
        pdf = directory / f"{name}.pdf"
        result = probe(pdf, None)
        result["pdf"]["path"] = pdf.name
        pages = PdfReader(pdf).pages
        actual = result["links"].copy()
        assert len(actual) == len(expected) > 0, (name, len(actual), len(expected))
        errors = []
        for link in expected:
            height = float(pages[link["page"]].mediabox.height)
            rect = [link["x"] * .75, height - (link["y"] + link["height"]) * .75,
                    (link["x"] + link["width"]) * .75, height - link["y"] * .75]
            candidates = [item for item in actual if item["page"] == link["page"] + 1
                          and item["uri"] == with_root_path(link["uri"]) and item["action"] == "/URI"]
            assert candidates, (name, link)
            match = min(candidates, key=lambda item: max(abs(a-b) for a, b in zip(item["rect"], rect)))
            error = max(abs(a-b) for a, b in zip(match["rect"], rect))
            # Chromium rounds link hit areas to device pixels before PDF conversion.
            assert error <= 1, (name, rect, match, error)
            assert match["rect"][2] > match["rect"][0] and match["rect"][3] > match["rect"][1]
            actual.remove(match)
            errors.append(error)
        result["maxAnnotationVsDomErrorPt"] = max(errors)
        results[name] = result
    assert results["studio-hwp"]["links"] == results["studio-hwpx"]["links"]
    for name, oracle in evidence.get("oracles", {}).items():
        actual = results[name]["links"]
        oracle_result = probe(Path(oracle["pdf"]), None)
        reference = oracle_result["links"]
        results[name]["oraclePdf"] = oracle_result["pdf"]
        comparisons = []
        for expected in oracle["uris"]:
            generated = [item for item in actual if with_root_path(item["uri"]) == with_root_path(expected)]
            original = [item for item in reference if with_root_path(item["uri"]) == with_root_path(expected)]
            assert generated and original, (name, expected)
            for link in generated:
                same_page = [item for item in original if item["page"] == link["page"]]
                assert len(same_page) == 1, (name, link, same_page)
                rect = same_page[0]["rect"]
                normalized = [min(rect[0], rect[2]), min(rect[1], rect[3]),
                              max(rect[0], rect[2]), max(rect[1], rect[3])]
                comparisons.append({"page": link["page"], "uri": expected,
                                    "oracleRectPt": normalized, "generatedRectPt": link["rect"],
                                    "deltaPt": [a-b for a, b in zip(link["rect"], normalized)]})
        results[name]["oracleUrisMatched"] = oracle["uris"]
        # Layout differences against Hancom are reported separately from annotation placement.
        results[name]["oracleGeometryComparisons"] = comparisons
    return {"parser": "pypdf", "browser": evidence["browser"], "results": results}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    result = verify(args.directory)
    (args.directory / "pdf-validation.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({name: {"pages": item["pageCount"], "links": item["linkCount"],
                            "maxErrorPt": item["maxAnnotationVsDomErrorPt"]}
                      for name, item in result["results"].items()}, indent=2))
