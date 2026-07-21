# -*- coding: utf-8 -*-
"""#2430 폰트 ASCII 메트릭 실측 — 한글 COM 통제 문서 → PDF per-char advance.

각 (폰트, ASCII 글자) 케이스 = 같은 글자 N연속 1문단(SIZE_PT, 자간0, 장평100).
PDF rawdict 의 같은 글자 연속 origin 간격 median = 무신축 advance(em).
공백은 'a␣'×N 교대 패턴에서 (a→a 간격) − w(a) 로 도출.

폰트별 개별 PDF·TSV 를 생성한다(--per-face, 기본). 통합 PDF 는 subset 폰트명이
T1/Haansoft 등으로 병합돼 요청 face 식별이 불가하므로, gen_metrics 가 요구하는
`ladder_<face>.tsv`(요청 face 이름 기준) 규약을 per-face 로 만족시킨다.

사용(권장: 한/글 COM 안정성을 위해 face 당 프로세스 1개):
  for f in 한양신명조 한양중고딕 한양견명조 한양견고딕 휴먼명조; do
      python tools/task2430/hy_ascii_ladder.py --fonts "$f" --out-dir output/poc/task2430
  done
  # → output/poc/task2430/ladder_<face>.tsv (열: face, code, char, adv_em)
  # 이후: python tools/task2430/gen_metrics.py --ladder-dir output/poc/task2430
  # 재검증(비 Windows 포함): gen_metrics.py --ladder-dir ... --verify

preflight: 측정 전 각 요청 face 가 실제 HFT 로 선택 가능한지 CharShape 왕복으로
검증한다(실존 HFT 는 FontType=2 유지, 미설치 face 는 fallback 으로 FontType 변질).
하나라도 실패하면 TSV 를 만들지 않고 non-zero 로 중단한다 — 존재하지 않는
이름을 지정하는 것이 곧 negative-control 이다. 측정 후에는 PDF 임베드 폰트
테이블에서 시스템 폰트 fallback(Haansoft*)이 섞이지 않았는지 재확인한다.
결과는 out-dir/preflight_report.tsv 에 requested_face 기준으로 누적 병합된다
(per-face 프로세스 분할 실행에서도 5종 identity 행이 모두 보존된다).

Windows + 한컴(pyhwpx) 전제. 경로는 리포지토리 루트 기준(절대 경로 하드코딩 없음).
"""
import argparse
import os
import statistics
import sys
import time

sys.stdout.reconfigure(encoding="utf-8")

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
DEFAULT_OUT = os.path.join("output", "poc", "task2430")
CHARS = [chr(c) for c in range(0x21, 0x7F)]  # 0x21..0x7E (94자), 공백은 별도 패턴
# 직선 따옴표는 한/글 편집기 자동 치환(스마트 따옴표) 대상이라 삽입 경로별로
# 측정 여부가 갈린다(통제 불능). 커밋 테이블과 동일하게 측정 제외·보간 처리한다.
# 일괄 삽입 시 실측치(2026-07-21, 한글 2022): 보간 대비 최대 +64% (' 신명조
# 241→395 등) — 교정은 10k 게이트 필요, 별도 후속 트랙.
EXCLUDE_AUTOCORRECT = {0x22, 0x27}
SIZE_PT = 14
N = 12
HFT_FONT_TYPE = 2  # 한/글 CharShape FontType: 2=HFT(한컴 전용), TTF fallback 시 변질
LANGS = ("Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User")


def _new_hwp():
    """항상 새 한/글 프로세스에 붙는다(종료 중인 인스턴스 재부착 방지).

    직전 인스턴스 teardown 중이면 CoCreateInstance 가 일시 실패하므로 백오프 재시도.
    """
    from pyhwpx import Hwp

    last = None
    for _ in range(4):
        try:
            return Hwp(new=True, visible=False)
        except Exception as e:  # noqa: BLE001
            last = e
            time.sleep(4)
    raise last


def _quit_hwp(hwp):
    try:
        hwp.quit()
    except Exception:  # noqa: BLE001
        pass


def _set_face(hwp, face):
    cs = hwp.hwp.HParameterSet.HCharShape
    hwp.hwp.HAction.GetDefault("CharShape", cs.HSet)
    for lang in LANGS:
        setattr(cs, f"FaceName{lang}", face)
        setattr(cs, f"FontType{lang}", HFT_FONT_TYPE)
        setattr(cs, f"Ratio{lang}", 100)
        setattr(cs, f"Spacing{lang}", 0)
    cs.Height = hwp.hwp.PointToHwpUnit(SIZE_PT)
    hwp.hwp.HAction.Execute("CharShape", cs.HSet)


