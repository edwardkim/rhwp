---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 21 — issue2007 표의 시각 잔여 결함 재판정

## 대상

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 우선 재판정: p2의 셀 내부 문단 겹침, p4의 표 우측 외곽 세로선 소실

Stage 20은 p10–p17의 RowBreak continuation 물리 page clip을 PDF와 대조해 보정했다.
이 단계에서는 페이지 owner 이동과 별개인 표 내부 paint 결함을, 현재 commit에서 PDF와
직접 다시 대조한 뒤 각각의 render-tree/SVG 경로를 분리해 분석한다.

## 판정 규칙

- `fidelity_compare`의 `table_cell_text_overlap` 및 `svg_table_border_clip`는 후보만
  제공한다. PDF와 동일한지를 PNG로 최종 확인한다.
- p2와 p4가 재현되지 않으면 이미 해결된 것으로 기록하고 추가 보정하지 않는다.
- 재현되면 각 결함의 원인·수정·증적을 이 문서에 추가하고, Stage 종료 전 focused 회귀를
  남긴다.

## 현재 commit 재판정

`dd17e6a1d`에서 입력과 기준 PDF를 p2·p4별로 192 DPI(794×1123 CSS px 대응) PNG 쌍으로
직접 대조했다.

- p2: 두 열의 조문 대비표에서 셀 안 문단이 겹쳐 paint되는 현상은 재현되지 않았다.
  PDF와 같은 행 순서·셀 경계로 배치된다.
- p4: 큰 표의 우측 외곽 세로선이 현재 SVG와 raster에 모두 존재하며, PDF와 같은 표 frame을
  이룬다.
- `fidelity_compare --text-only --layout-ledger`도 p2–p4에서
  `table_cell_text_overlap=0`, `svg_table_border_clip` 후보 0, body↔footer/table frame
  후보 0을 기록했다. 전체 render tree 쪽수는 PDF와 같은 17이다.

## 증적

저장소 PNG가 LFS 대상이 아님을 `git check-attr filter`로 먼저 확인했다.

- [p2 PDF 직접 대조](../pr/assets/task_m100_3820_stage21_issue2007_table_visual_residual/review_p002_after_pair.png)
- [p4 PDF 직접 대조](../pr/assets/task_m100_3820_stage21_issue2007_table_visual_residual/review_p004_after_pair.png)
- [표 내부 text-overlap 원장](../pr/assets/task_m100_3820_stage21_issue2007_table_visual_residual/table-cell-text-overlap-candidates.tsv)
- [표 외곽선 clip 원장](../pr/assets/task_m100_3820_stage21_issue2007_table_visual_residual/svg-table-border-clip-candidates.tsv)
- [layout 후보 원장](../pr/assets/task_m100_3820_stage21_issue2007_table_visual_residual/layout-candidates.tsv)
- [페이지 수 원장](../pr/assets/task_m100_3820_stage21_issue2007_table_visual_residual/page-count-ledger.tsv)

재현 명령:

```bash
RHWP_BIN=target/task-3820-3821-fidelity/release-test/rhwp \
  python3 tools/fidelity_compare/fidelity_compare.py 1 3 \
    --source samples/basic/issue2007_nested_cell_pagination_42065.hwp \
    --reference-pdf pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf \
    --label issue2007-stage21-table-residual \
    --reference-grade 'Hancom 2020 reference PDF' \
    --text-only --layout-ledger --out-dir /private/tmp/rhwp-stage21-fidelity-ledger
```

## 결론

p2 및 p4는 현재 commit에서 추가 코드 보정 없이 정상이다. 따라서 이 두 과거 결함을
현재 잔여 결함으로 이월하지 않는다.
