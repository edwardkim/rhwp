#!/usr/bin/env python3
"""개요 탐색(`get_outline_navigation`) 검증용 HWPX fixture 를 만든다.

두 개를 낸다.

- `outline_navigation_table_cell_number.hwpx` — 리뷰 지적을 좁게 재현하는 최소 fixture.
- `outline_navigation_panel_demo.hwpx` — 패널을 실제로 눌러 볼 데모. 3수준 계층 15개
  개요가 3쪽에 걸쳐 있어 접기/펼치기·방향키 훑기·이동(스크롤)을 모두 확인할 수 있고,
  가운데에 표 셀 번호 경계도 들어 있다.

최소 fixture 는 한 파일 안에 두 경계를 함께 담는다.

1. 개요 계층 — `개요 1`(수준 0) 아래 `개요 2`(수준 1). 목록이 계층으로 접히는지,
   번호가 `1.` / `가.` 로 확장되는지 확인한다.
2. 표 셀 번호 문단 — 앞 개요 `1.` 과 뒤 개요 사이에 **같은 번호 정의(id 1)** 를 쓰는
   `NUMBER` 문단을 표 셀에 넣는다. 렌더러는 이 셀 문단으로 카운터를 전진시키므로 뒤
   개요는 `3.` 이다. 셀 문단을 건너뛰는 구현은 `2.` 를 내놓아 화면과 어긋난다.

정규식 유혹용으로 `1. 일반 본문` 문단도 한 줄 넣는다 — 개요 속성이 없으므로 탐색
목록에 나오면 안 된다.

`samples/hwpx/ref/ref_empty.hwpx`(한컴 2020 이 저장한 빈 문서)의 머리말·번호 정의를
그대로 쓰고 본문만 합성한다. 산출물은 결정적이다(zip 타임스탬프 고정).

산출물은 저장소에 커밋되어 있으므로 테스트는 이 스크립트를 실행하지 않는다 — 재생성할
때만 기준 문서가 필요하다. PR #4093 의 확인용 자료를 한자리에 모으려고
`samples/pr4093/` 에 둔다 — `samples/` 아래는 overflow-cell 원장의 전수 대상이므로
fixture 를 바꾸면 `tests/overflow_cell_baseline.rs` 게이트를 함께 돌린다
(`mydocs/manual/pr_review/local_validation.md` 4.3.1).
"""

from __future__ import annotations

import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "samples" / "hwpx" / "ref" / "ref_empty.hwpx"
OUTPUT_DIR = ROOT / "samples" / "pr4093"
OUTPUT = OUTPUT_DIR / "outline_navigation_table_cell_number.hwpx"
DEMO_OUTPUT = OUTPUT_DIR / "outline_navigation_panel_demo.hwpx"
ZIP_TIMESTAMP = (1980, 1, 1, 0, 0, 0)

# ref_empty 의 개요 문단 모양 — paraPr 2~4 가 개요 1~3 수준이고 style 도 같은 번호다.
# 번호 정의 id 1 의 서식은 수준별로 `^1.` / `^2.`(HANGUL_SYLLABLE) / `^3)` 이라
# 화면에는 `1.` / `가.` / `1)` 로 나온다.
OUTLINE_LEVELS = {0: (2, 2), 1: (3, 3), 2: (4, 4)}
BODY_PARA_PR = 0

# ref_empty 의 paraPr 는 0~19 다. 20 번으로 "개요와 같은 번호 정의를 쓰는 NUMBER 문단"을
# 추가한다 — paraPr 2(개요 1)에서 heading 만 NUMBER/idRef=1 로 바꾼 사본이다.
NUMBER_PARA_PR_ID = 20
NUMBER_PARA_PR = (
    f'<hh:paraPr id="{NUMBER_PARA_PR_ID}" tabPrIDRef="1" condense="20" fontLineHeight="0"'
    ' snapToGrid="1" suppressLineNumbers="0" checked="0">'
    '<hh:align horizontal="JUSTIFY" vertical="BASELINE"/>'
    '<hh:heading type="NUMBER" idRef="1" level="0"/>'
    '<hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="KEEP_WORD" widowOrphan="0"'
    ' keepWithNext="0" keepLines="0" pageBreakBefore="0" lineWrap="BREAK"/>'
    '<hh:autoSpacing eAsianEng="0" eAsianNum="0"/>'
    '<hh:margin><hc:intent value="0" unit="HWPUNIT"/><hc:left value="2000" unit="HWPUNIT"/>'
    '<hc:right value="0" unit="HWPUNIT"/><hc:prev value="0" unit="HWPUNIT"/>'
    '<hc:next value="0" unit="HWPUNIT"/></hh:margin>'
    '<hh:lineSpacing type="PERCENT" value="160" unit="HWPUNIT"/>'
    '<hh:border borderFillIDRef="2" offsetLeft="0" offsetRight="0" offsetTop="0"'
    ' offsetBottom="0" connect="0" ignoreMargin="0"/>'
    "</hh:paraPr>"
)