def preflight(hwp, fonts, report_path):
    """각 face 를 CharShape 왕복으로 검증한다. 실패 목록을 돌려준다.

    실존 HFT: 설정한 FaceName/FontType=2 가 그대로 읽힌다.
    미설치 face: 한/글이 fallback 폰트로 해소하며 FontType 이 2 가 아니게 된다.
    """
    rows, failures = [], []
    for face in fonts:
        hwp.hwp.HAction.Run("FileNew")
        _set_face(hwp, face)
        cs = hwp.hwp.HParameterSet.HCharShape
        hwp.hwp.HAction.GetDefault("CharShape", cs.HSet)
        rb_name, rb_type = cs.FaceNameHangul, cs.FontTypeHangul
        ok = rb_name == face and rb_type == HFT_FONT_TYPE
        rows.append((face, rb_name, rb_type, "OK" if ok else "MISSING"))
        if not ok:
            failures.append(face)
        print(f"  [preflight] {face}: readback=({rb_name!r}, FontType={rb_type}) "
              f"{'OK' if ok else '** HFT 미확인 **'}")
    _merge_preflight_report(report_path, rows)
    return failures


HEADER = "requested_face\treadback_face\treadback_fonttype\tverdict\n"


def _merge_preflight_report(report_path, rows):
    """preflight_report.tsv 를 requested_face 기준으로 누적 병합한다.

    per-face 프로세스 분할 실행(권장 경로)에서 매 실행이 파일을 덮어쓰면 마지막
    face 만 남아 5종 identity 증거가 소실된다. 기존 행을 읽어 이번 실행 결과로
    같은 face 를 갱신하고, 나머지는 보존한 뒤 requested_face 순으로 재기록한다.
    """
    merged = {}
    if os.path.exists(report_path):
        with open(report_path, encoding="utf-8") as fh:
            for i, line in enumerate(fh):
                if i == 0 and line.startswith("requested_face"):
                    continue
                cols = line.rstrip("\n").split("\t")
                if len(cols) == 4:
                    merged[cols[0]] = tuple(cols)
    for r in rows:
        merged[r[0]] = tuple(str(x) for x in r)
    with open(report_path, "w", encoding="utf-8") as fh:
        fh.write(HEADER)
        for face in sorted(merged):
            fh.write("\t".join(merged[face]) + "\n")


def check_pdf_fonts(pdf_path, face):
    """PDF 임베드 폰트 테이블에서 시스템 fallback 혼입을 검출한다.

    실존 HFT 는 Type3(식별자 병합) 또는 subset 명이 요청 face(cp949 mojibake
    포함)로 나타난다. 미설치 face 는 'Haansoft *'(함초롬 계열 TTF)로 해소되므로
    해당 엔트리가 보이면 fallback 으로 판정한다.
    """
    import fitz

    d = fitz.open(pdf_path)
    entries = set()
    for pno in range(d.page_count):
        for f in d.get_page_fonts(pno):
            name, ftype = f[3], f[2]
            base = name.split("+", 1)[-1]
            try:  # 한/글 PDF 는 face 명을 cp949 bytes 그대로 실음 → 복원 시도
                decoded = base.encode("latin-1").decode("cp949")
            except (UnicodeEncodeError, UnicodeDecodeError):
                decoded = base
            entries.add((decoded, ftype))
    bad = [e for e in entries if "haansoft" in e[0].lower() or "함초롬" in e[0]]
    return sorted(entries), bad


def gen_pdf(hwp, font, pdf_path):
    """한 face 에 대해 ASCII+공백 통제 문단을 담은 PDF 를 생성한다.

    문단별 COM 호출(수백 회)은 한/글 COM 을 간헐 크래시시키므로, face 1회
    설정 후 \r\n(문단 경계) 연결 일괄 삽입으로 호출 수를 최소화한다.
    """
    hwp.hwp.HAction.Run("FileNew")
    _set_face(hwp, font)
    lines = [("a " * N + "a") if ch == " " else ch * N for ch in CHARS + [" "]]
    hwp.insert_text("\r\n".join(lines) + "\r\n")
    hwp.save_as(pdf_path, format="PDF")


