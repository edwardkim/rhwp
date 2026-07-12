# #1999 Stage 1 - issue1949 기준 PDF 쪽수/시각 정합 분석

## 대상

- 이슈: https://github.com/edwardkim/rhwp/issues/1999
- 샘플: `samples/issue1949_giant_cell_nested_tables_perf.hwpx`
- 기준 PDF: `pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf`

## 초기 상태

- `upstream/devel` 기준 브랜치 `task_m100_1999_issue1949_page_parity`에서 시작했다.
- 기존 #1985 visual sweep 결과는 SVG 112쪽 / 기준 PDF 115쪽, `flagged=100/112`다.
- 대표 mismatch는 `output/task1274/pr1985_issue1949/review/review_103.png`와
  `mydocs/pr/assets/pr_1985_issue1949_review_103.png`에 남아 있다.

## 분석 방향

- #1985는 성능 병목 제거가 목적이었고, 쪽수 정합은 후속으로 분리되어 있다.
- #1999는 성능 캐시를 유지하면서 RowBreak 거대 셀 분할 높이가 기준 PDF보다
  과소 산정되는 원인을 문서 속성 기반으로 찾는다.
- 특정 샘플명, 페이지 번호, 이슈 번호, 임의 계수로 맞추는 보정은 사용하지 않는다.

## 수정 내용

### RowBreak 컷 판정 보정

- RowBreak 거대 셀의 visible-tail over-fill grace가 빈 문단 spacer 뒤에 다시 본문이
  이어지는 경우까지 허용되어, 실제 셀 중간 본문을 이전 쪽으로 과도하게 끌어올렸다.
- `grace_visible_tail_before_spacer`에서 첫 spacer 전후의 `CellUnit` 구성을 함께 보고,
  spacer 뒤에 일반 가시 본문이 이어지면 구조적 꼬리줄이 아닌 문단 사이 여백으로
  판정하도록 좁혔다.
- HWPX RowBreak 셀의 저장 `LineSeg.vertical_pos` 리셋은 모두 쪽 경계로 볼 수 없다.
  현재 조각이 페이지 절반도 채우지 못한 중간 리셋은 같은 쪽 안의 로컬 좌표 재시작으로
  흡수하고, 하단 근처 리셋은 기존처럼 저장 쪽 경계로 보존하도록 했다.

### continuation 그림 반복 렌더 방지

- 사용자가 확인한 한컴 화면 기준으로, 일부 그림은 문서 전체 반복 그림이 아니라 특정
  문단에 붙은 non-inline 그림이다.
- 기존 `cell_cut_contains_non_inline_control_units`는 컷 시작이 해당 문단 뒤로 지난
  경우에도 그 문단의 non-inline 그림을 후속 continuation에서 다시 렌더 후보로 보아,
  문단 310의 그림 2개가 뒤쪽 페이지마다 반복되고 이어지는 내용이 아래로 밀렸다.
- 현재 컷 범위 안에 해당 문단의 실제 `CellUnit`이 포함될 때만 non-inline 그림을
  렌더 후보로 보도록 바꿨다. 이는 파일명/쪽번호가 아니라 셀 컷 범위와 문단-유닛
  대응에 근거한다.

## 검증

- `env CARGO_INCREMENTAL=0 cargo build`
- `target/debug/rhwp info samples/issue1949_giant_cell_nested_tables_perf.hwpx`
  - 결과: `페이지 수: 115`
- `target/debug/rhwp export-render-tree samples/issue1949_giant_cell_nested_tables_perf.hwpx -o output/task1999_rt_probe_after -p 102`
  - 결과: `render_tree_103.json`에서 반복 그림 노드가 사라지고 표/텍스트만 배치됨.
- `env CARGO_INCREMENTAL=0 cargo test row_cut_tests --lib`
  - 결과: 16 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1949_giant_cell_render_perf`
  - 결과: 1 passed, `page_count() == 115`

### visual sweep

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key issue1999-p103-after \
  --hwp samples/issue1949_giant_cell_nested_tables_perf.hwpx \
  --pdf pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf \
  --page 103 \
  --out output/task1999_issue1999_p103_after \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF/render-tree: 115 / 115 / 115쪽
- 선택 페이지: 103쪽
- flagged: 0 / 1
- pixel match: 89.06581%
- visual_accuracy_proxy_percent: 5.34185%
- compare: `output/task1999_issue1999_p103_after/issue1999-p103-after/compare/compare_103.png`
- overlay: `output/task1999_issue1999_p103_after/issue1999-p103-after/overlay/overlay_103.png`
- review: `output/task1999_issue1999_p103_after/issue1999-p103-after/review/review_103.png`

비고:

- 반복 그림으로 인한 p103 대변위는 제거되었다.
- `visual_accuracy_proxy_percent`는 여전히 낮지만, 이는 표 위치/폭/폰트 차이까지 포함한
  내용 픽셀 중심 보조값이다. 이번 단계의 주 목표인 115쪽 parity와 반복 그림 제거는
  확인했다.

### 14~16쪽 추가 시각 검증

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key issue1999-p14-16-after \
  --hwp samples/issue1949_giant_cell_nested_tables_perf.hwpx \
  --pdf pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf \
  --pages 14-16 \
  --out output/task1999_issue1999_p14_16_after \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF/render-tree: 115 / 115 / 115쪽
- 선택 페이지: 14~16쪽
- flagged: 1 / 3
- p14: pixel match 91.64224%, visual_accuracy_proxy_percent 14.64454%, flags 없음
- p15: pixel match 92.88879%, visual_accuracy_proxy_percent 12.65274%,
  `render_tree_frame_tail_overflow`, `line_band_drift`, `column_line_band_drift`,
  `large_ink_region_drift`
- p16: pixel match 90.96575%, visual_accuracy_proxy_percent 11.55383%, flags 없음
- p14 review: `output/task1999_issue1999_p14_16_after/issue1999-p14-16-after/review/review_014.png`
- p15 review: `output/task1999_issue1999_p14_16_after/issue1999-p14-16-after/review/review_015.png`
- p16 review: `output/task1999_issue1999_p14_16_after/issue1999-p14-16-after/review/review_016.png`

판정:

- p15는 아직 기준 PDF와 시각 흐름이 맞지 않는다.
- p15 render tree에서 `3.5.2.1` 이후 문단 일부가 프레임 하단을 넘어가며, 기준 PDF와
  그림/본문 배치가 다르다.
- 115쪽 parity와 p103 반복 그림 제거만으로 #1999 완료로 보기에는 부족하므로, p15의
  non-inline 그림 continuation 배치와 tail overflow 원인을 추가로 좁혀야 한다.
