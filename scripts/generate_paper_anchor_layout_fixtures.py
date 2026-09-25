#!/usr/bin/env python3
"""Generate the synthetic HWPX fixtures in `samples/paper-anchor-layout/`.

Every fixture is built from the public empty document
`samples/hwpx/ref/ref_empty.hwpx` (A4, body 42520 HU wide). Only
`Contents/section0.xml` is rewritten (plus one generated 2x2 PNG and its
manifest entry for the picture fixture). No user form, text or image is
copied, so the output is publishable.

The geometry contracts that these inputs lock are described in
`samples/paper-anchor-layout/README.md` and tested by
`tests/cases/paper_anchor_float_reference.rs` and
`tests/cases/paper_anchor_table_row_height.rs`.

Usage:
    python3 scripts/generate_paper_anchor_layout_fixtures.py
"""

from __future__ import annotations

import hashlib
import json
import struct
import zipfile
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "samples" / "hwpx" / "ref" / "ref_empty.hwpx"
OUT_DIR = ROOT / "samples" / "paper-anchor-layout"
ZIP_TIMESTAMP = (1980, 1, 1, 0, 0, 0)

# ref_empty.hwpx page geometry (HWPUNIT): width 59528, margins left/right 8504.
BODY_LEFT = 8504
BODY_WIDTH = 42520

PARA_OPEN = (
    '<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
    'columnBreak="0" merged="0">'
)


def lineseg(vertpos: int, width: int, vertsize: int = 1000, spacing: int = 600) -> str:
    baseline = vertsize * 85 // 100
    return (
        f'<hp:lineseg textpos="0" vertpos="{vertpos}" vertsize="{vertsize}" '
        f'textheight="{vertsize}" baseline="{baseline}" spacing="{spacing}" '
        f'horzpos="0" horzsize="{width}" flags="393216"/>'
    )


def paragraph(text: str, segs: str, ctrl: str = "") -> str:
    body = f"<hp:t>{text}</hp:t>" if text else "<hp:t/>"
    return (
        f'{PARA_OPEN}<hp:run charPrIDRef="0">{body}{ctrl}</hp:run>'
        f"<hp:linesegarray>{segs}</hp:linesegarray></hp:p>"
    )


def cell(
    row: int,
    col: int,
    width: int,
    height: int,
    paras: str,
    *,
    has_margin: bool,
    margin: tuple[int, int, int, int],
) -> str:
    left, right, top, bottom = margin
    return (
        f'<hp:tc name="" header="0" hasMargin="{int(has_margin)}" protect="0" '
        'editable="0" dirty="0" borderFillIDRef="1">'
        '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" '
        'vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" '
        'textHeight="0" hasTextRef="0" hasNumRef="0">'
        f"{paras}</hp:subList>"
        f'<hp:cellAddr colAddr="{col}" rowAddr="{row}"/>'
        '<hp:cellSpan colSpan="1" rowSpan="1"/>'
        f'<hp:cellSz width="{width}" height="{height}"/>'
        f'<hp:cellMargin left="{left}" right="{right}" top="{top}" bottom="{bottom}"/>'
        "</hp:tc>"
    )


def table(
    rows: list[list[str]],
    *,
    width: int,
    height: int,
    wrap: str,
    treat_as_char: bool,
    vert_rel: str,
    horz_rel: str,
    vert_offset: int,
    horz_offset: int,
    in_margin: tuple[int, int, int, int] = (0, 0, 0, 0),
    table_id: int = 100,
) -> str:
    left, right, top, bottom = in_margin
    body = "".join(f"<hp:tr>{''.join(cells)}</hp:tr>" for cells in rows)
    return (
        f'<hp:tbl id="{table_id}" zOrder="0" numberingType="TABLE" '
        f'textWrap="{wrap}" textFlow="BOTH_SIDES" lock="0" pageBreak="CELL" '
        f'repeatHeader="0" rowCnt="{len(rows)}" colCnt="{len(rows[0])}" '
        'cellSpacing="0" borderFillIDRef="1" noAdjust="0">'
        f'<hp:sz width="{width}" widthRelTo="ABSOLUTE" height="{height}" '
        'heightRelTo="ABSOLUTE" protect="0"/>'
        f'<hp:pos treatAsChar="{int(treat_as_char)}" affectLSpacing="0" '
        'flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" '
        f'vertRelTo="{vert_rel}" horzRelTo="{horz_rel}" vertAlign="TOP" '
        f'horzAlign="LEFT" vertOffset="{vert_offset}" horzOffset="{horz_offset}"/>'
        '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
        f'<hp:inMargin left="{left}" right="{right}" top="{top}" bottom="{bottom}"/>'
        f"{body}</hp:tbl>"
    )