def lineseg(horz_size: int) -> str:
    return (
        '<hp:linesegarray><hp:lineseg textpos="0" vertpos="0" vertsize="1000" textheight="1000"'
        f' baseline="850" spacing="600" horzpos="0" horzsize="{horz_size}" flags="393216"/>'
        "</hp:linesegarray>"
    )


def paragraph(
    para_id: int,
    para_pr: int,
    style: int,
    text: str,
    horz_size: int = 42520,
    page_break: bool = False,
) -> str:
    return (
        f'<hp:p id="{para_id}" paraPrIDRef="{para_pr}" styleIDRef="{style}"'
        f' pageBreak="{1 if page_break else 0}" columnBreak="0" merged="0">'
        f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'
        f"{lineseg(horz_size)}</hp:p>"
    )


def table_host_paragraph(para_id: int, cell_paragraph: str) -> str:
    """`cell_paragraph` 하나만 담은 1×1 표를 품은 문단."""
    cell = (
        '<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0"'
        ' borderFillIDRef="2">'
        '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER"'
        ' linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0"'
        f' hasNumRef="0">{cell_paragraph}</hp:subList>'
        '<hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/>'
        '<hp:cellSz width="41952" height="1200"/>'
        '<hp:cellMargin left="510" right="510" top="141" bottom="141"/>'
        "</hp:tc>"
    )
    table = (
        '<hp:tbl id="1094795585" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM"'
        ' textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0"'
        ' rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="2" noAdjust="0">'
        '<hp:sz width="41952" widthRelTo="ABSOLUTE" height="1200" heightRelTo="ABSOLUTE"'
        ' protect="0"/>'
        '<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0"'
        ' holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="COLUMN" vertAlign="TOP"'
        ' horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
        '<hp:outMargin left="283" right="283" top="283" bottom="283"/>'
        '<hp:inMargin left="510" right="510" top="141" bottom="141"/>'
        f"<hp:tr>{cell}</hp:tr></hp:tbl>"
    )
    return (
        f'<hp:p id="{para_id}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0"'
        ' merged="0">'
        f'<hp:run charPrIDRef="0">{table}</hp:run>'
        f"{lineseg(42520)}</hp:p>"
    )


def replace_once(value: str, before: str, after: str, label: str) -> str:
    if value.count(before) != 1:
        raise RuntimeError(f"{label} marker must appear exactly once")
    return value.replace(before, after, 1)


def zip_info(name: str, compression: int) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(name, ZIP_TIMESTAMP)
    info.compress_type = compression
    info.external_attr = 0o100644 << 16
    return info


def write_fixture(output: Path, body: str) -> None:
    """`ref_empty` 의 머리말·구역 설정을 그대로 쓰고 본문만 갈아 끼운 HWPX 를 쓴다."""
    with zipfile.ZipFile(SOURCE) as source:
        entries = {name: source.read(name) for name in source.namelist()}

    header = entries["Contents/header.xml"].decode("utf-8")
    header = replace_once(
        header, '<hh:paraProperties itemCnt="20">', '<hh:paraProperties itemCnt="21">', "paraPr 개수"
    )
    header = replace_once(
        header, "</hh:paraProperties>", f"{NUMBER_PARA_PR}</hh:paraProperties>", "paraPr 목록 끝"
    )
    entries["Contents/header.xml"] = header.encode("utf-8")

    # 문단 0 은 secPr 를 품은 원본 빈 문단이라 그대로 두고 그 뒤에 본문을 붙인다.
    section = entries["Contents/section0.xml"].decode("utf-8")
    section = replace_once(section, "</hs:sec>", f"{body}</hs:sec>", "구역 끝")
    entries["Contents/section0.xml"] = section.encode("utf-8")

    with zipfile.ZipFile(output, "w") as archive:
        # mimetype 은 HWPX 규약상 첫 항목이자 무압축이어야 한다.
        archive.writestr(zip_info("mimetype", zipfile.ZIP_STORED), entries.pop("mimetype"))
        for name, payload in entries.items():
            archive.writestr(zip_info(name, zipfile.ZIP_DEFLATED), payload)

    print(f"wrote {output.relative_to(ROOT)}")


