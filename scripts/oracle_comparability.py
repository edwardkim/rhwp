#!/usr/bin/env python3
"""기준 PDF 가 그 문서와 **비교 가능한지** 판정한다 (#7396).

시각 검증은 "rhwp 출력이 정본과 다르다" 를 결함으로 읽는다. 그런데 정본 PDF 는
**그것을 출력한 장비의 글꼴 환경**에서 한/글이 다시 짠 조판일 수 있다. 그러면 같은
차이가 rhwp 결함이 아니라 기준의 사정이다. 이 스크립트는 그 구분을 착수 **전에**
기계적으로 낸다.

판정 근거 다섯 가지를 각각 수치로 낸다.

  paperBox        정본 MediaBox 와 rhwp 쪽 상자가 같은가. 다르면 다른 건 볼 것도 없다.
  declaredFaces   charPr 이 참조한 face 가 정본에 임베드됐는가. 빠졌으면 한/글이
                  **대체**했고, 그 슬롯의 폭·줄나눔은 선언 글꼴의 것이 아니다.
                  이름은 반드시 정규화한다 — 서브셋 접두(`INPILL+`), 굵기 접미
                  (`-Bold`), 공백 유무(`PalatinoLinotype`)가 전부 다르게 적힌다.
  storedLines     저장 LineSeg 의 끊음을 rhwp 가 재현하는가. 재현한다면 그 문단은
                  측정 경로를 타지 않으므로 **폭을 고쳐도 줄이 안 바뀐다**.
                  판정은 총합이 아니라 `oracleKeepsButRhwpMisses` — **정본은 지켰는데
                  rhwp 만 놓친** 문단 — 으로 한다. 둘이 서로 다른 문단을 놓쳐도 총합은
                  같아질 수 있어 방향을 못 준다.
  oracleRetypeset 정본의 끊음이 저장 LineSeg 와 다른가. 다르면 정본이 재조판했다.
                  rhwp 가 저장을 지키고 정본이 재조판했다면 **둘 다 옳을 수 있다**.
  fontScale       같은 글자열 줄에서 정본과 rhwp 의 글꼴 크기 비. 1.0 에서 벗어나면
                  글자폭 비교가 통째로 기울어진다.
  contentScale    `정본 x = a·rhwp x + b` 의 기울기 `a`. **글꼴 비와 같이 움직이면**
                  rhwp 결함이 아니라 기준 산출물의 **인쇄 배율**이다(축소·확대, 용지에
                  맞추기). 글꼴만 움직이고 위치가 그대로일 때만 글자 크기 결함이다.

종료 코드는 언제나 0 이다 — 판정은 데이터이고, 게이트가 아니다. 90% 실루엣 게이트의
예외는 정책대로 `--font-mismatch-evidence` 로만 남긴다. 대표 review PNG 직접 판독을
대신하지도 않는다.

    python3 scripts/oracle_comparability.py \
      samples/exam_eng.hwp pdf/exam_eng-2022.pdf \
      --rhwp-bin target/pr-review/release/rhwp

저장소의 정본 4쌍으로 확인한 결과다 (`RHWP_FONT_PATH=ttfs/hwp:ttfs/windows`).

    1382000_domestic_violence_survey.hwp   rhwp_저장줄_이탈  정본이 저장 끊음을 9문단 더 지킴
    exam_eng.hwp                           글꼴크기_불일치   정본/rhwp = 1.0643 (짝 660줄)
    hwpctl_API_v2.4.hwp                    비교가능_주의     선언 face 11개 중 10개 대체
    1341000_research_report_footnotes.hwp  비교가능_주의     선언 face 24개 중 21개 대체

`declaredFaces` 는 **과잉 보고 쪽으로 기운다** — 이름 없이 임베드된 서브셋 글꼴
(mutool 이 `T2`·`T3` 로 표시)은 이름으로 판별할 수 없다. 그래서 이 축은 판정에서
최하위 등급(`비교가능_주의`)으로만 쓰고, 확정은 전진폭 대조로 한다.

## 줄은 **양쪽에 같은 규칙**으로 만든다

`storedLines` 의 초판은 rhwp 쪽을 쪽 전체 baseline 으로 뭉치고 정본 쪽은 mutool 의
`<line>` 을 썼다. 표 문서에서 그 둘은 서로 **다르게** 칸을 합쳐서, 같은 문서에 대해
두 수치가 비대칭으로 어긋났다. `issue1853_caption_precedes_body_split.hwpx` 실측:

    수정 전   rhwp 1,331줄(중앙 23자)   정본 1,785줄(중앙 16자)
    수정 후   rhwp 1,559줄(중앙 17자)   정본 1,539줄(중앙 17자)

지금은 양쪽 다 `<char>`/`<text>` 의 baseline 으로 모으고 `split_baseline_row` 하나로
끊는다. 정본 290쌍 전수에서 이 교정이 큐를 이렇게 바꾼다.

    rhwp_저장줄_이탈  13 -> 8
      빠진 6문서 중 5문서는 `oracleKeepsButRhwpMisses = 0` (전부 artifact)
      새로 들어온 hwpspec.hwp 는 45문단 — 종전 규칙이 `저장줄_재현`으로 가렸다
"""

