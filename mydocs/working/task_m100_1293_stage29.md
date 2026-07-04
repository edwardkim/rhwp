# Task 1293 Stage 29: compact 미주 TAC 그림 하단 fit 보정

## 목적

Stage28에서 `pi=593`/`pi=613` 잔여 overflow가 단순 `미주 사이` gap 문제가 아니라,
compact 미주 내부 TAC 그림/도형 only 문단의 실제 renderer y와 pagination fit 기준이 어긋나는
문제임을 확인했다.

이번 단계에서는 특정 문항 번호나 특정 샘플명을 조건으로 쓰지 않고, 다음 공통 조건에만 보정을
적용한다.

- compact 미주 흐름
- TAC 그림/도형 only 문단
- 저장 vpos rewind 또는 lazy/page base 보정으로 실제 renderer y가 sequential y보다 아래에 놓이는 경우
- 하단에서 다음 문단 또는 그림 drawing bbox가 frame 밖으로 나갈 위험이 있는 경우

## 수정 가설

현재 pagination은 `format_paragraph()`의 sequential 높이로 fit 여부를 먼저 판단한다. 그러나 renderer는
`HeightCursor::vpos_adjust()`를 통과하면서 TAC 그림 문단의 y를 저장 vpos 기준으로 다시 잡는다.

따라서 compact 미주에서 TAC 그림/도형 only 문단을 배치할 때는 다음 중 하나가 필요하다.

1. typeset fit 단계에서 renderer `HeightCursor`가 만들 y를 예측해 더 보수적으로 단/쪽을 넘긴다.
2. renderer `HeightCursor`가 하단 TAC 그림만 순차 y로 cap해 pagination과 같은 y를 쓰게 한다.

Stage28 후속 검토에서 2번은 폐기한다. renderer에서 하단 TAC 그림을 순차 y로 강제로 당기면 앞쪽
문단과 그림이 겹칠 수 있다. 실제 문제는 "그림을 어디에 그릴 것인가"보다 "그림이 그려질 하단을
pagination이 미리 예약했는가"에 가깝다.

Stage29는 1번을 좁은 조건으로 시도한다. 다만 검증 중 보이는 구분선이 없는 샘플까지 같은
조건을 적용하면 page count가 23쪽에서 24쪽으로 늘어나는 회귀가 확인되었다. 따라서 이번 단계의
채택 범위는 `보이는 구분선 + 큰 미주 사이 + local vpos rewind가 이전 content bottom을 침범하는 경우`
로 한정한다.

- compact 미주 다단 흐름에서 TAC 그림/도형 only 문단을 typeset할 때,
  저장 `LINE_SEG`/shape 높이 기준의 실제 drawing bottom이 현재 단 하단을 넘는지 계산한다.
- sequential 높이로는 들어가더라도 실제 drawing bottom이 단 하단을 넘으면, 해당 문단을 다음 단/쪽으로
  넘겨 renderer가 frame 밖으로 그리지 않게 한다.
- 문단 번호, 문제 번호, 샘플명은 조건에 쓰지 않는다. 조건은 compact 미주 + TAC 그림/도형 only +
  실제 drawing bottom fit 불일치로 한정한다.

## 검증 계획

- `cargo fmt --all -- --check`
- `cargo test --lib compact_endnote -- --nocapture`
- `cargo build --bin rhwp`
- target sweep:
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between0-below20`

## 검증 중 폐기한 후보

### 모든 compact local rewind crossing에 적용

- 수정: 이전 content bottom보다 앞쪽으로 rewind하는 문단에서는 `compact_local_rewind`와
  `tac_picture_rewind_height` 축약을 막았다.
- 결과:
  - `2024-11-practice-no-separator-above20-between20-below20`: overflow 1건으로 줄었지만
    page count가 `24/24/23`으로 회귀했다.
  - `2024-11-practice-above20-between0-below20`: 기존 0 overflow가 8건으로 회귀했다.
- 판단: renderer 정합 방향은 맞지만 적용 범위가 넓어 기본/작은 `미주 사이` 흐름까지 과하게
  보수적으로 예약한다. 폐기한다.

### 큰 미주 사이 전체에 적용

- 수정: 큰 `미주 사이`가 흡수되지 않은 경우에만 같은 crossing 보정을 적용했다.
- 결과:
  - `2024-11-practice-above20-between0-below20`: 0 overflow를 유지했다.
  - `2024-11-practice-no-separator-above20-between20-below20`: 여전히 page count가 `24/24/23`으로
    회귀했다.
- 판단: 구분선 없음 문서에서는 Stage27의 전체 gap 반영과 crossing 보정을 함께 쓰면 PDF보다 한 쪽
  늦어진다. 폐기한다.

## 채택한 수정

- `prev_en_content_bottom_vpos`를 추가해 직전 미주 문단의 실제 content bottom vpos를 추적한다.
- 단/쪽 advance, split, internal rewind advance 때는 `prev_en_bottom_vpos`와 함께
  `prev_en_content_bottom_vpos`도 초기화한다.
- 보이는 구분선이 있고 큰 `미주 사이`가 흡수되지 않은 문서에서 local vpos rewind가 직전 content
  bottom을 침범하면, pagination의 compact rewind 축약을 적용하지 않는다.
- 같은 조건에서는 TAC 그림 only 문단의 `tac_picture_rewind_height` 축약도 적용하지 않아 renderer의
  실제 drawing y와 pagination fit 판단을 맞춘다.

## 검증 결과

- `cargo fmt --all -- --check`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 28개 통과
- `cargo build --bin rhwp`: 통과
- target sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --target 2024-11-practice-above0-between20-below2 --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage29_visible_large_between_rewind_target --rhwp-bin target/debug/rhwp`

| target | page count | overflow_lines | title/order/equation/frame 후보 | 판단 |
|---|---:|---:|---:|---|
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 4 | 0 | Stage28 기준 유지. 이번 보정 대상에서 제외 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 3 | 0 | Stage27의 38건 대비 개선 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0 | 기존 0 overflow 유지 |

## 남은 문제

- `no-separator` target의 `pi=593`/`pi=613` overflow 4건은 이번 보정으로 해결하지 않는다.
  보이는 구분선이 없는 문서는 Stage27의 전체 `미주 사이` gap 반영이 필요하고, 같은 crossing
  보정을 더하면 page count가 늘어난다.
- 다음 스테이지에서는 구분선 없음 문서의 TAC 그림 문단을 별도로 보지 말고, Stage27 gap 소비 이후
  `pi=593`/`pi=613`이 frame 밖으로 나가는 원인을 renderer/pagination 공통 기준에서 다시 좁힌다.
