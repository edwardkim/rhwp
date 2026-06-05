# Task 1284 Stage 13: 미주사이20 문28 좌단 tail 수식 이월 보정

## 배경

- PR 준비 CI를 재시작하던 중, 작업지시자가 `3-09월_교육_통합_2024-미주사이20.hwp` 22쪽 문28 시각 불일치를 추가 확인했다.
- 한컴 기준으로는 왼쪽 단 하단의 문28 끝 수식/꼬리 줄이 오른쪽 단 첫 줄로 이월되어야 한다.
- 현재 rhwp 렌더링은 해당 꼬리 줄을 왼쪽 단 하단에 남겨 마지막 줄과 겹쳐 보인다.

## 관찰 항목

- 대상 문서: `3-09월_교육_통합_2024-미주사이20.hwp`
- 위치: 22쪽, 왼쪽 단 하단 문28
- 기대: 문28 마지막 tail 수식/줄이 오른쪽 단 첫 줄로 넘어가며 좌단 하단에서 겹치지 않는다.
- 현재: 문28 마지막 tail 수식/줄이 좌단 하단에 잔류해 하단 텍스트와 overlap 된다.

## 분석 계획

- `output/task1274/2024-09-between20/analysis/render_tree_022.json`과 `question_flow.json`에서 문28 관련 paragraph index, column, bbox를 확인한다.
- 현 sweep이 이 유형을 탐지하지 못한다면, 좌단 tail line order/column 이월 후보를 `task1274_visual_sweep.py`에 추가할지 검토한다.
- pagination/typeset 쪽에서 last-column/page-tail 보정과 달리 “왼쪽 단 하단에서 다음 단으로 보내야 하는 tail” 분기가 누락됐는지 확인한다.

## sweep 누락 원인

- sweep이 신호를 완전히 못 본 것은 아니다.
  - `metrics.json` 22쪽 `column_line_band_drift_candidates`에는 우단 평균 `53.7px`, p90 `72.5px` 후보가 들어가 있다.
  - 그러나 현재 flag 승격 조건은 `equation_text_overlap`, `question_title_text_overlap`, `line_order_overlap`, `question_marker_drift` 중 하나가 있어야 `column_line_band_drift`를 붙인다.
  - 22쪽은 이 semantic flag가 비어 있어 최종 `flags: []`로 떨어졌다.
- 하단 overflow도 숫자로는 보였지만 관용 처리됐다.
  - rhwp frame bottom은 약 `1097px`이고, 문제 줄은 render tree 기준 `pi=1114`, bbox `y=1098.2`, `h=16.8`이다.
  - `rhwp_outside_frame_pixels=11`, `rhwp_outside_frame_extent_px=5`, `content_bottom_delta_px=4.0`이라 `frame_overflow_tolerated_bleed=true`가 되어 flag에서 제외됐다.
- 기존 equation/text overlap 검출은 “수식 bbox가 다른 텍스트 bbox를 직접 덮는 경우” 중심이다.
  - 이번 문제는 수식 자체가 텍스트를 직접 덮는다기보다, 좌단 하단에 남으면 안 되는 tail 줄이 frame 밖/하단에 잔류한 column flow 오류다.
  - 따라서 `equation_text_overlap_candidates`와 `line_order_overlap_candidates` 모두 비어 있었다.

## sweep 보강 방향

- render tree 기준으로 frame bottom을 넘는 `TextLine`/`Equation` tail line을 별도 후보로 잡는다.
- 하단 bleed가 작더라도 해당 line이 body column 마지막 줄이고 PDF의 대응 column line count/drift가 큰 경우는 관용 처리하지 않는다.
- `column_line_band_drift_candidates`가 단독으로 커도 annotation PNG를 생성하도록 flag 승격 조건을 완화한다.

## 구현

- `tests/issue_1139_inline_picture_duplicate.rs`
  - `issue_1284_2024_between20_page22_23_question_tail_matches_pdf`에 page22 문28 tail 검증을 추가했다.
  - `pi=1114` 마지막 ㉡ 식이 page22 오른쪽 단 첫 item으로 이동하고, `pi=1115` 그래프가 그 뒤에 이어지는지 dump와 render tree bbox로 고정했다.
- `src/renderer/typeset.rs`
  - 큰 `미주 사이` 다단 미주에서 현재 tail 한 줄 뒤에 vpos rewind를 가진 TAC 그림-only 문단이 이어질 때, tail 한 줄도 그림과 함께 다음 단으로 넘기도록 보정했다.
  - 대상 패턴은 `ep_idx > 0`, 다음 단이 남은 상태, 현재 단 하단 88% 이후, 한 줄 tail, 다음 문단이 treat-as-char picture/shape only이고 vpos가 되감기는 경우로 제한했다.
- `scripts/task1274_visual_sweep.py`
  - render tree의 `TextLine` bbox가 frame bottom을 넘는 하단 tail 후보를 `render_tree_frame_tail_overflow`로 검출한다.
  - 기존 픽셀 기반 frame bleed가 관용 처리되더라도 tail bbox와 column/line drift가 같이 있으면 annotation PNG를 생성한다.
  - summary 출력에 `tail=[...]` 항목을 추가했다.

## 검증

- `cargo fmt --all -- --check`: 통과.
- `python3 -m py_compile scripts/task1274_visual_sweep.py`: 통과.
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20_page22_23_question_tail_matches_pdf -- --nocapture`: 통과.
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`: 5개 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 62개 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`: `flagged=0/24`, `tail=[]`.
- `python3 scripts/task1274_visual_sweep.py`: 전체 6종 수행 완료.
  - `2024-09-between20`: `flagged=0/24`, `tail=[]`.
  - 새 tail 검출기가 기존 잠복 후보 6쪽을 추가로 발견했다.

## 후속 후보

새 sweep 검출기는 stage13 대상 문서는 깨끗하게 통과시켰지만, 다른 문서에서 아래 후보를 추가로 잡았다. 별도 stage에서 annotation PNG와 PDF를 대조해 실제 수정 대상인지 판단한다.

| 대상 | 쪽 | 후보 | annotation |
|---|---:|---|---|
| `2023-09` | 16 | `pi=845`, 문28, 오른쪽 단, `overflow=11.0px` | `output/task1274/2023-09/analysis/annotated_016.png` |
| `2023-09` | 19 | `pi=953`, 문29, 오른쪽 단, `overflow=14.4px`; `pi=935`, 문28, 왼쪽 단, `overflow=5.4px` | `output/task1274/2023-09/analysis/annotated_019.png` |
| `2022-10` | 14 | `pi=779`, 문25, 오른쪽 단, `overflow=17.0px` | `output/task1274/2022-10/analysis/annotated_014.png` |
| `2022-10` | 17 | `pi=962`, 문29, 오른쪽 단, `overflow=16.8px` | `output/task1274/2022-10/analysis/annotated_017.png` |
| `2022-11-practice` | 11 | `pi=553`, 문14, 오른쪽 단, `overflow=9.7px` | `output/task1274/2022-11-practice/analysis/annotated_011.png` |
| `2022-11-practice` | 19 | `pi=906`, 문25, 왼쪽 단, `overflow=5.5px` | `output/task1274/2022-11-practice/analysis/annotated_019.png` |

## 검증 계획

- 우선 `tests/issue_1139_inline_picture_duplicate.rs`의 관련 task 1284 테스트를 확장해 문28 tail이 22쪽 오른쪽 단으로 이월되는 조건을 고정한다.
- 수정 후 해당 테스트만 먼저 실행한다.
- sweep 전체와 PR 준비 CI는 시각 보정 완료 후 다시 수행한다.
