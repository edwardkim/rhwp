# Task M100 #1459 Stage 3

- 작성일: 2026-06-22
- 모드: 기여자 모드. PR 전 오늘할일 문서는 생성하거나 갱신하지 않는다.
- 이전 커밋: `5f9df1ad task 1459: TopAndBottom TAC 간격 중복 보정`

## 목표

비-TAC `TopAndBottom` 그림을 caret 문자 흐름에서 제외한다.

Stage 2 이후 위쪽 투명도 50 그림은 `treat_as_char=false`인 자리차지 그림으로 먼저 렌더된다. 이 그림은 문서 문자처럼 취급되지 않으므로 Home/ArrowUp/마우스 클릭 등으로 캐럿이 그림 앞/뒤 문자 위치에 들어가면 안 된다.

## 현재 관찰

- `helpers::is_logical_inline_control`과 cursor 관련 로컬 판정이 `Control::Picture(_)`를 무조건 인라인 문자처럼 센다.
- `find_logical_control_positions`, `navigable_text_len`, cursor rect, line navigation이 비-TAC 그림까지 논리 offset에 포함할 수 있다.
- 샘플 #1459의 caret 가능한 위치는 TAC 그림 앞/뒤 2곳이어야 하며, 비-TAC 그림의 bbox 기준 caret stop은 없어야 한다.

## 수정 방향

- 논리 인라인 컨트롤 판정에서 Picture/Shape/Table은 `treat_as_char=true`일 때만 caret 문자 흐름에 포함한다.
- 비-TAC 그림 클릭은 caret 위치로 바로 변환하지 않는다.
- #1459 샘플 테스트에 cursor rect/line navigation 조건을 추가한다.

## 검증 계획

- `cargo test --profile release-test --test issue_1459_topbottom_picture_reflow -- --nocapture`
- #1452 저장 커서/그림 이동 회귀 테스트
- cursor helper 관련 lib 테스트

## 구현 결과

- `treat_as_char=true`인 Shape/Table/Picture/Equation만 편집용 논리 문자 슬롯에 포함하도록 공통 판정을 정리했다.
- cursor rect, hit test, vertical navigation 후보 수집에서 비-TAC 그림 bbox를 caret 문자 후보로 쓰지 않도록 필터링했다.
- #1459 샘플에 비-TAC TopAndBottom 그림이 caret stop이 되지 않는 회귀 테스트를 추가했다.

## 검증 결과

- `cargo test --profile release-test --lib logical_positions -- --nocapture`: 통과.
- `cargo test --profile release-test --test issue_1459_topbottom_picture_reflow -- --nocapture`: 통과.
- `cargo test --profile release-test --test issue_1452_saved_caret -- --nocapture`: 통과.
- `cargo test --profile release-test --test issue_1139_inline_picture_duplicate issue_1293_equation_control_is_not_always_treat_as_char -- --nocapture`: 통과.
- `cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과.
- `cargo fmt --check`: 통과.
- `wasm-pack build --target web --out-dir pkg`: 통과 (`pkg/`는 gitignored 생성물).
