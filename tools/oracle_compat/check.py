#!/usr/bin/env python3
"""기준 PDF 가 그 문서의 대조군으로 쓸 수 있는지 **착수 전에** 판정한다 (#7396).

한 세션에서 같은 오진을 세 번 했다 — 한글 글꼴이 안 든 cairo 산출본으로 가로 폭을 재고
(#7363, 무효로 닫음), 글꼴 환경이 다른 두 본의 쪽수 차를 엔진 차로 읽고(#7394),
이미 나온 결론을 중복 제기했다(#7009). 셋 다 **"이 PDF 를 이 문서에 써도 되는가"** 를
착수 전에 묻지 못해 생겼다. 이 스크립트가 그 질문에 답한다.

사용법:

    python tools/oracle_compat/check.py <문서.hwp|hwpx> [기준.pdf ...]
    python tools/oracle_compat/check.py <문서> --auto      # pdf/ 에서 이름으로 찾는다

`pypdf` 와 `pypdfium2` 가 필요하다. `--rhwp` 로 바이너리를 주면 문서가 선언한 글꼴을
`rhwp info --json` 으로 읽는다(HWP5·HWPX 공통). 안 주면 HWPX 는 `header.xml` 을 직접
읽고 HWP5 는 선언 글꼴 검사를 건너뛴다.

## 무엇을 재나

1. **서명** — Producer·Creator·쪽수. `cairo` 는 PDF writer 이지 조판 엔진이 아니므로
   그 자체로는 배제 사유가 아니다(#7394). 서명만으로 판정하지 않는다.
2. **한글 글꼴 내장 여부** — 한글을 그릴 수 있는 글꼴이 하나도 없으면 가로 폭·줄 나눔을
   그 PDF 로 판정할 수 없다(#7352 의 49건 부류). 세로·쪽 대응에는 여전히 쓸 수 있다.
3. **한글 전진폭** — 한글→한글 이웃 원점 차의 중앙값(pt). 후보가 둘 이상이면 이 값으로
   환경이 갈린다. 편람 실측에서 세 환경이 정확히 갈렸다 — KoPub 설치본 셋(383쪽),
   기본 치환본 넷(384쪽), 더 좁은 제3의 치환(`-2024.pdf`, 383쪽). **이것이 판정의 주축**이다.
4. **선언 글꼴 ↔ 내장 글꼴** — 보조 정보로만 쓴다. 한글 이름(`KoPub바탕체 Light`)과
   내장 이름(`KoPubBatangLight`)이 달라 대응이 깨지기 쉬워 판정 경로에 넣지 않는다.

판정은 `비교가능 / 가로판정불가 / 환경상이 / 단독후보` 로 나오며, 각 판정에 **그 PDF 로
무엇까지 말할 수 있는지**를 함께 적는다. 후보는 여럿을 함께 넣어야 환경 비교가 된다.
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import statistics
import subprocess
import sys
import zipfile

HANGUL_FONT_HINTS = (
    "batang", "gulim", "dotum", "gungsuh", "malgun", "haansoft", "hcr", "hy",
    "kopub", "nanum", "pretendard", "spoqa", "noto", "함초롬", "휴먼", "한컴",
    "바탕", "굴림", "돋움", "명조", "고딕", "한양",
)

#: 이 폭을 넘게 갈리면 두 본이 서로 다른 글꼴 환경이라고 본다(pt).
#: 편람 실측 9.456 vs 9.949 vs 10.079 에서 가장 가까운 두 본의 차가 0.13 이므로
#: 그보다 살짝 크게 잡아 같은 환경의 잡음(≤0.02)과 가른다.
ENVIRONMENT_SPLIT_PT = 0.15


def is_hangul(ch: str) -> bool:
    return "가" <= ch <= "힣"


def declared_fonts(doc: str, rhwp: str | None) -> tuple[set[str], str]:
    """문서가 선언한 글꼴 face 집합과 그 출처."""
    if rhwp:
        # Windows `CreateProcess` 는 `/` 가 섞인 상대 경로를 못 찾는다 — 절대 경로로 편다.
        binary = os.path.abspath(os.path.normpath(rhwp))
        try:
            out = subprocess.run(
                [binary, "info", os.path.abspath(doc), "--json"],
                capture_output=True, timeout=3600,
            ).stdout.decode("utf-8", "replace")
            fonts = json.loads(out).get("fonts") or []
            if fonts:
                return set(fonts), "rhwp info"
            print("  (rhwp info 가 글꼴을 안 냈다 — header 직독으로 넘어간다)")
        except Exception as exc:  # noqa: BLE001
            print("  (rhwp info 실패: %s — header 직독으로 넘어간다)" % type(exc).__name__)
    if doc.lower().endswith(".hwpx"):
        try:
            with zipfile.ZipFile(doc) as archive:
                head = archive.read("Contents/header.xml").decode("utf-8", "replace")
            return set(re.findall(r'<hh:font\b[^>]*face="([^"]+)"', head)), "header.xml"
        except Exception:
            pass
    return set(), "(못 읽음)"


def pdf_fonts(path: str, sample: int = 25) -> tuple[set[str], int, str, str]:
    import pypdf

    reader = pypdf.PdfReader(path)
    meta = reader.metadata or {}
    total = len(reader.pages)
    step = max(1, total // sample)
    faces: set[str] = set()
    for i in range(0, total, step):
        res = reader.pages[i].get("/Resources")
        if not res:
            continue
        fonts = res.get("/Font")
        if not fonts:
            continue
        for _, ref in fonts.get_object().items():
            base = str(ref.get_object().get("/BaseFont", ""))
            if base:
                faces.add(base.split("+")[-1])
    return faces, total, str(meta.get("/Producer") or ""), str(meta.get("/Creator") or "")


def hangul_advance_pt(path: str, sample: int = 20) -> tuple[float, int]:
    """한글→한글 이웃 원점 차의 중앙값(pt)과 표본 수.

    잉크 상자가 아니라 **원점** 차다 — 글리프 잉크는 글꼴마다 여백이 달라 전진폭을
    말해 주지 않는다.
    """
    import pypdfium2 as pdfium

    doc = pdfium.PdfDocument(path)
    total = len(doc)
    step = max(1, total // sample)
    deltas: list[float] = []
    for pno in range(0, total, step):
        page = doc[pno].get_textpage()
        count = page.count_chars()
        chars: list[str] = []
        boxes: list[object] = []
        for i in range(count):
            ch = page.get_text_range(i, 1)
            chars.append(ch)
            boxes.append(None if ch in ("\r", "\n") else page.get_charbox(i, loose=False))
        for i in range(count - 1):
            a, b = boxes[i], boxes[i + 1]
            if not a or not b or abs(a[3] - b[3]) > 2:
                continue
            delta = b[0] - a[0]
            if 0 < delta < 30 and is_hangul(chars[i]) and is_hangul(chars[i + 1]):
                deltas.append(delta)
    if not deltas:
        return 0.0, 0
    return statistics.median(deltas), len(deltas)


def hangul_capable(faces: set[str]) -> set[str]:
    out: set[str] = set()
    for face in faces:
        low = face.lower()
        if any(hint in low or hint in face for hint in HANGUL_FONT_HINTS):
            out.add(face)
    return out


def norm(name: str) -> str:
    return re.sub(r"[\s_()-]+", "", name).lower()


#: 선언 face 의 한글 이름과 내장 face 의 라틴 이름을 잇는 키.
#: `KoPub바탕체 Light` 는 PDF 에 `KoPubBatangLight` 로 들어간다 — 이름이 그대로 겹치지
#: 않으므로 부분 문자열 대조만으로는 못 잡는다.
FAMILY_KEYS = (
    ("바탕", "batang"), ("굴림", "gulim"), ("돋움", "dotum"), ("궁서", "gungsuh"),
    ("맑은", "malgun"), ("함초롬", "hcr"), ("헤드라인", "hdr"), ("한컴", "haansoft"),
)


def face_keys(name: str) -> set[str]:
    """대조에 쓸 키 — 라틴 토큰과 한글 이름에서 끌어낸 계열 키."""
    flat = norm(name)
    keys = {tok for tok in re.findall(r"[a-z]{3,}", flat)}
    for ko, la in FAMILY_KEYS:
        if ko in name:
            keys.add(la)
    return keys


def matched_faces(declared: set[str], embedded: set[str]) -> set[str]:
    """선언 face 중 내장본에서 이름이 확인되는 것."""
    flats = [norm(f) for f in embedded]
    out: set[str] = set()
    for face in declared:
        keys = face_keys(face)
        if not keys:
            continue
        if any(any(k in flat for k in keys) for flat in flats):
            out.add(face)
    return out


def label(path: str) -> str:
    """같은 이름의 후보가 여러 폴더에 있으므로 부모 폴더까지 보여 준다."""
    parent = os.path.basename(os.path.dirname(path))
    base = os.path.basename(path)
    return "%s/%s" % (parent, base) if parent and parent != "pdf" else base


def judge(doc: str, pdfs: list[str], rhwp: str | None) -> int:
    declared, source = declared_fonts(doc, rhwp)
    declared_hangul = hangul_capable(declared)
    print("문서      %s" % doc)
    print("선언 글꼴 %d개 (%s) — 한글 face %d개"
          % (len(declared), source, len(declared_hangul)))
    if declared_hangul:
        print("          %s" % ", ".join(sorted(declared_hangul)[:8]))
    print()

    rows: list[dict] = []
    seen: set[str] = set()
    for path in pdfs:
        real = os.path.normcase(os.path.abspath(path))
        if real in seen:
            continue
        seen.add(real)
        try:
            faces, pages, producer, creator = pdf_fonts(path)
            advance, samples = hangul_advance_pt(path)
        except Exception as exc:  # noqa: BLE001
            print("  %-46s 읽기 실패 (%s)" % (os.path.basename(path)[:46], type(exc).__name__))
            continue
        matched = matched_faces(declared_hangul, faces)
        rows.append({
            "path": path, "pages": pages, "producer": producer, "creator": creator,
            "embedded_hangul": hangul_capable(faces), "matched": matched,
            "advance": advance, "samples": samples,
        })

    if not rows:
        return 1

    advances = [r["advance"] for r in rows if r["samples"] >= 200]
    spread = (max(advances) - min(advances)) if len(advances) >= 2 else 0.0
    narrowest = min(advances) if advances else 0.0

    print("%-46s %5s %10s %8s %8s  %s"
          % ("PDF", "쪽", "한글전진", "내장한글", "선언일치", "판정"))
    for row in rows:
        # 판정은 **후보 간 전진폭 비교**가 주축이다. 선언 face 이름 대조는 보조 정보로만
        # 쓴다 — 한글/라틴 이름이 달라 대응이 깨지기 쉽고, 오늘 실측에서 전진폭만으로
        # 세 환경(9.94 / 10.15 / 10.19)이 정확히 갈렸다.
        if not row["embedded_hangul"]:
            verdict = "가로판정불가"
        elif len(advances) < 2:
            verdict = "단독후보"
        elif row["advance"] > narrowest + ENVIRONMENT_SPLIT_PT:
            verdict = "환경상이"
        else:
            verdict = "비교가능"
        row["verdict"] = verdict
        print("%-46s %5d %8.3fpt %8d %8d  %s"
              % (label(row["path"])[:46], row["pages"], row["advance"],
                 len(row["embedded_hangul"]), len(row["matched"]), verdict))

    print()
    for row in rows:
        print("· %s" % label(row["path"]))
        print("    Producer=%s  Creator=%s"
              % (row["producer"][:34] or "—", row["creator"][:24] or "—"))
        if row["verdict"] == "가로판정불가":
            print("    한글 글꼴이 하나도 안 들어 있다 — **가로 폭·줄 나눔을 이 본으로 판정하지 않는다**")
            print("    (#7352 부류). 쪽 대응·세로 위치에는 쓸 수 있다.")
        elif row["verdict"] == "단독후보":
            print("    전진폭을 잴 수 있는 후보가 이 하나뿐이라 환경 비교를 못 했다 —")
            print("    같은 문서의 다른 기준본을 함께 넣어라.")
            print("    한글 글꼴은 들어 있으므로 가로 판정 자체는 막히지 않는다.")
        elif row["verdict"] == "환경상이":
            print("    한글 전진폭이 후보 최소본보다 %.3fpt 넓다 — 치환 환경 출력이다."
                  % (row["advance"] - narrowest))
            print("    이 본과 최소본의 쪽수 차이는 결함이 아니라 환경 차다(#7394·#7009).")
            print("    내장 글꼴 목록만으로는 안 갈린다(제3의 치환). 전진폭이 갈랐다.")
        else:
            print("    선언 글꼴이 내장돼 있고 한글 전진폭도 후보 중 최소다 — 대조군으로 쓸 수 있다.")

    if len(advances) >= 2:
        print()
        print("한글 전진폭 폭 %.3fpt (최소 %.3f · 최대 %.3f) — %.2fpt 를 넘으면 환경이 갈린 것으로 본다."
              % (spread, min(advances), max(advances), ENVIRONMENT_SPLIT_PT))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="기준 PDF 의 대조 가능성을 판정한다 (#7396)")
    parser.add_argument("document")
    parser.add_argument("pdfs", nargs="*")
    parser.add_argument("--auto", action="store_true",
                        help="pdf/ 에서 문서 이름으로 후보를 찾는다")
    parser.add_argument("--rhwp", help="rhwp 실행 파일 (선언 글꼴을 읽는 데 쓴다)")
    args = parser.parse_args()

    pdfs = list(args.pdfs)
    if args.auto or not pdfs:
        stem = os.path.splitext(os.path.basename(args.document))[0]
        root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
        found = [
            p for p in glob.glob(os.path.join(root, "pdf", "**", "*.pdf"), recursive=True)
            if os.path.splitext(os.path.basename(p))[0].startswith(stem)
        ]
        pdfs = sorted(found)
        if not pdfs:
            print("pdf/ 에서 '%s' 로 시작하는 후보를 못 찾았다." % stem)
            return 1
    return judge(args.document, pdfs, args.rhwp)


if __name__ == "__main__":
    sys.exit(main())
