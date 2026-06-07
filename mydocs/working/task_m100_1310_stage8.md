# Task #1310 Stage 8 - 미주 가상 문단 수평 커서 이동 보정

## 1. 발견 현상

`samples/3-09월_교육_통합_2022.hwp` 의 미주가 시작되는 9쪽에서 `문1)` 쪽에 커서를
둔 뒤 오른쪽 방향키를 누르면 커서가 정상적으로 전진하지 않았다.

Stage 7 수정 후 콘솔 panic 은 더 이상 발생하지 않았지만, 커서 위치는 유지되는 상태였다.

## 2. 원인

TypeScript 커서 네비게이션은 본문 위치에서 `navigateNextEditable(..., context=[])` 를 호출한다.
하지만 Rust 쪽 `navigate_next_editable()` 은 빈 context 를 실제 본문 문단 배열
(`section.paragraphs`) 로만 해석했다.

미주 영역은 pagination 단계에서 `endnote_paragraphs` 로 생성되는 렌더 문단이다. 따라서
`para=602` 처럼 본문 문단 수 451을 넘지만 렌더 문단으로는 유효한 인덱스가 들어오면,
수평 이동 탐색은 현재 문단을 찾지 못하고 조용히 `Boundary` 로 빠졌다. TypeScript 쪽은
`Boundary` 결과를 받으면 이전 커서 위치를 복원하므로, 사용자 입장에서는 오른쪽 방향키가
동작하지 않는 것처럼 보였다.

## 3. 수정

`doc_tree_nav.rs` 에 본문/컨테이너 문단 해석을 분리하는 helper 를 추가했다.

- context 가 비어 있으면 렌더 문단 인덱스 공간을 사용한다.
  - 본문 문단
  - pagination 미주 가상 문단
- context 가 있으면 기존처럼 표/글상자/캡션 내부 문단 배열을 사용한다.
- `navigate_to_para_start/end`, 표/글상자 탈출 경로도 같은 helper 를 사용하도록 정리했다.

수정 파일:

- `src/document_core/queries/doc_tree_nav.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

## 4. 회귀 테스트

추가 테스트:

```text
issue_1139_endnote_virtual_paragraph_right_arrow_moves_within_text
```

테스트 내용:

- `samples/3-09월_교육_통합_2022.hwp`
- 미주 가상 문단 `para=602`
- `navigateNextEditable(0, 602, 0, +1, [])`
- 기존이면 `Boundary`
- 수정 후 `type=text`, 같은 렌더 문단에서 `charOffset > 0`

## 5. 검증

통과:

```bash
cargo fmt --all -- --check
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_virtual_paragraph_right_arrow_moves_within_text -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_virtual_paragraph_vertical_move_does_not_panic -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate endnote_virtual -- --nocapture
cargo check
cargo check --target wasm32-unknown-unknown --lib
```