def minimal_body() -> str:
    """리뷰 지적을 좁게 재현하는 본문 5문단."""
    return "".join(
        [
            # 1 → "1."
            paragraph(2000, 2, 2, "개요"),
            # 2 → "가."
            paragraph(2001, 3, 3, "목적"),
            # 3 → 개요 속성이 없어 목록에 나오면 안 된다.
            paragraph(2002, 0, 0, "1. 일반 본문"),
            # 4 → 표 셀의 NUMBER 문단이 카운터를 2 로 전진시킨다(목록에는 없다).
            table_host_paragraph(
                2003, paragraph(2004, NUMBER_PARA_PR_ID, 0, "표 셀 번호 문단", 39000)
            ),
            # 5 → 셀 번호를 지나온 뒤라 "3."
            paragraph(2005, 2, 2, "요구사항"),
        ]
    )


def demo_body() -> str:
    """패널을 눌러 볼 데모 본문 — 3수준 개요 15개가 3쪽에 걸쳐 있다.

    개요마다 본문 두 줄을 딸려 보내 이동(스크롤)이 눈에 보이게 하고, 3쪽 머리에서
    표 셀 번호 경계를 한 번 더 통과시킨다 — 패널 번호와 본문 번호가 같아야 한다.
    """
    parts: list[str] = []
    para_id = 3000

    def add(level: int | None, text: str, page_break: bool = False) -> None:
        nonlocal para_id
        para_pr, style = OUTLINE_LEVELS[level] if level is not None else (BODY_PARA_PR, 0)
        parts.append(paragraph(para_id, para_pr, style, text, page_break=page_break))
        para_id += 1

    def body_lines(topic: str) -> None:
        add(None, f"{topic} 설명 문단 — 개요를 눌렀을 때 이 자리로 스크롤한다.")
        add(None, f"{topic} 보충 문단 — 방향키로 훑는 동안에는 화면이 움직이지 않아야 한다.")

    # 1쪽 — 1. 아래 2·3수준
    add(0, "총칙")
    body_lines("총칙")
    add(1, "목적")
    body_lines("목적")
    add(2, "배경")
    body_lines("배경")
    add(2, "적용 범위")
    body_lines("적용 범위")
    add(1, "용어 정의")
    body_lines("용어 정의")

    # 2쪽 — 같은 계층이 한 수준 되감기는지 본다
    add(0, "본문 규정", page_break=True)
    body_lines("본문 규정")
    add(1, "요구사항")
    body_lines("요구사항")
    add(2, "기능 요구")
    body_lines("기능 요구")
    add(2, "비기능 요구")
    body_lines("비기능 요구")
    add(1, "제약 조건")
    body_lines("제약 조건")

    # 3쪽 — 표 셀 번호 경계. 셀 문단이 4 를 가져가므로 뒤 개요는 5. 다.
    add(0, "표가 낀 구간", page_break=True)
    body_lines("표가 낀 구간")
    parts.append(
        table_host_paragraph(para_id, paragraph(para_id + 1, NUMBER_PARA_PR_ID, 0, "표 셀 번호 문단", 39000))
    )
    para_id += 2
    add(1, "표 뒤 하위 개요")
    body_lines("표 뒤 하위 개요")
    add(0, "부칙")
    body_lines("부칙")
    add(1, "시행일")
    body_lines("시행일")
    add(1, "경과 조치")
    body_lines("경과 조치")

    return "".join(parts)


def main() -> int:
    if not SOURCE.is_file():
        raise RuntimeError(f"기준 문서가 없다: {SOURCE}")

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    write_fixture(OUTPUT, minimal_body())
    write_fixture(DEMO_OUTPUT, demo_body())
    return 0


if __name__ == "__main__":
    sys.exit(main())
