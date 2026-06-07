# Task #1310 Stage 9 - 미주 수식 커서 좌표 되감김 보정

## 1. 발견 현상

9쪽 미주 영역에서 커서가 움직이기 시작했지만, 수식이 포함된 미주 문단에서 오른쪽 방향키를
반복 입력하면 커서가 반복적으로 첫 위치로 돌아가는 현상이 있었다.

예시:

- `samples/3-09월_교육_통합_2022.hwp`
- 9쪽 미주 `문1)` 이후 수식 풀이 문단
- `pi=471`, `pi=474`, `pi=479`

Stage 8 이후 콘솔 오류와 `Boundary` 문제는 사라졌지만, `getCursorRect` 좌표가 수식 컨트롤
앞/뒤를 제대로 따라가지 못했다.

## 2. 원인

미주 가상 문단의 수식 `EquationNode` 는 원본 미주 위치를 `note_ref` 로 보존한다.
그런데 렌더 노드의 `para_index` 도 원본 note 위치로 대체하는 경로가 있었다.

커서 좌표 계산(`getCursorRect`)은 화면상의 렌더 문단 인덱스와 control index 로 inline 수식
bbox 를 찾는다. 따라서 `pi=471` 같은 렌더 문단에서 커서 좌표를 찾을 때, 수식 bbox 가 매칭되지
않고 빈 TextRun fallback 이 선택되어 x 좌표가 줄 시작으로 되돌아갔다.

## 3. 수정

수식 노드의 역할을 분리했다.

- `EquationNode.para_index/control_index`: 현재 화면에 렌더된 문단/컨트롤 위치
- `EquationNode.note_ref`: 원본 미주 내부 위치

이렇게 하면 커서/선택/inline bbox 조회는 렌더 문단 기준으로 동작하고, 미주 수식 속성 편집용
원본 위치 정보는 계속 보존된다.

수정 파일:

- `src/renderer/layout/paragraph_layout.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

## 4. 회귀 테스트

추가 테스트:

```text
issue_1139_endnote_equation_cursor_rects_do_not_rewind_to_line_start
```

테스트 내용:

- 9쪽 미주 수식 문단 `pi=471`, `pi=474`, `pi=479`
- offset 별 `getCursorRect` x 좌표가 오른쪽 이동 중 왼쪽 줄 시작으로 되감기지 않는지 확인
- adjacent 수식 경계는 `이전 수식 끝 == 다음 수식 시작` 이므로 같은 x 를 허용
- `navigateNextEditable(..., +1, [])` 가 `Boundary` 로 빠지지 않는지 확인

## 5. 검증

통과:

```bash
cargo fmt --all -- --check
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_equation_cursor_rects_do_not_rewind_to_line_start -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_virtual_paragraph_right_arrow_moves_within_text -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate endnote_virtual -- --nocapture
cargo check
cargo check --target wasm32-unknown-unknown --lib
```