from __future__ import annotations

import argparse
import collections
import html
import json
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SUBSET_PREFIX = re.compile(r"^[A-Z]{6}\+")
WEIGHT_SUFFIX = re.compile(
    r"[-,]?(BoldItalic|Bold|Italic|Regular|Medium|Light|PSMT|PS|MT)$"
)
NON_NAME = re.compile(r"[\s\-_]")


class Unavailable(RuntimeError):
    """외부 도구나 입력이 없어 그 축을 잴 수 없다."""


def normalize_face(name: str) -> str:
    """정본과 원문의 face 이름을 같은 자리에서 비교할 수 있게 접는다."""
    name = SUBSET_PREFIX.sub("", name or "")
    # mutool 은 CP949 이름을 latin-1 바이트로 흘린다.
    try:
        name = name.encode("latin-1").decode("cp949")
    except (UnicodeEncodeError, UnicodeDecodeError):
        pass
    if re.fullmatch(r"[0-9A-Fa-f]{6,}", name):
        try:
            name = bytes.fromhex(name).decode("cp949")
        except (ValueError, UnicodeDecodeError):
            pass
    name = WEIGHT_SUFFIX.sub("", name)
    return NON_NAME.sub("", name).lower()


def run(cmd: list[str], timeout: int = 900) -> bytes:
    try:
        done = subprocess.run(cmd, capture_output=True, timeout=timeout)
    except FileNotFoundError as exc:
        raise Unavailable(f"실행 파일 없음: {cmd[0]}") from exc
    except subprocess.TimeoutExpired as exc:
        raise Unavailable(f"시간 초과: {' '.join(cmd[:2])}") from exc
    if done.returncode != 0 and not done.stdout:
        raise Unavailable(
            f"{cmd[0]} 실패(rc={done.returncode}): "
            f"{done.stderr.decode('utf-8', 'replace').strip()[:200]}"
        )
    return done.stdout


# ---------------------------------------------------------------- 원문 읽기


