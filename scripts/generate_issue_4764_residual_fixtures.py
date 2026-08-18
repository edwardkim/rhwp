#!/usr/bin/env python3
"""Generate #4764 residual raster catalogs and page-isolation fixtures.

Each locked page from #4764's HWP 2020 mapping becomes catalog + isolation
cases. Known leftovers (issue4090 wrap, 76076 glyph/paint, #4490/#4491 font)
are tagged; other pages stay `none` so remesure does not invent layout bugs.
"""

from __future__ import annotations

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "tests" / "fixtures" / "issue_4764"

CATALOG_HEADER = (
    "corpus\tsource_kind\tpage_index\thuman_page\tlocked_count\tresidual\t"
    "isolation\tink_budget_ppm\thist_l1\tbbox_hu\tfont_env\twrap_risk\t"
    "glyph_risk\ttable_risk\tstage\tnote"
)
CASE_HEADER = (
    "case_id\tcorpus\tpage_index\thuman_page\tscenario\tinject\t"
    "expect_class\texpect_isolated\texpect_doc_ok\texpect_page_count\t"
    "ink_budget_ppm\thist_l1\tbbox_hu\tnote"
)
FINGERPRINT_HEADER = (
    "corpus\tpage_index\twidth\theight\tink_ppm\tleft_strip_ink\t"
    "right_band_ink\thist32\tbbox\tresidual_seed"
)

# Locked counts from PR #4763 / issue #4764.
CORPORA = [
    {
        "id": "admin_hwp",
        "kind": "hwp",
        "pages": 383,
        "locked": 383,
        "stage": "s125",
        "note": "2025 admin handbook HWP remesure",
    },
    {
        "id": "admin_hwpx",
        "kind": "hwpx",
        "pages": 383,
        "locked": 383,
        "stage": "s134",
        "note": "2025 admin handbook HWPX remesure",
    },
    {
        "id": "reg_76076",
        "kind": "hwp",
        "pages": 82,
        "locked": 82,
        "stage": "s74",
        "note": "76076 regulatory analysis residual glyph/paint",
    },
    {
        "id": "issue4090",
        "kind": "hwpx",
        "pages": 17,
        "locked": 17,
        "stage": "s71",
        "note": "issue4090 sandbox wrap isolation",
    },
    {
        "id": "hwp3_sample16",
        "kind": "hwp",
        "pages": 16,
        "locked": 0,
        "stage": "s201",
        "note": "hwp3 sample16 raster remesure",
    },
    {
        "id": "hwp3_hwp5_2010",
        "kind": "hwp",
        "pages": 16,
        "locked": 0,
        "stage": "s201",
        "note": "hwp3 converted 2010",
    },
    {
        "id": "hwp3_hwp5_2018",
        "kind": "hwp",
        "pages": 16,
        "locked": 0,
        "stage": "s201",
        "note": "hwp3 converted 2018",
    },
    {
        "id": "hwp3_hwp5_2020",
        "kind": "hwp",
        "pages": 16,
        "locked": 0,
        "stage": "s201",
        "note": "hwp3 converted 2020",
    },
    {
        "id": "hwp3_hwp5_2022",
        "kind": "hwp",
        "pages": 16,
        "locked": 0,
        "stage": "s201",
        "note": "hwp3 converted 2022",
    },
    {
        "id": "hwp3_hwp5_2024",
        "kind": "hwp",
        "pages": 16,
        "locked": 0,
        "stage": "s201",
        "note": "hwp3 converted 2024",
    },
    {
        "id": "note_tail_hwp",
        "kind": "hwp",
        "pages": 8,
        "locked": 0,
        "stage": "s16",
        "note": "footnote tail HWP remesure",
    },
    {
        "id": "note_tail_hwpx",
        "kind": "hwpx",
        "pages": 8,
        "locked": 0,
        "stage": "s16",
        "note": "footnote tail HWPX remesure",
    },
    {
        "id": "issue2006",
        "kind": "hwpx",
        "pages": 24,
        "locked": 0,
        "stage": "s22",
        "note": "policy research report remesure",
    },
    {
        "id": "issue4490",
        "kind": "hwp",
        "pages": 6,
        "locked": 0,
        "stage": "s4490",
        "note": "4490 p2 font width/weight",
    },
    {
        "id": "issue4491",
        "kind": "hwp",
        "pages": 40,
        "locked": 0,
        "stage": "s169",
        "note": "4491 p9/p26/p36 font and table",
    },
]

