# Task #1728 Stage 1 보고서 — 현 devel 기준 잔여 RowBreak 시각 차이 재검증

## 배경

#1728은 #1744 merge 후에도 다음 잔여 갈래 때문에 open 상태로 유지됐다.

- scattered header RowBreak 6쪽 하단 over-fill / 행 컷 차이
- PDF 하단 rule 라인 계열 잔여 차이

현 `devel`에는 이후 #1787의 실제 변경 커밋이 통합 PR #1817로 반영되어 있다.

- #1787: `Task #1748: 컷 걸침 rowspan 셀 높이 기반 유닛 컷`
- #1817 merge commit: `7acbc314a1d6ddd1b439038f346434f11eec6268`
- 현 `devel` 포함 커밋: `1236dd023e7350f04486208dae1a7dbb7814fe3c`

따라서 이번 stage는 새 코드 수정 전에 #1728 잔여 증상이 현 `devel`에서 실제로 남아 있는지 확인하는 검증이다.

## 절차 상태

- 열린 PR 확인: 없음.
- #1728 assignee: `jangster77` 지정 완료.
- 작업 브랜치: `local/task_m100_1728` (`upstream/devel` 기준).
- #1748 issue도 아직 open 상태이나, #1787/#1817 반영으로 해결됐는지 함께 확인했다.

## 로컬 테스트

```bash
env CARGO_INCREMENTAL=0 cargo test --test issue_1748_rowbreak_straddle_rowspan -- --nocapture
```

결과:

- 3 passed / 0 failed.
- 컷 페이지 p6에서 걸친 rowspan 셀 텍스트가 셀 박스를 넘지 않는지 확인.
- 연속 페이지 p7에서 걸친 rowspan 셀이 처음부터 중복 재렌더되지 않는지 확인.

## PDF 기준 시각 재검증

`scripts/task1274_visual_sweep.py`는 선택 페이지만 분석하더라도 현재 구현상 전체 문서 SVG/PDF raster를 먼저
생성한다. giant 42쪽 전체 export가 오래 걸려 중단하고, 같은 가이드의 산출물 형식(`compare`, `overlay`,
`review`)을 유지하도록 5~6쪽만 직접 생성했다.

사용 경로:

- giant 입력: `samples/task1718/table_giant_cell_overfill.hwp`
- giant 기준 PDF: `pdf/table_giant_cell_overfill-2024.pdf`
- scattered 입력: `samples/table_scattered_header_rowbreak.hwp`
- scattered 기준 PDF: `pdf/table_scattered_header_rowbreak-2024.pdf`
- 산출물: `output/task1728_manual_visual/`

대표 review PNG:

| target | page | review PNG | visual_accuracy_proxy_percent |
|--------|------|------------|-------------------------------|
| giant | 5 | `output/task1728_manual_visual/giant-rowbreak/review/review_005.png` | 9.45677 |
| giant | 6 | `output/task1728_manual_visual/giant-rowbreak/review/review_006.png` | 9.97716 |
| scattered | 5 | `output/task1728_manual_visual/scattered-rowbreak/review/review_005.png` | 6.40070 |
| scattered | 6 | `output/task1728_manual_visual/scattered-rowbreak/review/review_006.png` | 6.06970 |

`visual_accuracy_proxy_percent`는 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값이다. 이 케이스는
폰트/선/표 텍스트 세부 차이가 많아 수치가 낮지만, 이슈의 핵심 판정은 96 DPI dark-pixel bbox의 top/bottom 차이와
표 하단 over-fill 여부다.

## 96 DPI dark-pixel bbox 결과

기준: grayscale `<160`, 페이지 PNG 기준.

### giant RowBreak

| page | PDF bbox | rhwp bbox | delta |
|------|----------|-----------|-------|
| p5 | `(82, 94, 710, 1099)` | `(80, 91, 709, 1098)` | `(-2, -3, -1, -1)` |
| p6 | `(82, 94, 710, 1099)` | `(80, 91, 709, 1097)` | `(-2, -3, -1, -2)` |

#1728 최초 관찰의 giant p5/p6 top `-17~-18px`, bottom `+16px` 유형은 현 `devel`에서 재현되지 않는다.

### scattered RowBreak

| page | PDF bbox | rhwp bbox | delta |
|------|----------|-----------|-------|
| p5 | `(77, 77, 718, 1081)` | `(75, 75, 717, 1075)` | `(-2, -2, -1, -6)` |
| p6 | `(77, 77, 718, 1081)` | `(75, 75, 717, 1077)` | `(-2, -2, -1, -4)` |

#1728/#1748 최초 관찰의 scattered p6 bottom `+13px` over-fill은 현 `devel`에서 `-4px`로 바뀌어
게이트 `|dBot| <= 5px` 안에 들어온다.

## 판단

현 `devel` 기준으로 #1728의 잔여 핵심 증상은 해결된 것으로 판단한다.

- footer 쪽번호 / giant continuation 상단 spacing: #1744로 반영 완료.
- scattered p6 하단 over-fill / 걸친 rowspan 셀 중복 렌더: #1787 실제 변경이 #1817로 반영 완료.
- #1728 본문 기준 p5/p6 top/bottom 수치가 모두 허용 범위로 들어왔다.

따라서 새 코드 수정 없이 #1728과 #1748을 close 후보로 판단했다. 작업지시자 승인 후 두 이슈 모두 close 완료했다.

- #1728 close comment: https://github.com/edwardkim/rhwp/issues/1728#issuecomment-4870196637
- #1748 close comment: https://github.com/edwardkim/rhwp/issues/1748#issuecomment-4870196842

## 실제 close 코멘트

### #1728

```markdown
현 `devel` 기준으로 #1728의 잔여 PDF 시각 차이를 재검증했습니다.

- #1744: footer 쪽번호 위치와 giant continuation 상단 `spacing_before` 갈래 반영
- #1817: #1787의 `Task #1748` 변경을 통합해 scattered RowBreak p6 하단 over-fill 갈래 반영
- 현 `devel` 검증:
  - `cargo test --test issue_1748_rowbreak_straddle_rowspan` 통과
  - giant p5/p6 96 DPI dark-pixel bbox: top `-3px`, bottom `-1~-2px`
  - scattered p6 96 DPI dark-pixel bbox: `dBot=-4px`, 게이트 `|dBot| <= 5px` 통과

따라서 #1728에서 추적하던 RowBreak continuation 5~6쪽 PDF 기준 시각 차이는 현재 기준 해결된 것으로 보고 close 합니다.
```

### #1748

```markdown
현 `devel` 기준으로 #1748 재현 샘플을 다시 확인했습니다.

- #1787의 실제 변경은 통합 PR #1817에 cherry-pick 되어 merge 되었습니다.
- merge commit: `7acbc314a1d6ddd1b439038f346434f11eec6268`
- 현 `devel` 검증:
  - `cargo test --test issue_1748_rowbreak_straddle_rowspan` 통과
  - `samples/table_scattered_header_rowbreak.hwp` p6 bbox: PDF `(77,77,718,1081)`, rhwp `(75,75,717,1077)`, `dBot=-4px`
  - 기대 게이트 `|dBot| <= 5px` 통과

따라서 scattered header RowBreak p6 하단 over-fill은 해결된 것으로 보고 close 합니다.
```