def picture(
    pic_id: int,
    *,
    size: int,
    vert_rel: str,
    horz_rel: str,
    vert_offset: int,
    horz_offset: int,
    comment: str,
) -> str:
    return (
        f'<hp:pic id="{pic_id}" zOrder="{pic_id}" numberingType="PICTURE" '
        'textWrap="IN_FRONT_OF_TEXT" textFlow="BOTH_SIDES" lock="0" '
        'dropcapstyle="None" href="" groupLevel="0" instid="0" reverse="0">'
        '<hp:offset x="0" y="0"/>'
        f'<hp:orgSz width="{size}" height="{size}"/>'
        f'<hp:curSz width="{size}" height="{size}"/>'
        '<hp:flip horizontal="0" vertical="0"/>'
        f'<hp:rotationInfo angle="0" centerX="{size // 2}" centerY="{size // 2}" '
        'rotateimage="1"/>'
        '<hp:renderingInfo><hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
        '<hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>'
        '<hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/></hp:renderingInfo>'
        '<hc:img binaryItemIDRef="image1" bright="0" contrast="0" effect="REAL_PIC" '
        'alpha="0"/>'
        f'<hp:imgRect><hc:pt0 x="0" y="0"/><hc:pt1 x="{size}" y="0"/>'
        f'<hc:pt2 x="{size}" y="{size}"/><hc:pt3 x="0" y="{size}"/></hp:imgRect>'
        '<hp:inMargin left="0" right="0" top="0" bottom="0"/>'
        f'<hp:imgDim dimwidth="{size}" dimheight="{size}"/><hp:effects/>'
        f'<hp:sz width="{size}" widthRelTo="ABSOLUTE" height="{size}" '
        'heightRelTo="ABSOLUTE" protect="0"/>'
        '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="0" '
        'allowOverlap="1" holdAnchorAndSO="0" '
        f'vertRelTo="{vert_rel}" horzRelTo="{horz_rel}" vertAlign="TOP" '
        f'horzAlign="LEFT" vertOffset="{vert_offset}" horzOffset="{horz_offset}"/>'
        '<hp:outMargin left="0" right="0" top="0" bottom="0"/>'
        f"<hp:shapeComment>{comment}</hp:shapeComment></hp:pic>"
    )


