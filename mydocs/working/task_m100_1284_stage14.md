# Task 1284 Stage 14: 전체 sweep tail 후보 판별

## 배경

- stage13에서 `3-09월_교육_통합_2024-미주사이20.hwp` 22쪽 문28 tail 이월 문제를 수정했다.
- 동시에 sweep에 `render_tree_frame_tail_overflow` 검출을 추가했다.
- 수정 대상 문서인 `2024-09-between20`은 `flagged=0/24`, `tail=[]`로 정리됐다.
- 전체 6종 sweep에서는 새 검출기가 다른 문서의 하단 tail 후보 6쪽을 추가로 잡았다.

## 남은 후보

| 대상 | 쪽 | 후보 | annotation |
|---|---:|---|---|
| `2023-09` | 16 | `pi=845`, 문28, 오른쪽 단, `overflow=11.0px` | `output/task1274/2023-09/analysis/annotated_016.png` |
| `2023-09` | 19 | `pi=953`, 문29, 오른쪽 단, `overflow=14.4px`; `pi=935`, 문28, 왼쪽 단, `overflow=5.4px` | `output/task1274/2023-09/analysis/annotated_019.png` |
| `2022-10` | 14 | `pi=779`, 문25, 오른쪽 단, `overflow=17.0px` | `output/task1274/2022-10/analysis/annotated_014.png` |
| `2022-10` | 17 | `pi=962`, 문29, 오른쪽 단, `overflow=16.8px` | `output/task1274/2022-10/analysis/annotated_017.png` |
| `2022-11-practice` | 11 | `pi=553`, 문14, 오른쪽 단, `overflow=9.7px` | `output/task1274/2022-11-practice/analysis/annotated_011.png` |
| `2022-11-practice` | 19 | `pi=906`, 문25, 왼쪽 단, `overflow=5.5px` | `output/task1274/2022-11-practice/analysis/annotated_019.png` |

## 분석 목표

- 각 후보가 실제 한컴/PDF 대비 오류인지, 또는 render tree line box가 큰 수식 line-height를 포함해 생긴 sweep 오탐인지 구분한다.
- 실제 오류라면 기존 공통 tail 이월 로직으로 처리 가능한 패턴인지 확인한다.
- 오탐이라면 sweep 필터를 더 정교하게 만들어 실제 픽셀/컬럼 흐름 차이가 작은 후보는 제외한다.

## 진행 순서

1. annotation PNG와 compare PNG를 열어 후보별 실제 시각 차이를 확인한다.
2. `metrics.json`의 `render_tree_frame_tail_overflow_candidates`, `column_line_band_drift_candidates`, `rhwp_outside_frame_pixels`를 후보별로 대조한다.
3. 실제 오류 후보를 우선순위로 정하고, 테스트를 먼저 추가한다.
4. 소스 수정이 필요한 경우 작업지시자 승인 후 구현한다.

## 검증 계획

- 후보별 targeted test를 추가한 뒤 해당 테스트를 먼저 실행한다.
- `python3 scripts/task1274_visual_sweep.py --target <target>`로 해당 문서 focused sweep을 재확인한다.
- 후보가 정리되면 전체 sweep을 다시 수행한다.

## 구현 결과

### 실제 레이아웃 보정

- `3-10월_교육_통합_2022.hwp` 14쪽 문25
  - 오른쪽 단 하단에서 큰 inline 수식 tail 뒤 문25 제목이 저장 vpos 전체를 따라가면 뒤 풀이가 frame 밖으로 밀렸다.
  - `compact_endnote_question_title_after_tall_tail_backtrack` 공통 분기를 추가해 단 하단 큰 수식/inline 뒤 문항 제목은 제한 폭만 되감도록 했다.
  - 문25 tail `pi=779`가 page 14 frame 안에서 끝나는지 테스트로 고정했다.

- `3-10월_교육_통합_2022.hwp` 17쪽 문28/문29
  - 오른쪽 단 상단/중단에서 큰 수식 tail 뒤 새 문항 제목이 이미 PDF 순차 흐름과 맞는데도 저장 vpos forward를 따르면 문29 tail이 하단 frame 밖으로 밀렸다.
  - `compact_endnote_question_title_after_tall_upper_flow` 분기를 추가해 이 조건에서는 순차 y를 유지하고, 후속 미주 base만 같은 폭으로 보정하도록 했다.
  - 문28 제목, 문29 제목, 문29 tail bottom을 테스트로 고정했다.

- `3-11월_실전_통합_2022.hwp` 11쪽 문13/문14
  - 기본 7mm compact 미주에서 텍스트가 섞인 큰 수식 line 뒤 문항 제목이 line spacing 전체만큼 내려가 문14 tail이 하단으로 밀렸다.
  - `compact_endnote_question_title_after_tall_regular_gap` 분기를 추가해 보이는 line bottom 뒤 짧은 gap으로 붙도록 했다.
  - 문13/문14 제목과 문14 tail bottom을 테스트로 고정했다.

- `3-11월_실전_통합_2022.hwp` 19쪽 문24/문25
  - 단일 줄 prev 뒤 medium stale forward vpos에 7mm gap을 추가하면서 문25 tail이 하단 frame에 걸렸다.
  - `compact_endnote_stale_note_gap` 적용 조건을 큰 stale forward에만 걸리도록 좁혀, 순차 y가 이미 PDF 위치와 맞는 medium stale 케이스는 그대로 두었다.
  - 문24/문25 제목과 문25 tail bottom을 테스트로 고정했다.

- `3-09월_교육_통합_2023.hwp` 19쪽 문29
  - 기본 compact 미주 다줄 tail 뒤 새 문항 제목에서 큰 40px buffer를 쓰면 뒤 풀이가 frame 밖으로 밀렸다.
  - `bottom_new_note_gap_cap`의 기본 compact 다줄 tail 케이스는 `prev_line_spacing + 18px`로 축소했다.
  - 문29 제목과 tail bottom을 테스트로 고정했다.

### sweep 검출 보정

- `scripts/task1274_visual_sweep.py`에 `suppress_tolerated_frame_tail_candidates`를 추가했다.
- render tree line box만 큰 수식 높이를 포함해서 살짝 frame 아래로 보이지만, 실제 PDF/RHWP pixel bottom 차이가 작고 marker drift가 없는 후보는 `render_tree_frame_tail_overflow_suppressed_candidates`로 분리한다.
- 이 필터로 `3-09월_교육_통합_2023.hwp` 16쪽/19쪽의 작은 line-height 기반 tail 후보는 실제 flag에서 제외된다.

### 테스트 기대값 정정

- `3-09월_교육_통합_2024-미주사이20.hwp` 21쪽 문24 본문 첫 줄 테스트는 PDF의 일반 글자 y(약 294px)만 보던 설명이 과했다.
- 같은 줄의 큰 수식 상단은 PDF bbox 기준 약 284.5px이고, render tree `TextLine` bbox는 이 line box 상단을 대표하므로 허용 범위와 설명을 해당 의미에 맞게 정정했다.

## 검증 결과

| 명령 | 결과 |
|---|---|
| `python3 -m py_compile scripts/task1274_visual_sweep.py` | 통과 |
| `cargo test --test issue_1139_inline_picture_duplicate issue_1284 -- --nocapture` | 16 passed |
| `cargo test --lib compact_endnote -- --nocapture` | 31 passed |
| `python3 scripts/task1274_visual_sweep.py` | 6종 전체 `flagged=0`: `2022-09 0/23`, `2023-09 0/20`, `2024-09-below20 0/23`, `2024-09-between20 0/24`, `2022-10 0/18`, `2022-11-practice 0/21` |