def measure_pdf(pdf_path):
    """PDF 에서 문자별 무신축 advance(em) 를 측정한다 → {char: adv_em}."""
    import fitz

    d = fitz.open(pdf_path)
    by_char = {}  # char -> [adv_px,...]
    space_pat = []  # a-a dx 목록(공백폭 도출용)
    for pno in range(d.page_count):
        for blk in d[pno].get_text("rawdict")["blocks"]:
            if blk["type"] != 0:
                continue
            for line in blk["lines"]:
                chars = [c for span in line["spans"] for c in span["chars"]]
                text = "".join(c["c"] for c in chars)
                stripped = text.strip()
                if not stripped:
                    continue
                uniq = set(stripped)
                if uniq <= {"a", " "} and " " in text and "a" in uniq:
                    xs = [c["origin"][0] for c in chars if c["c"] == "a"]
                    if len(xs) >= 4:
                        dxs = [xs[i + 1] - xs[i] for i in range(1, len(xs) - 2)]
                        space_pat.append(statistics.median(dxs))
                elif len(uniq) == 1 and len(stripped) >= 6:
                    ch = stripped[0]
                    xs = [c["origin"][0] for c in chars if c["c"] == ch]
                    if len(xs) >= 4:
                        dxs = [xs[i + 1] - xs[i] for i in range(1, len(xs) - 2)]
                        by_char.setdefault(ch, []).append(statistics.median(dxs))
    out = {ch: statistics.median(v) / SIZE_PT for ch, v in by_char.items()}
    if space_pat and "a" in by_char:
        out[" "] = (statistics.median(space_pat) - statistics.median(by_char["a"])) / SIZE_PT
    return out


def write_ladder(face, adv, tsv_path):
    with open(tsv_path, "w", encoding="utf-8") as fh:
        fh.write("face\tcode\tchar\tadv_em\n")
        for ch, em in sorted(adv.items(), key=lambda kv: ord(kv[0])):
            if ord(ch) in EXCLUDE_AUTOCORRECT:
                continue
            fh.write(f"{face}\t{ord(ch)}\t{ch!r}\t{em:.4f}\n")


def main():
    ap = argparse.ArgumentParser(description="폰트 ASCII advance 실측 → ladder_<face>.tsv")
    ap.add_argument(
        "--fonts",
        default="한양신명조,한양중고딕,한양견명조,한양견고딕,휴먼명조",
        help="쉼표 구분 face 목록(한글 표시명)",
    )
    ap.add_argument("--out-dir", default=DEFAULT_OUT, help="산출 디렉터리(리포 루트 기준 상대 허용)")
    ap.add_argument("--pdf-only", action="store_true", help="기존 PDF 재측정(COM 생성 생략)")
    ap.add_argument(
        "--skip-preflight", action="store_true",
        help="HFT 실존 preflight 생략(기존 PDF 재측정 등 비 COM 경로 전용)",
    )
    a = ap.parse_args()

    out_dir = a.out_dir if os.path.isabs(a.out_dir) else os.path.join(REPO_ROOT, a.out_dir)
    os.makedirs(out_dir, exist_ok=True)
    fonts = [f.strip() for f in a.fonts.split(",") if f.strip()]

    # 한/글 COM 은 한 프로세스에서 인스턴스를 재생성하면 클라이언트 상태가
    # 오염되어 이후 호출이 전부 실패한다. 프로세스당 인스턴스 1개를 공유하고,
    # 대량 실행 시 face 별로 프로세스를 나눠 호출하는 것을 권장한다(문서 참조).
    hwp = _new_hwp() if not a.pdf_only else None
    try:
        if hwp is not None and not a.skip_preflight:
            failures = preflight(hwp, fonts, os.path.join(out_dir, "preflight_report.tsv"))
            if failures:
                print(f"[abort] HFT 미확인 face {len(failures)}종: {', '.join(failures)} — "
                      "TSV 를 생성하지 않는다 (fallback 결과 오기록 방지)")
                sys.exit(2)

        fallback_hit = []
        for face in fonts:
            safe = face.replace(" ", "_")
            pdf = os.path.join(out_dir, f"ladder_{safe}.pdf")
            tsv = os.path.join(out_dir, f"ladder_{safe}.tsv")
            if hwp is not None:
                gen_pdf(hwp, face, pdf)
            if not os.path.exists(pdf):
                print(f"  [skip] {face}: PDF 없음 {pdf}")
                continue
            entries, bad = check_pdf_fonts(pdf, face)
            if bad:
                print(f"  [fallback] {face}: PDF 에 시스템 대체 폰트 혼입 {bad} — TSV 미생성")
                fallback_hit.append(face)
                continue
            adv = measure_pdf(pdf)
            write_ladder(face, adv, tsv)
            digs = [adv[c] for c in adv if c.isdigit()]
            dstr = f"digit_em={statistics.mean(digs):.3f}" if digs else "digit 미측정"
            print(f"  {face}: {len(adv)}자 {dstr} pdf_fonts={entries} → {os.path.relpath(tsv, REPO_ROOT)}")
    finally:
        if hwp is not None:
            _quit_hwp(hwp)
    if fallback_hit:
        sys.exit(3)


if __name__ == "__main__":
    main()
