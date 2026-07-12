# #1999 Stage 2 - HWP/HWPX 115쪽 parity 및 14~16쪽 시각 검증

## 목적

Stage 1에서 HWPX 기준 115쪽 parity와 일부 반복 그림 제거를 확인했지만, 이후 사용자가
`samples/issue1949_giant_cell_nested_tables_perf.hwp`도 기준 PDF와 같은 115쪽이어야 한다고
명시했다. 이 스테이지는 범위를 다음으로 제한한다.

- 기준 PDF 쪽수: `pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf` = 115쪽
- HWPX 샘플 쪽수: `samples/issue1949_giant_cell_nested_tables_perf.hwpx` = 115쪽
- HWP 샘플 쪽수: `samples/issue1949_giant_cell_nested_tables_perf.hwp` = 115쪽
- 사용자가 지적한 14~16쪽을 HWP/HWPX 각각 기준 PDF와 시각 비교한다.

## 방법론 보정

- Stage 1에 계속 누적하지 않고 Stage 2 문서를 새로 만들었다.
- 이 스테이지에서는 코드 추가 분석, 쪽수 확인, visual sweep, 테스트 결과를 순서대로 기록한다.
- 특정 파일명, 페이지 번호, PR/issue 번호, 임의 계수로 맞추는 분기는 사용하지 않는다.
- 보정 근거는 문서에서 읽은 표/셀/문단/control 배치 속성과 레이아웃 단위 처리에 둔다.

## 현재 확인할 사항

1. 최종 코드 상태에서 PDF/HWP/HWPX 쪽수가 모두 115쪽인지 확인한다.
2. HWP 14~16쪽 visual sweep 결과가 `flagged=0/3`인지 확인한다.
3. HWPX 14~16쪽 visual sweep 결과가 `flagged=0/3`인지 확인한다.
4. `tests/issue_1949_giant_cell_render_perf.rs`가 HWPX뿐 아니라 HWP 115쪽 조건도 검증하는지 확인한다.
5. 코드 수정 후 focused test와 관련 visual sweep 결과를 이 문서에 기록한다.

## 진행 기록

### Stage 2 시작 상태

- 작업 브랜치: `task_m100_1999_issue1949_page_parity`
- 변경 파일:
  - `src/renderer/layout/table_layout.rs`
  - `src/renderer/layout/table_partial.rs`
  - `tests/issue_1949_giant_cell_render_perf.rs`
  - `samples/issue1949_giant_cell_nested_tables_perf.hwp`
- 장기 실행 중인 cargo/visual sweep 프로세스는 없는 상태에서 재확인한다.

### 쪽수 확인

명령:

```bash
pdfinfo pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf | rg '^Pages:'
target/debug/rhwp info samples/issue1949_giant_cell_nested_tables_perf.hwpx | rg '페이지 수'
target/debug/rhwp info samples/issue1949_giant_cell_nested_tables_perf.hwp | rg '페이지 수'
```

결과:

- 기준 PDF: 115쪽
- HWPX: `페이지 수: 115`
- HWP: `페이지 수: 115`

### HWP 14~16쪽 visual sweep

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key issue1999-p14-16-hwp-final \
  --hwp samples/issue1949_giant_cell_nested_tables_perf.hwp \
  --pdf pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf \
  --pages 14-16 \
  --out output/task1999_issue1999_p14_16_hwp_final \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF/render-tree: 115 / 115 / 115쪽
- 선택 페이지: 14~16쪽
- 자동 후보: `flagged=0/3`
- p14 pixel match: 91.64224%, visual_accuracy_proxy_percent: 14.64454%
- p15 pixel match: 92.89652%, visual_accuracy_proxy_percent: 11.07196%
- p16 pixel match: 92.44052%, visual_accuracy_proxy_percent: 13.93861%
- p14 review: `output/task1999_issue1999_p14_16_hwp_final/issue1999-p14-16-hwp-final/review/review_014.png`
- p15 review: `output/task1999_issue1999_p14_16_hwp_final/issue1999-p14-16-hwp-final/review/review_015.png`
- p16 review: `output/task1999_issue1999_p14_16_hwp_final/issue1999-p14-16-hwp-final/review/review_016.png`

시각 판정:

- 자동 후보는 0개지만, review 이미지를 직접 보면 기준 PDF와 배치가 아직 일치하지 않는다.
- p14는 rhwp 쪽에서 그림 2개가 하단에 먼저 나오고, 기준 PDF는 같은 그림이 p15 상단에 남는다.
- p15는 rhwp 쪽에서 `3.4.6.3` 본문부터 시작하고, 기준 PDF는 그림 2개와 `3.4.7` 뒤 표가 먼저 나온다.
- p16은 큰 겹침은 사라졌지만, 이전 페이지의 그림/표 위치 차이 때문에 흐름 정합을 최종 통과로 보기는 어렵다.

### HWPX 14~16쪽 visual sweep

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key issue1999-p14-16-hwpx-final \
  --hwp samples/issue1949_giant_cell_nested_tables_perf.hwpx \
  --pdf pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf \
  --pages 14-16 \
  --out output/task1999_issue1999_p14_16_hwpx_final \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF/render-tree: 115 / 115 / 115쪽
- 선택 페이지: 14~16쪽
- 자동 후보: `flagged=0/3`
- p14 pixel match: 91.64224%, visual_accuracy_proxy_percent: 14.64454%
- p15 pixel match: 92.89652%, visual_accuracy_proxy_percent: 11.07196%
- p16 pixel match: 92.44052%, visual_accuracy_proxy_percent: 13.93861%
- p14 review: `output/task1999_issue1999_p14_16_hwpx_final/issue1999-p14-16-hwpx-final/review/review_014.png`
- p15 review: `output/task1999_issue1999_p14_16_hwpx_final/issue1999-p14-16-hwpx-final/review/review_015.png`
- p16 review: `output/task1999_issue1999_p14_16_hwpx_final/issue1999-p14-16-hwpx-final/review/review_016.png`

시각 판정:

- HWP와 같은 결과이며, HWP/HWPX 파서 차이보다 공통 레이아웃 처리 문제로 보는 것이 타당하다.

### 회귀 테스트 보강

- `tests/issue_1949_giant_cell_render_perf.rs`에 HWP 저장본도 115쪽인지 확인하는 assert를 추가했다.
- 전체 렌더 성능 회귀 검증은 기존처럼 HWPX 전체 페이지 렌더를 유지한다.

## Stage 2 결론

- PDF/HWP/HWPX 쪽수 parity는 115 / 115 / 115로 맞다.
- 그러나 사용자가 요구한 14~16쪽 시각 검증은 아직 통과가 아니다.
- 다음 스테이지에서는 p14~p15의 그림 2개와 뒤따르는 표/본문이 기준 PDF보다 한 페이지 앞에 붙는 원인을
  문서 속성 기반으로 분석한다.