def read_source(rhwp_bin: Path, doc: Path, work: Path) -> dict:
    """HWPX 로 내보내 선언 face·charPr 참조·저장 LineSeg·문단 글자열을 읽는다."""
    out = work / "comparability.hwpx"
    run([str(rhwp_bin), "export-hwpx", str(doc), str(out)])
    if not out.exists():
        raise Unavailable(f"HWPX 변환 실패: {doc}")
    with zipfile.ZipFile(out) as zf:
        header = zf.read("Contents/header.xml").decode("utf-8", "replace")
        sections = [
            zf.read(name).decode("utf-8", "replace")
            for name in sorted(zf.namelist())
            if re.match(r"Contents/section\d+\.xml", name)
        ]

    tables: dict[str, dict[int, str]] = {}
    for block in re.finditer(
        r'<hh:fontface[^>]*lang="(\w+)"[^>]*>(.*?)</hh:fontface>', header, re.S
    ):
        tables[block.group(1)] = {
            int(i): face
            for i, face in re.findall(r'id="(\d+)" face="([^"]*)"', block.group(2))
        }
    slots = ("HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER")
    attrs = ("hangul", "latin", "hanja", "japanese", "other", "symbol", "user")

    referenced: set[str] = set()
    for char_pr in re.finditer(r'<hh:charPr id="\d+"[^>]*>.*?</hh:charPr>', header, re.S):
        ref = re.search(r"<hh:fontRef ([^/]*)/>", char_pr.group(0))
        if not ref:
            continue
        for lang, attr in zip(slots, attrs):
            got = re.search(rf'{attr}="(\d+)"', ref.group(1))
            if got:
                face = tables.get(lang, {}).get(int(got.group(1)))
                if face:
                    referenced.add(face)

    paragraphs = []
    for body in sections:
        # `<hp:p>` 는 **중첩된다** — 표를 담은 문단 안에 칸 문단이 들어간다. 비탐욕
        # 정규식으로 짝지으면 바깥 문단이 첫 칸 문단의 `</hp:p>` 에서 닫혀, 그 문단의
        # 글자열과 컷이 서로 다른 문단의 것으로 섞인다. 깊이로 정확히 짝짓는다.
        opened: list[int] = []
        for tag in re.finditer(r"<hp:p\b[^>]*?(/?)>|</hp:p>", body):
            if tag.group(0).startswith("</"):
                if not opened:
                    continue
                inner = body[opened.pop() : tag.start()]
            elif tag.group(1) != "/":
                opened.append(tag.end())
                continue
            else:
                continue
            text = html.unescape(
                re.sub(
                    r"<[^>]*>",
                    "",
                    "".join(re.findall(r"<hp:t[^>]*>(.*?)</hp:t>", inner, re.S)),
                )
            )
            cuts = [
                int(pos)
                for pos in re.findall(r'<hp:lineseg textpos="(\d+)"', inner)
            ]
            if len(text.strip()) >= 12:
                paragraphs.append({"text": text, "cuts": cuts})
    return {"referenced_faces": sorted(referenced), "paragraphs": paragraphs}


# ---------------------------------------------------------------- 출력 읽기


# 한 baseline 위의 글자를 가로 공백으로 끊는 폭(글꼴 크기 배수).
#
# 쪽 전체를 baseline 하나로 뭉치면 **표의 여러 칸이 한 줄이 된다**. 그러면
# `judge_lines` 의 "머리 글자열로 끝나는 줄이 있는가" 가 우연히 참이 되고, rhwp(SVG)와
# 정본(stext)의 뭉치는 방식이 서로 달라 두 수치가 **비대칭으로** 어긋난다. 실측:
# `22037757-chuncheon-personnel-rule-annex13.hwpx` 가 rhwp 102 / 정본 116 으로 나왔는데,
# 같은 규칙을 양쪽에 걸면 rhwp 110 / 정본 101 로 **뒤집힌다**(그 문서에서 "정본은 지키고
# rhwp 만 놓친" 문단은 0건이다).
#
# 1.5 는 칸 사이 여백과 자간·정렬 늘림을 가르는 값이다. 양쪽 끝 정렬로 벌어진 낱말
# 사이는 한 글자 폭을 넘기 어렵고, 표 칸 사이 안여백은 그보다 넓다.
LINE_SPLIT_GAP_EM = 1.5


