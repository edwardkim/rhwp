---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 23 — issue2007 p13 PDF 물리 owner drift

## 재개 범위

Stage 22는 p9의 완료된 다행 표 뒤 제목이 ancestor clip으로 사라지는 결함을
`8f3a63c47`에서 보정했다. 그 보정은 p9 source 경계에 한정하며, p13의 물리 page
owner를 해결했다고 주장하지 않는다.

이번 단계의 단일 대상은 다음 direct pair다.

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 매핑: PDF p13 ↔ `rhwp export-svg --page 12` (0-based), visual sweep `--page 13`

## 기존 관찰과 조사 계약

Stage 20의 PDF 직접 대조에서는 기준 PDF p13의 선행 표시와 rhwp의 `4`가 달라 보였다.
이는 원래 p12→p13 fragment/text owner가 앞당겨진 문제로 가정했으나, 아래의 현재
직접 대조로 가설을 재판정한다.

이 단계에서는 다음 순서를 고정한다.

1. 현재 commit에서 p12·p13 PDF/SVG raster와 render tree를 다시 생성해 경계가 여전히
   존재하는지 확인한다.
2. 기준 PDF의 마지막 p12 문장과 첫 p13 문장을 source `pi`·`CellUnit`·nested split에
   대응한다. 임의의 global height 또는 font 보정은 금지한다.
3. 실제 source owner가 확인된 뒤에만 좁은 renderer 보정과 focused regression을 추가한다.
4. 수정 후 p12·p13 뿐 아니라 Stage 20의 p11/p17과 Stage 22 p9를 다시 확인한다.

페이지 수 또는 자동 픽셀 점수만으로 완료를 판정하지 않는다. 각 물리 페이지의 PDF
raster 직접 대조와 text owner를 함께 기록한다.

## 재검증 결과 — owner drift 가설 기각

현재 focused renderer binary와 Mac fontconfig(`휴먼명조`/`휴먼고딕`)에서 다음을 수행했다.

```bash
# `fidelity_compare` direct pair의 page range는 0-based, 끝 포함이다.
RHWP_BIN=target/task-3820-3821-fidelity/release-test/rhwp \
python3 tools/fidelity_compare/fidelity_compare.py 0 16 \
  --source samples/basic/issue2007_nested_cell_pagination_42065.hwp \
  --reference-pdf pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf \
  --label issue2007-stage23 --reference-grade 'Hancom 2020 baseline PDF' \
  --text-only --export-all-svg --layout-ledger --out-dir /private/tmp/issue2007-ledger

# `visual_sweep` page option은 viewer와 같은 1-based다.
python3 scripts/visual_sweep.py --hwp samples/basic/issue2007_nested_cell_pagination_42065.hwp \
  --pdf pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf \
  --key issue2007-stage23-full --pages 1-17 \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --dpi 144 \
  --out /private/tmp/issue2007-visual
```

- fidelity 원장: PDF/SVG/render tree는 모두 17쪽, text owner shift·sequence shift·cell-text
  overlap·SVG table-border clip 후보는 0건이다.
- p12→p13 PDF 직접 대조: 중앙선거관리위원회 다음에 국가인권위원회, 이어 감사원이 같은
  물리 순서로 배치된다. render tree의 p13은 `pi=92` 국가인권위원회, 이어진 표 `pi=93`,
  감사원 `pi=95` 순서다.
- 기준 PDF에서 `3`처럼 보였던 선행 문자는 PDF text 추출에서 U+FFFD로 손상된 PUA glyph이며,
  rhwp의 `4`와 텍스트 owner를 판정할 근거가 아니다. 즉 현재 증거로는 p13 owner drift를
  주장할 수 없다. glyph 표시 정합은 별도 paint-time glyph 조사 대상이다.
- p12 render tree에 보이는 후속 nested-table node는 ancestor Cell clip 밖(`y=1003.8`,
  clip bottom 약 `997.7`)이며 PDF/rhwp raster에는 paint되지 않는다. 이를 tail overflow로
  세면 false positive다.

## visual-sweep 보정 범위

144dpi full sweep에서 p5/p12의 `render_tree_frame_tail_overflow`는 실제 frame 밖 잉크가
0인데도, 96dpi render-tree y 좌표를 raster frame과 직접 비교해 발생했다. 따라서
`render_tree_frame_tail_candidates`는 page bbox→raster 좌표 투영 후 해당 bbox에 실제 잉크가
남은 line만 판정하도록 보정한다. 이 보정은 사용자-visible layout을 바꾸지 않고, 고 DPI의
자동 후보를 물리 페이지 기준으로 바로잡는다.

## 최종 검증과 증적

- `python3 -m unittest scripts/tests/test_visual_sweep.py` — 34 passed.
- focused renderer regression
  `CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2007_nested_cell_pagination`
  — 9 passed.
- 수정 후 144dpi 전체 sweep: PDF/SVG/render tree/raster/compare/overlay/review 모두 17/17,
  `flagged_pages.json`은 빈 배열이다. p5·p12의 false `tail`과 p10·p13의 hidden continuation
  node도 모두 0건이 됐다.

현재 native PDF 대조의 주요 증적은 다음 위치에 보존했다.

- `mydocs/pr/assets/task_m100_3820_stage23_issue2007_sweep/review_contact_sheet.png`
- `mydocs/pr/assets/task_m100_3820_stage23_issue2007_sweep/review_p002.png`
- `mydocs/pr/assets/task_m100_3820_stage23_issue2007_sweep/review_p004.png`
- `mydocs/pr/assets/task_m100_3820_stage23_issue2007_sweep/review_p010.png`
- `mydocs/pr/assets/task_m100_3820_stage23_issue2007_sweep/review_p013.png`
- `mydocs/pr/assets/task_m100_3820_stage23_issue2007_sweep/overlay_metrics.json`
- `mydocs/pr/assets/task_m100_3820_stage23_issue2007_sweep/flagged_pages.json`

직접 확인한 현재 native 출력에서는 p2의 셀 내 문단 중첩, p4의 우측 외곽선 소실, p10의
빈 continuation 표는 재현되지 않았다. p13도 국가인권위원회→감사원 순서가 PDF와 같다.
단, p2/p4/p10/p13의 raster 일치율 보조값은 각각 7.49756/19.41271/9.39024/8.86217%로
낮다. 이 값은 한컴 PDF와 native SVG의 글꼴 raster·글자폭 차이를 포함하므로 수용 판정이
아니며, WASM/브라우저 최종 시각 판정은 별도 범위다.
