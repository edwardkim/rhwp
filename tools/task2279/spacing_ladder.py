# -*- coding: utf-8 -*-
"""#2279 자간 통제 사다리 — 한글 COM 으로 (폰트×글자×자간×장평) 조합 문서 생성 → PDF 실측.

각 케이스 = 한 문단(동일 글자 12자 연속, 왼끝 아님이어도 단일 줄=무신축).
판정: adv_em(spacing) − adv_em(0) 의 기울기가
  - 폰트크기 비례(fs-비례): 0.01/% (em 단위)
  - 글자폭 비례: 0.01×base_em×ratio/%
"""
import os
import sys
import time

sys.stdout.reconfigure(encoding="utf-8")

OUT_DIR = r"C:\Users\planet\rhwp\output\poc\task2279_advance"
PDF = os.path.join(OUT_DIR, "spacing_ladder.pdf")
HWPX = os.path.join(OUT_DIR, "spacing_ladder.hwpx")

FONTS = ["휴먼명조", "한컴돋움", "함초롬바탕", "HY중고딕"]
CHARS = ["*", "0", "a", ".", "가"]
SPACINGS = [-20, -9, 0, 10, 20]
RATIOS = [100, 96]
SIZE_PT = 14
N = 12  # 글자 수


def launch(retries=5):
    from pyhwpx import Hwp
    for i in range(retries):
        try:
            return Hwp(visible=False)
        except Exception as e:
            print(f"launch retry {i+1}: {e}")
            time.sleep(8)
    raise RuntimeError("Hwp launch failed")


def main():
    hwp = launch()
    try:
        hwp.hwp.HAction.Run("FileNew")
        cases = []
        for font in FONTS:
            for ratio in RATIOS:
                for sp in SPACINGS:
                    for ch in CHARS:
                        cases.append((font, ratio, sp, ch))
        for idx, (font, ratio, sp, ch) in enumerate(cases):
            # 글자모양 설정 (이후 입력에 적용)
            cs = hwp.hwp.HParameterSet.HCharShape
            hwp.hwp.HAction.GetDefault("CharShape", cs.HSet)
            for lang in ("Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User"):
                setattr(cs, f"FaceName{lang}", font)
                setattr(cs, f"FontType{lang}", 2)  # TTF
                setattr(cs, f"Ratio{lang}", ratio)
                setattr(cs, f"Spacing{lang}", sp)
            cs.Height = hwp.hwp.PointToHwpUnit(SIZE_PT)
            hwp.hwp.HAction.Execute("CharShape", cs.HSet)
            hwp.insert_text(ch * N)
            hwp.hwp.HAction.Run("BreakPara")
        hwp.save_as(HWPX, format="HWPX")
        hwp.save_as(PDF, format="PDF")
        print(f"generated cases={len(cases)} → {PDF}")
    finally:
        hwp.quit()

    # ---- PDF 실측 ----
    import fitz
    import statistics
    d = fitz.open(PDF)
    lines = []  # (page, y, char, [dx...], size)
    for pno in range(d.page_count):
        for blk in d[pno].get_text("rawdict")["blocks"]:
            if blk["type"] != 0:
                continue
            for line in blk["lines"]:
                chars = [c for span in line["spans"] for c in span["chars"]]
                text = "".join(c["c"] for c in chars).strip()
                if len(text) < N - 2 or len(set(text)) != 1:
                    continue
                size = line["spans"][0]["size"]
                xs = [c["origin"][0] for c in chars if c["c"] == text[0]]
                dxs = [xs[i + 1] - xs[i] for i in range(1, len(xs) - 2)]  # 가장자리 제외
                if not dxs:
                    continue
                lines.append((pno, line["bbox"][1], text[0], statistics.median(dxs), size))
    lines.sort(key=lambda t: (t[0], t[1]))
    print(f"pdf measured lines={len(lines)} (expect {len(cases)})")

    # 케이스 순서 = 문서 순서 = PDF 읽기 순서
    print("font\tratio\tsp%\tchar\tadv_em\tbase0_em\td_em\tfs모델\t폭모델")
    base_cache = {}
    rows = []
    for (font, ratio, sp, ch), (pno, y, mch, dx, size) in zip(cases, lines):
        if mch != ch:
            print(f"  [경고] 케이스/측정 불일치: {font} {ratio} {sp} {ch!r} vs pdf {mch!r}")
        adv_em = dx / SIZE_PT
        rows.append((font, ratio, sp, ch, adv_em))
        if sp == 0:
            base_cache[(font, ratio, ch)] = adv_em
    for font, ratio, sp, ch, adv_em in rows:
        base0 = base_cache.get((font, ratio, ch))
        if base0 is None or sp == 0:
            continue
        d_em = adv_em - base0
        fs_pred = sp / 100.0
        w_pred = sp / 100.0 * base0
        print(f"{font}\t{ratio}\t{sp}\t{ch}\t{adv_em:.4f}\t{base0:.4f}\t{d_em:+.4f}\t{fs_pred:+.4f}\t{w_pred:+.4f}")


if __name__ == "__main__":
    main()