def split_baseline_row(row):
    """한 baseline 의 `(x, glyph, font_px)` 를 가로 공백에서 끊어 여러 줄로 낸다.

    **양쪽 입력에 같은 규칙으로 적용한다** — 그것이 이 함수가 있는 이유다.
    """
    row = sorted(row, key=lambda item: item[0])
    lines, current, previous_end = [], [], None
    for x, glyph, font_px in row:
        if previous_end is not None and x - previous_end > LINE_SPLIT_GAP_EM * font_px:
            if current:
                lines.append(current)
            current = []
        current.append((x, glyph, font_px))
        # 한 항목이 여러 글자를 담을 수 있다(rhwp `<text>` run). 끝점은 그 길이를 쓴다.
        previous_end = x + font_px * len(glyph)
    if current:
        lines.append(current)
    return lines


def comparison_key(text: str) -> str:
    """두 출력의 글자열을 견주기 위한 열쇠.

    공백과 함께 **사용자 정의 영역(PUA)과 옛한글 자모**를 버린다. 같은 글자를 한/글과
    rhwp 가 서로 다른 코드로 적기 때문이다 — `hwpspec.hwp` 의 「ᄒᆞᆫ글」 로고 글자를
    정본은 `U+F53A` 하나로, rhwp 는 자모 셋(`U+1112 U+119E U+11AB`)으로 적는다.
    글자열이 다르니 어떤 대조도 성립하지 않는다.

    **너비 문제가 아니다.** rhwp 는 그 묶음을 0.9259 em 으로 전진시켜 한 음절 폭을
    쓴다. 버리고 다시 세면 그 문서의 "정본만 지킨 문단" 이 45 -> 8 로 줄어든다 —
    37건이 표기 차이였다.

    매핑표 대신 버리는 쪽을 쓴다. 표는 글꼴·판본마다 달라 유지할 근거가 없고, 양쪽에서
    똑같이 버리면 `endswith` 대조는 그대로 성립한다.
    """
    return "".join(
        c
        for c in text
        if not c.isspace()
        and not 0xE000 <= ord(c) <= 0xF8FF
        and not 0x1100 <= ord(c) <= 0x11FF
        and not 0xA960 <= ord(c) <= 0xA97F
        and not 0xD7B0 <= ord(c) <= 0xD7FF
    )


def row_to_line(row, baseline):
    key = comparison_key("".join(glyph for _, glyph, _ in row))
    if not key:
        return None
    return {"key": key, "font_px": row[0][2], "x": row[0][0], "y": baseline}


def read_pdf(pdf: Path, pages: list[int] | None) -> dict:
    """정본의 쪽 상자·임베드 글꼴·줄 글자열·글꼴 크기를 읽는다."""
    args = ["mutool", "draw", "-F", "stext", "-o", "-", str(pdf)]
    if pages:
        args.append(",".join(str(p) for p in pages))
    text = run(args).decode("utf-8", "replace")

    boxes, lines = [], []
    for page in re.finditer(r"<page\b([^>]*)>(.*?)</page>", text, re.S):
        width = re.search(r'width="([\d.]+)"', page.group(1))
        height = re.search(r'height="([\d.]+)"', page.group(1))
        if width and height:
            boxes.append((float(width.group(1)), float(height.group(1))))
        # mutool 의 `<line>` 을 쓰지 않는다. 그 묶음은 rhwp SVG 쪽에 대응물이 없어
        # **두 입력의 줄 정의가 달라지는 지점**이었다(표의 이웃 칸을 한쪽만 묶는다).
        # 양쪽 모두 baseline 으로 모으고 같은 가로 공백 규칙으로 끊는다.
        rows = collections.defaultdict(list)
        size = 0.0
        for chunk in re.split(r"(<font[^>]*>)", page.group(2)):
            got = re.match(r'<font [^>]*size="([^"]*)"', chunk)
            if got:
                size = float(got.group(1)) * 96.0 / 72.0
                continue
            # `quad` 의 위 모서리가 아니라 `<char x= y=>` 의 **원점(baseline)** 으로 모은다.
            # 한 줄에 크기가 다른 글자가 섞이면 quad 의 위 모서리는 글자마다 달라져
            # 같은 줄이 여러 baseline 으로 흩어진다.
            for x, y, c in re.findall(
                r'<char [^>]*?x="([\d.-]+)" y="([\d.-]+)"[^>]*?c="([^"]*)"', chunk
            ):
                rows[round(float(y) * 96.0 / 72.0, 1)].append(
                    (float(x) * 96.0 / 72.0, html.unescape(c), size)
                )
        for baseline, row in sorted(rows.items()):
            for segment in split_baseline_row(row):
                got = row_to_line(segment, baseline)
                if got:
                    lines.append(got)

    embedded = set()
    # CP949 face 이름의 바이트를 보존해야 한다 — UTF-8 로 먼저 디코드하면
    # `INPILL+휴먼명조` 가 치환 문자로 뭉개져 "임베드 안 됨" 으로 오판한다.
    info = run(["mutool", "info", "-F", str(pdf)]).decode("latin-1")
    for quoted in re.finditer(r"'([^']+)'", info):
        embedded.add(normalize_face(quoted.group(1)))
    return {"boxes": boxes, "lines": lines, "embedded": embedded}


