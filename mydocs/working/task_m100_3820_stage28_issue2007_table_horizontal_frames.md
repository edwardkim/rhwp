---
kind: working
status: completed
canonical: mydocs/working/task_m100_3820_stage28_issue2007_table_horizontal_frames.md
last_verified: 2026-08-06
---

# Stage 28 — issue2007 p9–p14 표 상·하단선 브라우저 paint 정합

## 목표

`samples/basic/issue2007_nested_cell_pagination_42065.hwp`의 rhwp 페이지 9–14에서
중첩/연속 표의 상단 또는 하단 가로선이 기준 PDF와 다르게 보이는 결함을 재현하고 수정한다.
기준은 `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`의 같은 물리 페이지다.

## 판정 경계

- 이전 Stage 25–27의 native SVG → raster 증적은 source-flow와 node 존재 여부를 확인한 자료다.
  `TableCell`/Body clip 경계에서 Canvas/SVG stroke가 반폭 잘리는 브라우저 paint 동등성까지
  증명하지 못한다. 따라서 이 단계에서는 이를 최종 시각 통과 근거로 사용하지 않는다.
- `fidelity_compare`의 Chrome SVG 캡처, SVG clipPath·line 좌표 원장, 기준 PDF raster를 함께
  대조한다. 픽셀 점수는 후보 우선순위이며 최종 판정은 페이지별 시각 대조로 한다.
- WASM 패키지 빌드는 사용자가 수동으로 수행한다. 이 단계에서 `wasm-pack build`는 실행하지 않는다.

## 재현 계획

1. 최신 native CLI로 대상 SVG와 render tree를 다시 생성한다.
2. p9–p14의 `cell-clip-*`/`body-clip-*` 사각형과 표 외곽 가로 `Line`의 stroke 구간을
   대조해 clip 경계 절단 여부를 수집한다.
3. Chrome 기반 fidelity 비교 시트와 기준 PDF를 페이지별로 확인해 증상을 확정한다.
4. clip이 테두리 stroke의 반폭을 자르지 않도록 SVG와 Canvas 공통 clip 계약을 보정하고,
   상단·하단 가로선 회귀를 추가한다.
5. focused 회귀와 동일 명령의 재대조 결과를 assets에 남긴다.

## 진행 기록

- 2026-08-06: 사용자가 p9–p14 전부에서 표 상단 또는 하단 가로선이 정확히 표시되지 않음을
  보고했다. 브라우저 제어 세션은 연결 가능한 브라우저가 없어 사용할 수 없으므로,
  fidelity_compare가 사용하는 독립 Chrome 캡처와 SVG/clip 원장으로 재현한다.

## 재현 결과

- 수정 전 p9의 8×4 표 하단선 centerline은 `y=997.3467`, stroke 폭은 `0.5px`였으나,
  유효 `body-clip-3 → cell-clip-6 → cell-clip-8`의 하단은 `y=992.4`였다. 따라서
  source `Line` node는 존재해도 paint 영역이 0%가 되어 최종 SVG/Canvas에서 사라졌다.
- p10과 p13은 한 단계 바깥의 1×1 RowBreak wrapper에는 테두리가 없고 그 안쪽 표의
  실제 하단선이 다음 물리 페이지에 있었다. 기존의 “clip 경계 근처 source line” 검사만으로는
  현재 조각의 하단 frame 자체가 없다는 결함을 잡지 못했다.
- 수정 전 자동 후보는 p9 1건, p10 1건, p11 2건, p12 1건, p13 1건, p14 2건이었다.
  원장은 [before 후보](../pr/assets/task_m100_3820_stage28_issue2007_table_horizontal_frames/horizontal_candidates_before.tsv)에
  보관했다.

## 구현

- 완료된 표의 외곽 가로선이 wrapper clip 밖으로 최대 6px만 벗어난 경우에만 그 stroke paint
  영역을 포함하도록 clip을 확장했다. 임의의 후속 본문은 노출하지 않는다.
- 연속 표는 원래 표가 아닌 unbordered RowBreak wrapper 아래의 실제 bordered table까지 내려가,
  상·하단 frame centerline을 stroke 반폭과 작은 여유만큼 clip 안쪽에 배치한다.
  이전 페이지에 속하는 6px 미만 residual tail은 다시 frame으로 만들지 않아 p10/p13의 false top
  border를 막는다.
- `fidelity_compare`에 `svg-table-horizontal-border-clip-candidates.tsv`를 추가했다. source line이
  clip 경계에서 잘린 경우뿐 아니라, 다음 페이지에만 source 하단선이 있어 현재 fragment frame이
  빠진 경우도 후보화한다. 같은 table의 paint-safe frame이 있으면 오래된 off-page source line은
  중복 경보하지 않는다.

## 검증

- `CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2007_nested_cell_pagination`
  — 9 passed.
- `/tmp/rhwp-stage28-venv.wBrnNs/bin/python scripts/tests/test_fidelity_compare.py`
  — 41 passed. clip으로 완전히 잘린 하단선과 source 하단선이 멀리 있어 현재 physical frame이
  빠진 경우를 각각 회귀로 고정했다.
- 최신 native CLI로 p9–p14를 `--text-only --layout-ledger` 재실행했다. 요청 6쪽/완료 6쪽/누락 0쪽,
  가로 frame 후보는 전 페이지 0건이다. [after 후보](../pr/assets/task_m100_3820_stage28_issue2007_table_horizontal_frames/horizontal_candidates_after.tsv)를
  보관했다.
- 독립 Chrome SVG 캡처의 PDF·수정 전·수정 후 페이지별 PNG는
  [`mydocs/pr/assets/task_m100_3820_stage28_issue2007_table_horizontal_frames/`](../pr/assets/task_m100_3820_stage28_issue2007_table_horizontal_frames/)에
  보관했다. 이 Chrome은 한컴 글꼴 fallback이 달라 전체 pixel diff%를 통과 판정으로 사용하지 않았고,
  이 단계의 판정 대상인 상·하단 table frame의 paint와 clip 원장을 함께 확인했다.
- WASM 패키지 빌드는 사용자 수동 검증 범위이므로 이 단계에서 실행하지 않았다.
