# Task M100 #1459 Stage 2

- 작성일: 2026-06-22
- 모드: 기여자 모드. PR 전 오늘할일 문서는 생성하거나 갱신하지 않는다.
- 이전 커밋: `7e0b2efe task 1459: 자리차지 그림 혼합 문단 렌더 보정`

## 목표

한컴과 rhwp의 `투명도0-50-2nd그림글차처럼off.hwp` 세로 배치 차이를 줄인다.

Stage 1은 두 번째 비-TAC `TopAndBottom` 그림이 먼저 렌더되고 첫 번째 TAC 그림이 누락되지 않는 조건까지 해결했다. 그러나 rhwp는 두 그림 사이 간격이 한컴보다 크게 벌어진다. Stage 2에서는 임의 보정이 아니라 저장된 `LINE_SEG.vertical_pos`, 그림 공통 속성, 비-TAC `TopAndBottom` 배치 기준을 근거로 세로 위치를 재계산한다.

## 현재 관찰

- 문단 0의 `LINE_SEG`는 `vpos=7733`, `lh=7733`, `baseline=6573`, `line_spacing=600`이다.
- PageItem은 `FullParagraph(pi=0)`, `Shape(ci=2, tac=true)`, `Shape(ci=3, tac=false)` 순서로 생성된다.
- 한컴은 두 그림이 같은 문단 흐름 안에서 1행 간격으로 배치된다.
- rhwp는 TAC 그림에 비-TAC 그림 높이를 한 번 더 더해 그림 간격이 과대해졌다.

## 작업 가설

- TAC 그림 y는 같은 문단의 비-TAC `TopAndBottom` 예약 높이를 단순 합산하지 말고, 저장된 `LINE_SEG.vertical_pos`/line advance와 비-TAC 그림의 실제 배치 y를 기준으로 결정해야 한다.
- 비-TAC 그림 자체도 문단 시작 y와 `LINE_SEG.vertical_pos`를 중복 적용하거나 누락하는지 확인한다.

## 검증 계획

- 샘플 render tree에서 `ci=2`, `ci=3` y/bottom 값을 비교한다.
- 한컴 기준과 같은 “두 번째 50% 그림이 위, 첫 번째 0% TAC 그림이 아래, 간격은 한 줄 높이 수준” 조건을 테스트로 강화한다.
- 기존 #1452, #1139 관련 회귀 테스트를 재실행한다.

## 수정 결과

- `paragraph_layout`의 TAC 그림 y 계산에서 `para_topbottom_line_vpos_base`가 활성화된 경우 sibling `TopAndBottom` 예약 높이를 다시 더하지 않도록 했다.
- 이 샘플은 `LINE_SEG.vertical_pos=7733`이 이미 비-TAC 그림 높이만큼 다음 줄을 이동시킨 상태였으므로, 기존 Stage 1 보정은 같은 높이를 한 번 더 더하는 중복 적용이었다.
- #1459 통합 테스트는 두 이미지의 render bbox가 y 순서대로 이어지고 gap이 2px 이하인지 검증하도록 강화했다.

## 검증 결과

- `cargo test --profile release-test --test issue_1459_topbottom_picture_reflow -- --nocapture`
- `cargo test --profile release-test --lib topandbottom -- --nocapture`
- `cargo test --profile release-test --lib issue1452 -- --nocapture`
- `cargo test --profile release-test --test issue_1452_saved_caret -- --nocapture`
- `cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo build --release --bin rhwp`
- `target/release/rhwp export-render-tree samples/투명도0-50-2nd그림글차처럼off.hwp --page 0`
  - y 순서 기준 이미지 bbox: `132.3..235.4`, `235.4..338.5`
  - vertical gap: `0.0px`
- `wasm-pack build --target web --out-dir pkg`