def read_rhwp(rhwp_bin: Path, doc: Path, work: Path, pages: list[int] | None) -> dict:
    """rhwp SVG 에서 쪽 상자·줄 글자열·글꼴 크기를 읽는다."""
    out = work / "svg"
    args = [str(rhwp_bin), "export-svg", str(doc), "-o", str(out)]
    if pages and len(pages) == 1:
        args += ["-p", str(pages[0] - 1)]
    run(args)
    files = sorted(out.glob("*.svg"))
    if not files:
        raise Unavailable("rhwp SVG 산출 없음")

    boxes, lines = [], []
    for path in files:
        body = path.read_text(encoding="utf-8")
        head = re.search(r'<svg[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"', body)
        if head:
            boxes.append(
                (float(head.group(1)) * 72.0 / 96.0, float(head.group(2)) * 72.0 / 96.0)
            )
        rows = collections.defaultdict(list)
        for run_el in re.finditer(
            r'<text (?:x="([\d.]+)" y="([\d.]+)"|transform="translate\(([\d.]+),([\d.]+)\)[^"]*")'
            r'[^>]*font-size="([\d.]+)"[^>]*>(.*?)</text>',
            body,
            re.S,
        ):
            x = float(run_el.group(1) or run_el.group(3))
            y = float(run_el.group(2) or run_el.group(4))
            font_px = float(run_el.group(5))
            glyphs = html.unescape(re.sub(r"<[^>]*>", "", run_el.group(6)))
            # 한 `<text>` 는 통째로 담는다. 글자마다 x 를 지어내면(`x + n*font_px`)
            # 좁은 글자에서 끝점이 부풀어 칸 사이 공백이 묻히고, 전부 같은 x 로 담아
            # 정렬하면 동점이 글자 코드순으로 갈려 글자열이 뒤섞인다.
            if glyphs:
                rows[round(y, 1)].append((x, glyphs, font_px))
        for baseline, row in sorted(rows.items()):
            for segment in split_baseline_row(row):
                got = row_to_line(segment, baseline)
                if got:
                    lines.append(got)
    return {"boxes": boxes, "lines": lines}


# ---------------------------------------------------------------- 판정


def judge_paper(pdf_boxes, rhwp_boxes) -> dict:
    if not pdf_boxes or not rhwp_boxes:
        return {"status": "미측정", "reason": "쪽 상자를 읽지 못했다"}
    pw, ph = pdf_boxes[0]
    rw, rh = rhwp_boxes[0]
    ratio = max(abs(pw / rw - 1.0), abs(ph / rh - 1.0)) if rw and rh else 1.0
    return {
        "status": "일치" if ratio < 0.01 else "불일치",
        "pdfPt": [round(pw, 1), round(ph, 1)],
        "rhwpPt": [round(rw, 1), round(rh, 1)],
        "maxRelDiff": round(ratio, 4),
    }


