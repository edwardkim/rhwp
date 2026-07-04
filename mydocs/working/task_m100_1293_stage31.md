# Task 1293 Stage 31: TAC 그림 실제 하단 기준 분리

## 목적

Stage30에서 `구분선 없음 + 구분선위20 + 미주사이20 + 구분선아래20` 샘플의 잔여 overflow가
한 가지 원인이 아님을 확인했다.

- `pi=593`: 실제 TAC 그림 bbox는 frame 안쪽이지만, paragraph cursor와 후행 `문20` 제목 tail이
  frame 밖으로 이어진다.
- `pi=613`: 실제 TAC 그림 bbox 자체가 frame 밖으로 나간다.

이번 단계에서는 수치별 예외를 추가하지 않고, layout overflow 판정과 pagination 예약이 공통으로
참조할 수 있는 "실제 content bottom" 기준을 좁힌다.

## 구현 방향

1. `PageItem::Shape` 처리 후 남는 `y_offset`이 실제 그림 bbox bottom이 아니라 paragraph advance나
   후행 cursor를 나타내는 경우를 분리한다.
2. TAC 그림 문단의 renderer 실제 하단과 pagination `current_height`가 다르게 보는 지점을
   `layout.rs`와 `typeset.rs`에서 비교한다.
3. `pi=593` 같은 cursor-only overflow와 `pi=613` 같은 drawing overflow를 같은 하단 tail 분기로
   처리하지 않는다.

## 우선 검증 대상

- `2024-11-practice-no-separator-above20-between20-below20`
  - page count `23/23/23` 유지
  - page 12 `pi=593` false/후행 tail 판정 정리
  - page 13 `pi=613` 실제 그림 overflow 감소
- 회귀 가드:
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between0-below20`

## 검증 계획

- `cargo fmt --all -- --check`
- `cargo test --lib compact_endnote -- --nocapture`
- `cargo build --bin rhwp`
- focused sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage31_no_separator_target --rhwp-bin target/debug/rhwp`
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage31_between20_target --rhwp-bin target/debug/rhwp`
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage31_above20_between0_target --rhwp-bin target/debug/rhwp`

## 구현 내용

- `layout.rs`의 overflow 판정에서 `PageItem::Shape`도 `last_item_content_bottom`을 우선 사용하도록
  포함했다.
- TAC picture/shape를 `layout_shape_item`에서 직접 처리할 때 실제 bbox bottom을
  `last_item_content_bottom`에 기록한다.
  - 그림은 `pic_y + pic_h`, bottom caption이 있으면 caption bottom까지 반영한다.
  - TAC shape는 `shape_y + shape_h`를 기록한다.
- 이 변경으로 `Shape` 항목의 `y_offset`이 paragraph cursor advance를 나타내는 경우와 실제 그림
  bbox bottom을 구분한다.

## 검증 결과

### no-separator target

- 산출물: `output/task1293_stage31_no_separator_target/summary.json`
- page count: `23/23/23`
- renderer overflow:
  - Stage29/30 기준 4건에서 3건으로 감소했다.
  - page 12 `pi=593`의 `Shape overflow 14.1px`가 사라졌다.
  - page 13 `pi=613`의 실제 TAC 그림 overflow는 남았다.

남은 로그:

```text
LAYOUT_OVERFLOW_DRAW: section=0 pi=613 line=0 y=1157.6 col_bottom=1092.3 overflow=65.3px
LAYOUT_OVERFLOW: page=13, sec=0, col=0, para=613, type=FullParagraph, first=false, y=1157.6, bottom=1092.3, overflow=65.3px
LAYOUT_OVERFLOW: page=13, sec=0, col=0, para=613, type=Shape, first=false, y=1157.6, bottom=1092.3, overflow=65.3px
```

### 회귀 가드

- `output/task1293_stage31_between20_guard/summary.json`
  - page count: `22/22/22`
  - `frame_overflow_pages`, title/order/equation overlap 후보 없음
  - renderer overflow 2건은 기존 `pi=671` partial paragraph 잔여로, 이번 Shape bottom 변경의 새 회귀가 아니다.
- `output/task1293_stage31_above20_between0_guard/summary.json`
  - page count: `21/21/21`
  - renderer overflow 0건 유지
  - `frame_overflow_pages`, title/order/equation overlap 후보 없음

### 테스트

- `cargo fmt --all -- --check`: 통과
- `cargo build --bin rhwp`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 28개 통과

## 폐기한 후보

`pi=613`의 실제 overflow를 없애기 위해 typeset 쪽에서 두 가지 후보를 확인했으나 모두 폐기했다.

1. TAC picture-only rewind 위치가 현재 흐름보다 앞쪽이면 순차 누적하도록 바꾸는 후보
   - `2024-11-practice-no-separator-above20-between20-below20` page count가 `24/23`으로 증가했다.
2. 같은 단 안에 반복된 TAC picture-only tail을 하단에서 다음 단으로 넘기는 후보
   - 동일하게 page count가 `24/23`으로 증가했다.

따라서 Stage31에서는 layout overflow 판정의 실제 bbox 기준만 채택한다.

## 다음 단계

`pi=613`은 오탐이 아니라 실제 TAC 그림 bbox overflow다. Stage32에서는 page count를 유지하면서
그림 묶음의 순차 렌더 y와 pagination current height 차이를 줄이는 별도 조건을 다시 잡는다. 단순히
TAC picture-only를 더 보수적으로 넘기는 방식은 page count 회귀를 만들기 때문에 제외한다.