def png_2x2() -> bytes:
    """A deterministic 2x2 opaque grey PNG (no external image data)."""

    def chunk(kind: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(kind + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", crc)

    header = struct.pack(">IIBBBBB", 2, 2, 8, 0, 0, 0, 0)  # 8-bit greyscale
    raw = b"\x00\x80\x80" * 2
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def section_prefix(template: str) -> tuple[str, str]:
    """Split ref_empty's section into (root + first paragraph run head, tail).

    The first paragraph carries `secPr`; fixtures reuse it and append their
    own text and following paragraphs.
    """
    marker = '<hp:run charPrIDRef="0"><hp:t/></hp:run>'
    if template.count(marker) != 1:
        raise RuntimeError("ref_empty section0.xml changed: secPr paragraph marker missing")
    head, tail = template.split(marker)
    seg_close = "</hp:linesegarray></hp:p>"
    if not tail.endswith(seg_close + "</hs:sec>"):
        raise RuntimeError("ref_empty section0.xml changed: unexpected tail")
    return head, "</hs:sec>"


def first_paragraph(head: str, text: str) -> str:
    return (
        f'{head}<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'
        f"<hp:linesegarray>{lineseg(0, BODY_WIDTH)}</hp:linesegarray></hp:p>"
    )


def fixture_square_table(head: str, end: str) -> str:
    """A: paper-anchored Square table spanning the whole body width.

    Paragraph 1 is empty and only anchors the table. The table leaves no side
    lane (both lanes 0 px), so its anchor line and the next paragraph must
    come after the table bottom instead of overlapping the first row.
    """
    row_h = 3000
    rows = [
        [
            cell(
                r,
                0,
                BODY_WIDTH,
                row_h,
                paragraph(f"Row {r + 1}", lineseg(0, BODY_WIDTH - 1020)),
                has_margin=False,
                margin=(510, 510, 141, 141),
            )
        ]
        for r in range(2)
    ]
    tbl = table(
        rows,
        width=BODY_WIDTH,
        height=row_h * 2,
        wrap="SQUARE",
        treat_as_char=False,
        vert_rel="PAPER",
        horz_rel="PAPER",
        vert_offset=12000,
        horz_offset=BODY_LEFT,
    )
    return (
        first_paragraph(head, "Heading")
        + paragraph("", lineseg(1600, BODY_WIDTH), tbl)
        + paragraph("Next paragraph", lineseg(3200, BODY_WIDTH))
        + end
    )


def fixture_cell_picture(head: str, end: str) -> str:
    """B: pictures anchored in cells, positioned relative to the paper / page.

    Cell 0 holds a paper-relative picture, cell 1 a page(body)-relative one.
    Both cells start away from the reference origin, so measuring the offset
    from the cell instead of the paper/page shows up as a large x/y shift.
    """
    col_w = BODY_WIDTH // 2
    row_h = 3000
    pad = (510, 510, 141, 141)
    paper_pic = picture(
        201,
        size=3000,
        vert_rel="PAPER",
        horz_rel="PAPER",
        vert_offset=30000,
        horz_offset=15000,
        comment="paper-relative picture in cell 0",
    )
    page_pic = picture(
        202,
        size=3000,
        vert_rel="PAGE",
        horz_rel="PAGE",
        vert_offset=24000,
        horz_offset=6000,
        comment="page-relative picture in cell 1",
    )
    cells = [
        cell(
            0,
            0,
            col_w,
            row_h,
            paragraph("Paper", lineseg(0, col_w - 1020), paper_pic),
            has_margin=False,
            margin=pad,
        ),
        cell(
            0,
            1,
            col_w,
            row_h,
            paragraph("Page", lineseg(0, col_w - 1020), page_pic),
            has_margin=False,
            margin=pad,
        ),
    ]
    tbl = table(
        [cells],
        width=col_w * 2,
        height=row_h,
        wrap="TOP_AND_BOTTOM",
        treat_as_char=True,
        vert_rel="PARA",
        horz_rel="PARA",
        vert_offset=0,
        horz_offset=0,
    )
    return (
        first_paragraph(head, "Heading")
        + paragraph("", lineseg(1600, BODY_WIDTH, vertsize=row_h, spacing=600), tbl)
        + paragraph("Tail", lineseg(1600 + row_h + 600, BODY_WIDTH))
        + end
    )


def fixture_fallback_padding(head: str, end: str) -> str:
    """C1: paper-anchored table whose cell vertical padding is only a fallback.

    Table inMargin is all zero (unspecified) and cells have hasMargin="0", so
    the stored cell margin is the #2195 fallback. The single stored line
    (1000 HU) fits the declared row height (1800 HU); the fallback padding
    (850 + 850 HU) must not grow the row.
    """
    row_h = 1800
    rows = [
        [
            cell(
                r,
                0,
                BODY_WIDTH,
                row_h,
                paragraph(f"Line {r + 1}", lineseg(0, BODY_WIDTH - 1020, spacing=300)),
                has_margin=False,
                margin=(510, 510, 850, 850),
            )
        ]
        for r in range(3)
    ]
    tbl = table(
        rows,
        width=BODY_WIDTH,
        height=row_h * 3,
        wrap="TOP_AND_BOTTOM",
        treat_as_char=False,
        vert_rel="PAPER",
        horz_rel="PAPER",
        vert_offset=20000,
        horz_offset=BODY_LEFT,
    )
    return first_paragraph(head, "Heading") + paragraph("", lineseg(1600, BODY_WIDTH), tbl) + end


def fixture_residual_padding_pair(head: str, end: str) -> str:
    """C2: body-flow table whose stored cell vertical padding is a residual pair.

    hasMargin="0" and the table inMargin is all zero. The stored pair is
    top 20424 / bottom 1287 HU: the top axis is above the 2500 HU sanity limit,
    so the pair is residue and neither axis may grow the 1740 HU row.
    """
    row_h = 1740
    rows = [
        [
            cell(
                0,
                0,
                BODY_WIDTH,
                row_h,
                paragraph("Residual", lineseg(0, BODY_WIDTH - 1020, spacing=300)),
                has_margin=False,
                margin=(510, 510, 20424, 1287),
            )
        ]
    ]
    tbl = table(
        rows,
        width=BODY_WIDTH,
        height=row_h,
        wrap="TOP_AND_BOTTOM",
        treat_as_char=False,
        vert_rel="PARA",
        horz_rel="COLUMN",
        vert_offset=0,
        horz_offset=0,
    )
    return (
        first_paragraph(head, "Heading")
        + paragraph("", lineseg(1600, BODY_WIDTH, vertsize=row_h, spacing=600), tbl)
        + paragraph("Tail", lineseg(1600 + row_h + 600, BODY_WIDTH))
        + end
    )


FIXTURES = {
    "square-table-next-line.hwpx": (fixture_square_table, False),
    "cell-picture-page-paper.hwpx": (fixture_cell_picture, True),
    "paper-table-fallback-padding.hwpx": (fixture_fallback_padding, False),
    "residual-cell-padding-pair.hwpx": (fixture_residual_padding_pair, False),
}


def zip_info(name: str, compression: int) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(name, ZIP_TIMESTAMP)
    info.compress_type = compression
    info.external_attr = 0o100644 << 16
    return info


def write_fixture(path: Path, entries: dict[str, bytes], order: list[str]) -> None:
    with zipfile.ZipFile(path, "w") as out:
        for name in order:
            compression = zipfile.ZIP_STORED if name == "mimetype" else zipfile.ZIP_DEFLATED
            out.writestr(zip_info(name, compression), entries[name])


def main() -> int:
    with zipfile.ZipFile(SOURCE) as source:
        base = {name: source.read(name) for name in source.namelist()}
        base_order = source.namelist()

    head, end = section_prefix(base["Contents/section0.xml"].decode("utf-8"))
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    manifest = []
    for name, (build, needs_image) in FIXTURES.items():
        entries = dict(base)
        order = list(base_order)
        entries["Contents/section0.xml"] = build(head, end).encode("utf-8")
        if needs_image:
            content = entries["Contents/content.hpf"].decode("utf-8")
            item = (
                '<opf:item id="image1" href="BinData/image1.png" '
                'media-type="image/png" isEmbeded="1"/>'
            )
            anchor = '<opf:item id="section0"'
            if content.count(anchor) != 1:
                raise RuntimeError("ref_empty content.hpf changed: section0 item missing")
            entries["Contents/content.hpf"] = content.replace(anchor, item + anchor, 1).encode(
                "utf-8"
            )
            entries["BinData/image1.png"] = png_2x2()
            order.insert(order.index("Contents/section0.xml"), "BinData/image1.png")
        path = OUT_DIR / name
        write_fixture(path, entries, order)
        data = path.read_bytes()
        manifest.append(
            {
                "path": name,
                "bytes": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
            }
        )
    print(json.dumps(manifest, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