def judge_faces(referenced, embedded) -> dict:
    missing = [f for f in referenced if normalize_face(f) not in embedded]
    return {
        "status": "그대로" if not missing else "대체있음",
        "referenced": len(referenced),
        "substituted": len(missing),
        "substitutedFaces": missing[:12],
        "note": (
            "정본이 이 face 들을 임베드하지 않았다 — 한/글이 대체했고, 그 슬롯의 "
            "폭·줄나눔은 선언 글꼴의 것이 아니다"
            if missing
            else ""
        ),
    }


def judge_lines(paragraphs, rhwp_lines, pdf_lines) -> dict:
    """저장 끊음을 rhwp 가 재현하는지, 정본이 재조판했는지 센다."""
    # 줄 앞에 붙는 자동 번호·글머리표는 원문 글자열에 없다. 저장 머리 글자열이
    # 렌더된 줄의 **꼬리**와 같으면 그 끊음을 재현한 것으로 센다.
    def hits(lines, head):
        return any(line["key"].endswith(head) for line in lines)
    stored = rhwp_hit = pdf_hit = oracle_only = 0
    for para in paragraphs:
        cuts = para["cuts"]
        if len(cuts) < 2:
            continue
        text = para["text"]
        head = comparison_key(text[: cuts[1]])
        if len(head) < 12:
            continue
        stored += 1
        mine = hits(rhwp_lines, head)
        theirs = hits(pdf_lines, head)
        rhwp_hit += mine
        pdf_hit += theirs
        # 착수 대상은 이것 하나다 — **정본은 저장 컷을 지켰는데 rhwp 만 놓친** 문단.
        # 총합 비교는 둘이 서로 다른 문단을 놓쳐도 같은 수가 될 수 있어 방향을 못 준다.
        oracle_only += theirs and not mine
    if not stored:
        return {"status": "비해당", "reason": "여러 줄 저장 LineSeg 문단이 없다"}
    return {
        "status": "재현" if rhwp_hit * 2 >= stored else "재조판",
        "storedMultilineParagraphs": stored,
        "rhwpReproducesStoredCut": rhwp_hit,
        "oracleReproducesStoredCut": pdf_hit,
        "oracleKeepsButRhwpMisses": oracle_only,
        "note": (
            "rhwp 가 저장 끊음을 지키고 정본은 다시 짰다 — 줄 차이는 둘 다 옳을 수 있다"
            if rhwp_hit > pdf_hit and stored
            else ""
        ),
    }


