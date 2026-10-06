# Task M100 #1452 Stage 10 기록

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21
- 선행 커밋:
  - `2fde7a01 task 1452: 저장 커서와 문단부호 표시 정합 개선`

## 1. 배경

Stage 9에서 `samples/투명도0-50.hwp`의 저장 커서 복원, 문단부호/조판부호 표시 유지,
문단부호 표시 중 커서 y 보정을 적용했다.

사용자 수동 비교 결과, x 위치는 맞지만 캐럿이 아직 한컴 기준보다 위쪽에 있어 파란 조판부호와
동일한 세로 위치로 보이지 않는다.

## 2. 개선 목표

- 문단부호/조판부호 표시 상태에서 TAC 그림 뒤 저장 커서의 x 위치는 유지한다.
- 커서와 문단부호 위치를 임의 보정값이 아니라 인라인 그림 bbox의 세로 위치를 기준으로 계산한다.

## 3. 검증 계획

- `cargo fmt --check`
- `cargo test --test issue_1452_saved_caret -- --nocapture`
- `wasm-pack build --target web --out-dir pkg`
- `http://localhost:7700/` 브라우저에서 `samples/투명도0-50.hwp` 캐럿 위치 확인

## 4. 변경 내용

- `src/document_core/queries/cursor_rect.rs`
  - 문단부호 표시 여부에 따른 임의 y 보정을 제거했다.
  - 인라인 그림 앞/뒤 캐럿은 해당 그림 bbox에서 계산된 줄 메트릭을 그대로 사용한다.
  - 첫 번째 그림 앞(offset `0`)은 첫 번째 그림 기준선, 두 번째 그림 앞/뒤(offset `1`, `2`)은 두 번째 그림 기준선을 사용한다.
  - 인라인 그림 주변 텍스트 기준선을 찾지 못하는 경우 fallback을 그림 중앙이나 하단이 아니라
    그림 높이 기반 baseline(`h * 0.85`)과 caret ascent(`caret_h * 0.8`)로 계산한다.
- `tests/issue_1452_saved_caret.rs`
  - 문단부호 표시가 그림 bbox 기준 커서 y를 임의로 바꾸지 않는지 검증한다.
  - 첫 번째 그림 앞 커서가 두 번째 그림 앞/뒤 커서보다 위쪽 그림 기준선에 놓이는지 검증한다.

## 5. 검증 결과

- `cargo fmt --check` 통과.
- `cargo test --test issue_1452_saved_caret -- --nocapture` 통과.
- `wasm-pack build --target web --out-dir pkg --no-pack` 통과.
  - `wasm-pack build --target web --out-dir pkg`는 `pkg/package.json`의 `repository` 객체 파싱 단계에서 실패했지만,
    WASM/JS 산출물은 `--no-pack`으로 정상 갱신했다.
- 브라우저 검증:
  - 저장 위치(두 번째 그림 뒤): `.caret` style `left=700.45px`, `top=331.4px`, `height=12px`.
  - ArrowLeft 1회(두 번째 그림 앞): `.caret` style `left=133.55px`, `top=331.4px`, `height=12px`.
  - ArrowLeft 2회(첫 번째 그림 앞): `.caret` style `left=133.55px`, `top=219.2px`, `height=13.3px`.
  - 첫 번째/두 번째 그림 앞 커서가 각각 다른 그림 bbox 기준선에 놓이는 것을 확인했다.
