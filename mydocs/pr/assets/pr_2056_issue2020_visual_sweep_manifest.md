# Issue 2020 visual sweep evidence

Generated on 2026-07-08 before opening the #2020 close PR.

The evidence below preserves the representative visual sweep outputs that were
originally generated under `output/issue2020/pr_prep/`. The committed PNG files
are the stable evidence assets; the committed JSON files are the raw sweep
summaries copied from the same run.

## Close 기준

- This sweep is used as a structure/layout gate, not as a pixel-perfect font
  fidelity gate.
- `flagged_page_count=0` means the automatic structural visual flags were clear
  for the requested representative pages.
- Low `visual_accuracy_proxy_percent` values are retained as evidence, but are
  not treated as #2020 close blockers when the remaining difference is explained
  by Hancom font absence, embedded PDF font differences, or PUA fallback glyphs.

## Results

| Target | Pages | PDF pages | Flags | Pixel match | Visual proxy | Review asset | Overlay asset | Raw summary |
|--------|-------|-----------|-------|-------------|--------------|--------------|---------------|-------------|
| `issue2020-passport-lawgo-p1` | 1 | 2/2 | 0 | 80.50113 | 42.06503 | `pr_2056_issue2020_passport_lawgo_p1_review.png` | `pr_2056_issue2020_passport_lawgo_p1_overlay.png` | `pr_2056_issue2020_passport_lawgo_p1_summary.json` |
| `issue2020-passport-2022-p1` | 1 | 2/2 | 0 | 80.42947 | 42.24337 | `pr_2056_issue2020_passport_hwp2022_p1_review.png` | `pr_2056_issue2020_passport_hwp2022_p1_overlay.png` | `pr_2056_issue2020_passport_hwp2022_p1_summary.json` |
| `issue2020-fsc-hwp-p1-p2` | 1-2 | 5/5 | 0 | 86.90888 | 18.45890 | `pr_2056_issue2020_fsc_hwp_p1_p2_review.png` | `pr_2056_issue2020_fsc_hwp_p1_p2_overlay.png` | `pr_2056_issue2020_fsc_hwp_p1_p2_summary.json` |
| `issue2020-fsc-hwpx-p1-p2` | 1-2 | 5/5 | 0 | 86.83262 | 18.25356 | `pr_2056_issue2020_fsc_hwpx_p1_p2_review.png` | `pr_2056_issue2020_fsc_hwpx_p1_p2_overlay.png` | `pr_2056_issue2020_fsc_hwpx_p1_p2_summary.json` |
| `issue2020-bokhak-p1` | 1 | 1/1 | 0 | 86.82875 | 40.24777 | `pr_2056_issue2020_bokhak_p1_review.png` | `pr_2056_issue2020_bokhak_p1_overlay.png` | `pr_2056_issue2020_bokhak_p1_summary.json` |
| `issue2020-niklp-p3` | 3 | 35/35 | 0 | 90.20470 | 35.29146 | `pr_2056_issue2020_niklp_p3_review.png` | `pr_2056_issue2020_niklp_p3_overlay.png` | `pr_2056_issue2020_niklp_p3_summary.json` |

## Source pairs

| Target | Source document | Reference PDF |
|--------|-----------------|---------------|
| `issue2020-passport-lawgo-p1` | `samples/issue2020/passport_application_lawgo.hwp` | `pdf/issue2020/passport_application_lawgo-lawgo-2020.pdf` |
| `issue2020-passport-2022-p1` | `samples/issue2020/passport_application_lawgo.hwp` | `pdf/issue2020/passport_application_lawgo-2022.pdf` |
| `issue2020-fsc-hwp-p1-p2` | `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp` | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf` |
| `issue2020-fsc-hwpx-p1-p2` | `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwpx` | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwpx-2022.pdf` |
| `issue2020-bokhak-p1` | `samples/복학원서.hwp` | `pdf/issue2020/복학원서-2022.pdf` |
| `issue2020-niklp-p3` | `samples/2022년 국립국어원 업무계획.hwp` | `pdf/2022년 국립국어원 업무계획-2022.pdf` |

## Commands

```bash
python3 scripts/task1274_visual_sweep.py --out output/issue2020/pr_prep/visual_passport_lawgo_p1 --rhwp-bin target/release/rhwp --key issue2020-passport-lawgo-p1 --hwp samples/issue2020/passport_application_lawgo.hwp --pdf pdf/issue2020/passport_application_lawgo-lawgo-2020.pdf --page 1
python3 scripts/task1274_visual_sweep.py --out output/issue2020/pr_prep/visual_passport_2022_p1 --rhwp-bin target/release/rhwp --key issue2020-passport-2022-p1 --hwp samples/issue2020/passport_application_lawgo.hwp --pdf pdf/issue2020/passport_application_lawgo-2022.pdf --page 1
python3 scripts/task1274_visual_sweep.py --out output/issue2020/pr_prep/visual_fsc_hwp_p1_p2 --rhwp-bin target/release/rhwp --key issue2020-fsc-hwp-p1-p2 --hwp 'samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp' --pdf 'pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf' --pages 1-2
python3 scripts/task1274_visual_sweep.py --out output/issue2020/pr_prep/visual_fsc_hwpx_p1_p2 --rhwp-bin target/release/rhwp --key issue2020-fsc-hwpx-p1-p2 --hwp 'samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwpx' --pdf 'pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwpx-2022.pdf' --pages 1-2
python3 scripts/task1274_visual_sweep.py --out output/issue2020/pr_prep/visual_bokhak_p1 --rhwp-bin target/release/rhwp --key issue2020-bokhak-p1 --hwp samples/복학원서.hwp --pdf pdf/issue2020/복학원서-2022.pdf --page 1
python3 scripts/task1274_visual_sweep.py --out output/issue2020/pr_prep/visual_niklp_p3 --rhwp-bin target/release/rhwp --key issue2020-niklp-p3 --hwp 'samples/2022년 국립국어원 업무계획.hwp' --pdf 'pdf/2022년 국립국어원 업무계획-2022.pdf' --page 3
```