def judge_font_scale(rhwp_lines, pdf_lines) -> dict:
    pdf_by_key = {line["key"]: line["font_px"] for line in pdf_lines}
    ratios = []
    for line in rhwp_lines:
        other = pdf_by_key.get(line["key"])
        if other and line["font_px"] > 0:
            ratios.append(other / line["font_px"])
    if len(ratios) < 5:
        return {"status": "미측정", "matchedLines": len(ratios)}
    ratios.sort()
    median = ratios[len(ratios) // 2]
    return {
        "status": "일치" if abs(median - 1.0) < 0.005 else "불일치",
        "matchedLines": len(ratios),
        "medianOracleOverRhwp": round(median, 4),
        "note": (
            "같은 글자열 줄의 글꼴 크기가 다르다 — 글자폭 비교가 통째로 기울어진다"
            if abs(median - 1.0) >= 0.005
            else ""
        ),
    }


def judge_content_scale(rhwp_lines, pdf_lines, font_ratio: float | None) -> dict:
    """내용 전체가 같은 배율로 움직였는지 — 그렇다면 기준 산출물의 인쇄 배율이다.

    글꼴만 움직이고 위치가 제자리면 글자 크기 결함이고, 글꼴·x·y 가 함께 움직이면
    정본이 축소/확대 인쇄된 것이라 rhwp 를 고칠 일이 아니다. 짧은 글자열은 표의 같은
    셀 문구끼리 잘못 짝지어지므로 **정본에 한 번만 나오는 긴 줄**만 센다.
    """
    seen = collections.Counter(line["key"] for line in pdf_lines)
    pdf_by_key = {line["key"]: line for line in pdf_lines}
    xs, ys = [], []
    for line in rhwp_lines:
        other = pdf_by_key.get(line["key"])
        if not other or len(line["key"]) < 12 or seen[line["key"]] != 1:
            continue
        xs.append((line.get("x", 0.0), other.get("x", 0.0)))
        ys.append((line.get("y", 0.0), other.get("y", 0.0)))
    if len(xs) < 8:
        return {"status": "미측정", "matchedLines": len(xs)}

    def slope(pairs: list[tuple[float, float]]) -> float | None:
        """`정본 = a·rhwp + b` 의 기울기. 배율에 가운데 맞춤이 섞여도 `a` 는 남는다."""
        n = len(pairs)
        sx = sum(p[0] for p in pairs)
        sy = sum(p[1] for p in pairs)
        sxx = sum(p[0] * p[0] for p in pairs)
        sxy = sum(p[0] * p[1] for p in pairs)
        denominator = n * sxx - sx * sx
        return None if abs(denominator) < 1e-9 else (n * sxy - sx * sy) / denominator

    x_slope = slope(xs)
    y_slope = slope(ys)
    if x_slope is None or y_slope is None:
        return {"status": "미측정", "matchedLines": len(xs)}
    # 판정은 **x 기울기만** 쓴다. `y` 는 쪽마다 0 으로 되돌아가므로 문서 전체를 한 직선에
    # 맞추면 쪽 경계가 섞여 기울기가 무너진다(`1342000_edu_curriculum_map`: 실제 배율
    # 0.83 인데 통합 적합은 0.65). x 원점은 모든 쪽이 같아 그 문제가 없다. `y` 는
    # 참고로만 싣는다.
    uniform = (
        font_ratio is not None
        and abs(font_ratio - 1.0) >= 0.005
        and abs(x_slope - font_ratio) < 0.02
    )
    return {
        "status": "배율" if uniform else "제자리",
        "matchedLines": len(xs),
        "xSlope": round(x_slope, 4),
        "ySlope": round(y_slope, 4),
        "note": (
            "글꼴과 x 가 같은 비로 움직인다 — 정본이 축소/확대 인쇄된 것이라 "
            "rhwp 결함이 아니다"
            if uniform
            else ""
        ),
    }


def verdict(report: dict) -> tuple[str, str]:
    """막는 축부터 훑는다. 아래로 갈수록 "쓸 수 있되 주의" 에 가깝다."""
    if report["paperBox"]["status"] == "불일치":
        return "쪽상자_불일치", "쪽 상자가 달라 어떤 좌표 비교도 성립하지 않는다"
    if report["fontScale"]["status"] == "불일치":
        ratio = report["fontScale"]["medianOracleOverRhwp"]
        # 글꼴·x·y 가 함께 움직였으면 정본의 인쇄 배율이다 — rhwp 를 고칠 일이 아니다.
        if report.get("contentScale", {}).get("status") == "배율":
            scale = report["contentScale"]
            return (
                "정본_인쇄배율",
                f"정본 내용 전체가 {ratio} 배다(x 기울기 {scale['xSlope']}) — "
                "축소/확대 인쇄본이라 좌표·폭 비교의 기준으로 쓸 수 없다",
            )
        return (
            "글꼴크기_불일치",
            f"같은 글자열 줄의 글꼴 크기가 정본/rhwp = {ratio} 다 — "
            "폭 비교가 통째로 기울어져 있어 실루엣 수치를 글자폭 근거로 쓸 수 없다",
        )
    stored = report["storedLines"]
    if stored["status"] != "비해당":
        total = stored["storedMultilineParagraphs"]
        gap = stored["rhwpReproducesStoredCut"] - stored["oracleReproducesStoredCut"]
        only = stored["oracleKeepsButRhwpMisses"]
        # 몇 문단 차이는 잡음이다. 의미 있는 격차일 때만 축으로 올린다.
        if gap >= max(5, round(total * 0.05)):
            return (
                "저장줄_재현",
                f"rhwp 가 저장 끊음을 {gap}문단 더 지킨다 — 그 문단은 측정 경로를 "
                "타지 않으므로 폭을 고쳐도 줄이 바뀌지 않는다",
            )
        # **총합이 아니라 겹침으로 판정한다.** 둘이 서로 다른 문단을 놓쳐도 총합은
        # 같아질 수 있고, 그러면 "rhwp 가 낫다/못하다" 가 방향을 잃는다. 착수 대상은
        # 정본이 지킨 컷을 rhwp 만 놓친 문단이다.
        if only >= max(5, round(total * 0.05)):
            return (
                "rhwp_저장줄_이탈",
                f"정본이 지킨 저장 끊음을 rhwp 만 {only}문단 놓친다 — "
                "줄 차이의 책임이 rhwp 쪽에 있다",
            )
    faces = report["declaredFaces"]
    if faces["status"] == "대체있음":
        return (
            "비교가능_주의",
            f"쓸 수 있으나 선언 face {faces['referenced']}개 중 {faces['substituted']}개를 "
            "정본이 대체했다 — 조사 중인 슬롯이 그 안에 있으면 폭 차이를 rhwp 결함으로 "
            "읽기 전에 대체 글꼴부터 확인한다",
        )
    return "비교가능", "확인한 축에서 기준을 그대로 쓸 수 있다"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="기준 PDF 가 그 문서와 비교 가능한지 판정한다"
    )
    parser.add_argument("document", type=Path, help="원본 HWP/HWPX/HML")
    parser.add_argument("pdf", type=Path, help="대응 한컴 정본 PDF")
    parser.add_argument(
        "--page", type=int, action="append", help="1-based 쪽 번호 (여러 번 가능)"
    )
    parser.add_argument(
        "--rhwp-bin",
        type=Path,
        default=ROOT / "target" / "pr-review" / "release" / "rhwp",
        help="rhwp 실행 파일",
    )
    parser.add_argument("--json", action="store_true", help="봉투 JSON 만 출력")
    args = parser.parse_args()

    report = {
        "document": str(args.document),
        "pdf": str(args.pdf),
        "pages": args.page or "전체",
    }
    work = Path(tempfile.mkdtemp(prefix="oracle-comparability-"))
    try:
        source = read_source(args.rhwp_bin, args.document, work)
        oracle = read_pdf(args.pdf, args.page)
        mine = read_rhwp(args.rhwp_bin, args.document, work, args.page)
        report["paperBox"] = judge_paper(oracle["boxes"], mine["boxes"])
        report["declaredFaces"] = judge_faces(
            source["referenced_faces"], oracle["embedded"]
        )
        report["storedLines"] = judge_lines(
            source["paragraphs"], mine["lines"], oracle["lines"]
        )
        report["fontScale"] = judge_font_scale(mine["lines"], oracle["lines"])
        report["contentScale"] = judge_content_scale(
            mine["lines"],
            oracle["lines"],
            report["fontScale"].get("medianOracleOverRhwp"),
        )
        name, why = verdict(report)
        report["verdict"] = name
        report["verdictReason"] = why
    except Unavailable as exc:
        report["verdict"] = "미측정"
        report["verdictReason"] = str(exc)
    finally:
        shutil.rmtree(work, ignore_errors=True)

    if args.json:
        print(json.dumps(report, ensure_ascii=False, sort_keys=True))
        return 0
    print(f"판정: {report['verdict']} — {report['verdictReason']}")
    for axis in ("paperBox", "declaredFaces", "storedLines", "fontScale", "contentScale"):
        if axis in report:
            print(f"  {axis}: {json.dumps(report[axis], ensure_ascii=False)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