# 1-based human pages with leftover residuals after #3820 page-count lock.
KNOWN = {
    ("issue4090", 5): ("wrap_flow", 1, 0, 0, "s71", "right square table left strip"),
    ("issue4090", 7): ("wrap_flow", 1, 0, 0, "s71", "right square table left strip"),
    ("issue4090", 15): ("wrap_flow", 1, 0, 0, "s71", "right square table left strip"),
    ("issue4090", 17): ("wrap_flow", 1, 0, 0, "s71", "right square table left strip"),
    ("reg_76076", 18): ("glyph", 0, 1, 0, "s122", "rowspan tail glyph"),
    ("reg_76076", 19): ("glyph", 0, 1, 0, "s122", "rowspan tail glyph"),
    ("reg_76076", 33): ("paint", 0, 1, 0, "s68", "nested table paint"),
    ("reg_76076", 34): ("paint", 0, 1, 0, "s78", "nested table paint"),
    ("reg_76076", 35): ("glyph", 0, 1, 0, "s77", "nested fragment glyph"),
    ("reg_76076", 36): ("paint", 0, 1, 0, "s78", "nested table paint"),
    ("reg_76076", 81): ("glyph", 0, 1, 0, "s82", "owner-adjacent glyph"),
    ("reg_76076", 82): ("glyph", 0, 1, 0, "s82", "owner-adjacent glyph"),
    ("issue4490", 2): ("font_width", 0, 0, 0, "s4490", "font width/weight"),
    ("issue4491", 9): ("table_place", 0, 0, 1, "s4491", "table placement"),
    ("issue4491", 26): ("font_env", 0, 0, 0, "s169", "HCRDotum font environment"),
    ("issue4491", 36): ("font_weight", 0, 0, 0, "s4491", "font weight"),
    ("admin_hwp", 144): ("none", 0, 0, 0, "s66", "p144 header regression preserve"),
    ("admin_hwp", 145): ("none", 0, 0, 0, "s66", "p145 header regression preserve"),
    ("admin_hwp", 156): ("none", 0, 0, 0, "s67", "p156 square wrap regression preserve"),
    ("admin_hwpx", 144): ("none", 0, 0, 0, "s66", "hwpx p144 preserve"),
    ("admin_hwpx", 145): ("none", 0, 0, 0, "s66", "hwpx p145 preserve"),
    ("admin_hwpx", 156): ("none", 0, 0, 0, "s67", "hwpx p156 preserve"),
}

SCENARIOS = (
    ("clean", "none", "none", 0, 1),
    ("decode_fail", "decode_fail", "none", 1, 1),
    ("wrap_deficit", "left_strip_drop", "wrap_flow", 0, 1),
    ("glyph_shift", "glyph_hist", "glyph", 0, 1),
    ("paint_blob", "paint_fill", "paint", 0, 1),
    ("font_env", "face_substitute", "font_env", 0, 1),
    ("font_width", "advance_shift", "font_width", 0, 1),
    ("table_place", "table_shift", "table_place", 0, 1),
)


def seed_bytes(corpus: str, page_index: int, salt: bytes = b"") -> bytes:
    return hashlib.sha256(f"{corpus}:{page_index}".encode() + salt).digest()


def hist32(corpus: str, page_index: int) -> str:
    digest = seed_bytes(corpus, page_index)
    bins = []
    for i in range(32):
        bins.append(str(40 + digest[i] % 80))
    return ",".join(bins)


def page_residual(corpus: str, human: int) -> tuple[str, int, int, int, str, str]:
    if (corpus, human) in KNOWN:
        residual, wrap, glyph, table, stage, note = KNOWN[(corpus, human)]
        return residual, wrap, glyph, table, stage, note
    return "none", 0, 0, 0, "", ""


def write_catalogs() -> list[tuple[dict, int, str]]:
    OUT.mkdir(parents=True, exist_ok=True)
    rows_by_corpus: list[tuple[dict, int, str]] = []
    all_rows = [CATALOG_HEADER]
    for corpus in CORPORA:
        rows = [CATALOG_HEADER]
        for index in range(corpus["pages"]):
            human = index + 1
            residual, wrap, glyph, table, stage, note = page_residual(corpus["id"], human)
            stage = stage or corpus["stage"]
            note = note or f"{corpus['note']} p{human}"
            font_env = 1 if residual == "font_env" else 0
            isolation = "independent"
            line = (
                f"{corpus['id']}\t{corpus['kind']}\t{index}\t{human}\t"
                f"{corpus['locked']}\t{residual}\t{isolation}\t"
                f"{1200 + (index % 7) * 50}\t{80 + (index % 5) * 10}\t"
                f"{32 + (index % 3) * 8}\t{font_env}\t{wrap}\t{glyph}\t{table}\t"
                f"{stage}\t{note}"
            )
            rows.append(line)
            all_rows.append(line)
            rows_by_corpus.append((corpus, index, residual))
        path = OUT / f"catalog_{corpus['id']}.tsv"
        path.write_text("\n".join(rows) + "\n", encoding="utf-8", newline="\n")
    (OUT / "catalog_all.tsv").write_text("\n".join(all_rows) + "\n", encoding="utf-8", newline="\n")
    return rows_by_corpus


