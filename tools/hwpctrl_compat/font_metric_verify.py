"""임의 폰트의 ASCII advance 를 한글 PDF 와 rhwp 로 견준다 — 폰트별 metric 정확도 검증.

## §4.30~§4.31 의 일반화

`pdf_metric_oracle.py` 는 하드코딩한 함초롬바탕 표하고만 견줬다. 이 도구는 **rhwp 의 실제
출력**(`dump-carets`)과 견주므로 라우팅 경로(전용 표·embedded metric·0.5em 폴백)와 무관하게
어느 폰트든 검증한다.

절차:

1. 한글이 **격리 표본**(`가X가Y…`, 각 ASCII 를 `가` 로 감싸)을 그 폰트로 만들어 `.hwp` 와
   `.pdf` 로 낸다.
2. PDF 글리프 origin 델타 → 한글 advance/em(한국어 전각으로 정규화).
3. 같은 `.hwp` 에 `dump-carets` → rhwp advance/em.
4. 글자별로 견줘 8% 넘는 것을 표시한다.

## 쓰임

    python tools/hwpctrl_compat/font_metric_verify.py 함초롬돋움
    python tools/hwpctrl_compat/font_metric_verify.py "맑은 고딕" --tag malgun
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
EXE = REPO / "target" / "release" / "rhwp.exe"
OUT = REPO / "output" / "poc" / "hwpctrl" / "font_verify"


def build_sample(font: str, hwp: Path, pdf: Path, timeout: int) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    # 한글 폰트명을 `-c` 인자로 넘기면 시스템 코드페이지에서 깨진다 — 빌더를 **UTF-8 파일**로
    # 써서 실행한다. 폰트명·경로는 JSON 으로 실어 이스케이프를 피한다.
    faces = ("FaceNameHangul", "FaceNameLatin", "FaceNameHanja", "FaceNameJapanese",
             "FaceNameOther", "FaceNameSymbol", "FaceNameUser")
    cfg = json.dumps({"font": font, "hwp": str(hwp), "pdf": str(pdf), "faces": list(faces)},
                     ensure_ascii=False)
    code = (
        "import json, sys\n"
        f"cfg = json.loads(r'''{cfg}''')\n"
        "from pyhwpx import Hwp\n"
        "hwp = Hwp(new=True, visible=False); com = hwp.hwp\n"
        # 연속 ASCII 를 쓴다 — `가X가` 격리는 rhwp 의 혼합-스크립트 간격을 타 캐럿 advance 를
        # 오염시킨다. 앞의 `가나다` 는 정규화용 순수 한국어 run 이다.
        "seq = '가나다 ' + ''.join(chr(c) for c in range(0x21, 0x7f))\n"
        "com.HAction.GetDefault('InsertText', com.HParameterSet.HInsertText.HSet)\n"
        "com.HParameterSet.HInsertText.Text = seq\n"
        "com.HAction.Execute('InsertText', com.HParameterSet.HInsertText.HSet)\n"
        "com.Run('SelectAll')\n"
        "cs = com.HParameterSet.HCharShape\n"
        "com.HAction.GetDefault('CharShape', cs.HSet)\n"
        "for a in cfg['faces']:\n"
        "    try: setattr(cs, a, cfg['font'])\n"
        "    except Exception: pass\n"
        "cs.Height = 1000\n"
        "com.HAction.Execute('CharShape', cs.HSet)\n"
        # **왼쪽정렬을 강제한다.** 기본은 양쪽정렬이라 짧은 줄의 글자가 줄 폭에 퍼져 advance 가
        # 뻥튀기된다(가나다 사이가 143px). AlignType 설정은 안 먹어 액션으로 건다.
        "com.Run('SelectAll')\n"
        "com.Run('ParagraphShapeAlignLeft')\n"
        "com.SaveAs(cfg['hwp'], 'HWP', '')\n"
        "ok = com.SaveAs(cfg['pdf'], 'PDF', '')\n"
        "com.Quit()\n"
        "raise SystemExit(0 if ok else 1)\n"
    )
    builder = OUT / "_builder.py"
    builder.write_text(code, encoding="utf-8")
    proc = subprocess.run([sys.executable, "-X", "utf8", str(builder)], timeout=timeout, check=False)
    if proc.returncode != 0 or not pdf.exists() or not hwp.exists():
        raise SystemExit(f"표본 생성 실패: {font}")


def pdf_advances(pdf: Path) -> tuple[dict[str, float], float, set]:
    import fitz

    doc = fitz.open(str(pdf))
    glyphs, size = [], None
    for b in doc[0].get_text("rawdict")["blocks"]:
        for line in b.get("lines", []):
            for sp in line["spans"]:
                size = sp["size"]
                for ch in sp["chars"]:
                    glyphs.append((ch["c"], ch["origin"][0], ch["origin"][1]))
    lines: dict = defaultdict(list)
    for g in glyphs:
        lines[round(g[2])].append(g)
    adv: dict[str, list] = defaultdict(list)
    kor = []
    for gl in lines.values():
        gl.sort(key=lambda g: g[1])
        for i in range(len(gl) - 1):
            c = gl[i][0]
            nxt = gl[i + 1][0]
            a = (gl[i + 1][1] - gl[i][1]) / size
            if "가" <= c <= "힣" and "가" <= nxt <= "힣":
                kor.append(a)  # 순수 한국어 인접 쌍만 (혼합 경계 제외)
            elif 0x21 <= ord(c) < 0x7F and 0x21 <= ord(nxt) < 0x7F:
                adv[c].append(a)  # ASCII↔ASCII 인접만
    scale = sum(kor) / len(kor) if kor else 1.0
    font = {(sp["font"]) for b in doc[0].get_text("rawdict")["blocks"]
            for line in b.get("lines", []) for sp in line["spans"]}
    return {c: sum(v) / len(v) / scale for c, v in adv.items()}, scale, font


def rhwp_advances(hwp: Path) -> dict[str, float]:
    """dump-carets 로 rhwp advance/em. 문단 0 은 `가X가Y…` 라 홀수 offset 이 ASCII 다."""
    out = subprocess.run(
        [str(EXE), "dump-carets", str(hwp), "-p", "0", "--json"], capture_output=True, check=False
    )
    carets = json.loads(out.stdout.decode("utf-8", "replace"))["carets"]
    xs = {c["offset"]: c["x"] for c in carets}
    ys = {c["offset"]: c["y"] for c in carets}
    hs = {c["offset"]: c["height"] for c in carets}
    em = hs.get(0, 13.3)
    seq = "가나다 " + "".join(chr(c) for c in range(0x21, 0x7f))
    # 순수 한국어 인접 쌍(가나다)으로 정규화, ASCII↔ASCII 인접만 advance 로 쓴다 — PDF 쪽과
    # 같은 규칙이라야 배율이 상쇄된다. **줄 경계(y 가 다른 쌍)는 건너뛴다** — 줄 끝 글자는
    # 다음 줄 첫 글자와 음수 델타가 나온다.
    kor, adv = [], {}
    for off in range(len(seq) - 1):
        if off not in xs or off + 1 not in xs:
            continue
        if abs(ys.get(off, 0) - ys.get(off + 1, 0)) > 1:
            continue  # 줄 경계
        c, nxt = seq[off], seq[off + 1]
        a = (xs[off + 1] - xs[off]) / em
        if "가" <= c <= "힣" and "가" <= nxt <= "힣":
            kor.append(a)
        elif 0x21 <= ord(c) < 0x7F and 0x21 <= ord(nxt) < 0x7F:
            adv.setdefault(c, []).append(a)
    scale = sum(kor) / len(kor) if kor else 1.0
    return {c: sum(v) / len(v) / scale for c, v in adv.items()}


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("font")
    ap.add_argument("--tag", help="산출 파일 이름(기본은 폰트명)")
    ap.add_argument("--timeout", type=int, default=150)
    ap.add_argument("--reexport", action="store_true")
    args = ap.parse_args()

    tag = args.tag or args.font.replace(" ", "_")
    hwp = OUT / f"{tag}.hwp"
    pdf = OUT / f"{tag}.pdf"
    if args.reexport or not (hwp.exists() and pdf.exists()):
        build_sample(args.font, hwp, pdf, args.timeout)

    han, scale, pdf_font = pdf_advances(pdf)
    rhwp = rhwp_advances(hwp)
    print(f"폰트 요청: {args.font} · PDF 폰트: {pdf_font} · 장평 {scale:.4f}")
    print(f"{'글자':<5}{'한글/em':>9}{'rhwp/em':>9}{'차%':>7}")
    flags = []
    for cp in range(0x21, 0x7f):
        c = chr(cp)
        if c not in han or c not in rhwp:
            continue
        h, r = han[c], rhwp[c]
        pct = (r - h) / h * 100 if h else 0
        mark = " <<<" if abs(pct) > 8 else ""
        if abs(pct) > 8:
            flags.append((c, round(h, 3), round(r, 3), round(pct, 1)))
        print(f"'{c}'  {h:9.3f}{r:9.3f}{pct:+6.1f}%{mark}")
    print(f"\n8% 초과 {len(flags)}/{len([c for c in han if c in rhwp])}자")
    for c, h, r, p in flags:
        print(f"  '{c}' 한글={h} rhwp={r} ({p:+}%)")
    return 1 if flags else 0


if __name__ == "__main__":
    raise SystemExit(main())
