# Task 1293 Stage 45: 구분선위0 미주사이20 구분선아래2 page count 보정

## 배경

Stage44 전체 sweep에서 `2024-11-practice-above0-between20-below2`
(`3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp`)만
page count가 PDF와 맞지 않았다.

- SVG/render tree: 23쪽
- PDF: 22쪽
- overflow:
  - page 14 `pi=671` PartialParagraph 48.2px

## 목적

`구분선 위=0`, `미주 사이=20mm`, `구분선 아래=2mm` profile에서 큰 `미주 사이` 값이
어느 경계에서 pagination 높이로 소비되고, 어느 경계에서 저장 vpos에 흡수되어야 하는지
확인한다. 최종적으로 PDF와 같은 22쪽 흐름으로 맞추되, 다른 미주 설정 샘플의 page count와
overflow를 회귀시키지 않는다.

## 확인 계획

1. `output/task1293_stage44_full_sweep/2024-11-practice-above0-between20-below2`의
   render tree와 compare PNG에서 PDF 대비 첫 page drift가 커지는 위치를 찾는다.
2. `pi=671` 전후 문단의 페이지/단 배치를 dump한다.
3. `미주 사이 20mm` 경계 gap이 pagination에서 과다 소비되는지, renderer에서 과다 보존되는지
   `RHWP_VPOS_DEBUG`/target sweep으로 확인한다.
4. 수정은 개별 문단 번호가 아니라 이 profile의 공식 미주 모양 값과 compact 미주 흐름 구조로
   제한한다.

## 검증 계획

- 단일 target:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage45_single --rhwp-bin target/debug/rhwp`
- focused sweep:
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above0-between7-below2`
  - `2024-11-practice-above20-between7-below2`
  - `2024-11-practice-no-separator-above20-between20-below20`
- `cargo test --lib compact_endnote -- --nocapture`

## 분석 결과

현재 HEAD 기준 단일 sweep에서 mismatch가 재현되었다.

- 명령:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage45_single_baseline --rhwp-bin target/debug/rhwp`
- 결과:
  - SVG/PDF: `23/22`
  - render tree/PDF: `23/22`
  - overflow:
    - `LAYOUT_OVERFLOW_DRAW: section=0 pi=671 line=2 y=1140.5 col_bottom=1092.3 overflow=48.2px`
    - `LAYOUT_OVERFLOW: page=14, sec=0, col=0, para=671, type=PartialParagraph, first=false, y=1140.5, bottom=1092.3, overflow=48.2px`

임시 worktree probe로 Stage35 이후 커밋별 render tree page count를 확인했다.

| commit | 결과 |
|---|---:|
| `f27f0039` | 22쪽 |
| `c82e838a` | 22쪽 |
| `54597dfd` | 22쪽 |
| `45dcb5a3` | 23쪽 |
| `529571ef` | 23쪽 |
| `86c9b584` | 23쪽 + `pi=671` overflow |
| `822cf962` | 23쪽 + `pi=671` overflow |
| `8894f5a9` | 23쪽 + `pi=671` overflow |

원인은 두 갈래였다.

1. `45dcb5a3`의 TAC picture-only 묶음 이동 조건이 모든 `local_vpos_rewind` TAC 그림에 적용됐다.
   이 조건은 `구분선위20/미주사이7/구분선아래2`처럼 compact 미주 사이 흐름에서 필요한 보정이지만,
   `구분선위0/미주사이20/구분선아래2`처럼 큰 미주 사이가 별도 gap으로 소비되는 profile에도 발동해
   후반부 문30 TAC 그림 묶음을 한 쪽 늦게 밀었다.
2. `86c9b584`에서 internal rewind split-1 보정을 `compact_between_notes_gap`으로만 제한하면서,
   큰 미주 사이가 흡수되지 않는 profile의 `pi=671` partial split이 `lines=0..2`에서 `lines=0..3`으로
   바뀌었다. 그 결과 세 번째 줄이 좌측 단 하단에 남아 renderer overflow가 발생했다.

## 수정

- `src/renderer/typeset.rs`
  - `tac_picture_rewinds_before_column_base`에 `compact_between_notes_gap` 조건을 추가했다.
    - TAC 그림 묶음 강제 이동은 compact 미주 사이 profile에서만 적용한다.
    - 큰 미주 사이가 별도 gap으로 소비되는 profile은 기존처럼 같은 쪽 흐름을 유지한다.
  - internal rewind split-1 보정 조건을
    `compact_between_notes_gap || large_between_notes_gap_before_rewind`로 확장했다.
    - compact 흡수형 profile은 Stage40 보정을 유지한다.
    - 흡수되지 않은 큰 미주 사이 profile은 Stage35처럼 마지막 포함 줄을 다음 단으로 보내
      renderer overflow를 사전에 차단한다.

## 검증 결과

### 단일 target

- 명령:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage45_single_fix2 --rhwp-bin target/debug/rhwp`
- 결과:

| target | page count | overflow | frame/title/order/equation |
|---|---:|---:|---:|
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 0 | 0/0/0/0 |

`dump-pages -p 14` 기준 `pi=671` split도 Stage35와 같은 형태로 복구되었다.

- 수정 전: `lines=0..3`, `lines=3..4`
- 수정 후: `lines=0..2`, `lines=2..4`

### focused sweep

- 명령:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --target 2024-11-practice-above0-between7-below2 --target 2024-11-practice-above20-between7-below2 --target 2024-11-practice-shape987 --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage45_focused --rhwp-bin target/debug/rhwp`
- 결과:

| target | page count | overflow | frame/title/order/equation |
|---|---:|---:|---:|
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 0 | 0/0/0/0 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-shape987` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 0 | 0/0/0/0 |

### 단위/형식 검증

- `cargo build --bin rhwp`: passed
- `cargo test --lib compact_endnote -- --nocapture`: 28 passed
- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed

## 다음 단계

Stage45 수정은 Stage44 최우선 mismatch target을 정상화했다. 다음 stage에서는 전체 sweep을 다시
수행해 Stage44에 남아 있던 기존 교육 통합 계열 overflow 후보를 재분류한다.