def write_isolation_cases(pages: list[tuple[dict, int, str]]) -> None:
    lines = [CASE_HEADER]
    for corpus, index, residual in pages:
        human = index + 1
        expect_count = corpus["locked"] or corpus["pages"]
        for name, inject, expect_class, isolated, doc_ok in SCENARIOS:
            # Clean compare uses matching rasters, so the class is none.
            # Catalog leftovers are asserted separately from isolation injects.
            # decode_fail is isolated and must not change the locked page count.
            case_id = f"{corpus['id']}:{index}:{name}"
            note = f"{corpus['id']} p{human} {name} isolation"
            lines.append(
                f"{case_id}\t{corpus['id']}\t{index}\t{human}\t{name}\t{inject}\t"
                f"{expect_class}\t{isolated}\t{doc_ok}\t{expect_count}\t"
                f"{1500}\t{120}\t{40}\t{note}"
            )
    (OUT / "isolation_cases.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def write_fingerprints(pages: list[tuple[dict, int, str]]) -> None:
    lines = [FINGERPRINT_HEADER]
    for corpus, index, residual in pages:
        digest = seed_bytes(corpus["id"], index, b"fp")
        width = 96 + (digest[0] % 8) * 8
        height = 128 + (digest[1] % 8) * 8
        ink_ppm = 8_000 + digest[2] * 20
        left = 200 + digest[3]
        right = 180 + digest[4]
        x0, y0 = 8 + digest[5] % 6, 10 + digest[6] % 6
        x1, y1 = x0 + 40 + digest[7] % 10, y0 + 30 + digest[8] % 10
        bbox = f"{x0},{y0},{x1},{y1}"
        lines.append(
            f"{corpus['id']}\t{index}\t{width}\t{height}\t{ink_ppm}\t{left}\t"
            f"{right}\t{hist32(corpus['id'], index)}\t{bbox}\t{residual}"
        )
    (OUT / "fingerprints.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def write_pdf_manifest(pages: list[tuple[dict, int, str]]) -> None:
    lines = [
        "corpus\tpage_index\twidth_pt\theight_pt\tcontent_kind\tisolate_ok",
    ]
    for corpus, index, residual in pages:
        digest = seed_bytes(corpus["id"], index, b"pdf")
        width = 200 + digest[0] % 40
        height = 280 + digest[1] % 40
        kind = "rect" if residual in {"paint", "table_place"} else "text"
        lines.append(f"{corpus['id']}\t{index}\t{width}\t{height}\t{kind}\t1")
    (OUT / "pdf_pages.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def write_readme() -> None:
    text = """# Issue #4764 residual raster fixtures

Generated by `scripts/generate_issue_4764_residual_fixtures.py`.

- `catalog_*.tsv` — one row per physical page in the #4764 mapping table
- `catalog_all.tsv` — concatenated catalog
- `isolation_cases.tsv` — every page × isolation scenario
- `fingerprints.tsv` — deterministic synthetic raster fingerprints
- `pdf_pages.tsv` — isolated PDF page builder inputs

Page-count locks: admin handbook 383, 76076 82, issue4090 17.
Runtime classifiers must not branch on these file names.
"""
    (OUT / "README.md").write_text(text, encoding="utf-8", newline="\n")


def main() -> None:
    pages = write_catalogs()
    write_isolation_cases(pages)
    write_fingerprints(pages)
    write_pdf_manifest(pages)
    write_readme()
    isolation = (OUT / "isolation_cases.tsv").read_text(encoding="utf-8").count("\n")
    catalog = (OUT / "catalog_all.tsv").read_text(encoding="utf-8").count("\n")
    print(f"wrote {catalog} catalog lines, {isolation} isolation lines under {OUT}")


if __name__ == "__main__":
    main()
