# Task #1310 Stage 7 - 미주 가상 문단 세로 커서 이동 panic 방어

## 1. 발견 현상

수정본 WASM 빌드 후 rhwp-studio 동작 테스트 중 `moveVertical` 에서 다음 panic 이 발생했다.

```text
panicked at src/document_core/queries/cursor_nav.rs:924:90:
index out of bounds: the len is 451 but the index is 602
```

문서 로드는 정상이며, 커서의 세로 이동 중 본문 문단 수를 초과하는 문단 인덱스가
본문 경계 처리 경로로 들어와 panic 했다.

## 2. 원인

`get_render_paragraph_ref()` 는 이미 다음 인덱스 공간을 허용한다.

- 실제 본문 문단
- pagination 단계에서 생성한 미주 가상 문단 (`endnote_paragraphs`)

하지만 `handle_body_boundary()` 는 본문 문단 수(`section.paragraphs.len()`)만 기준으로
문단 경계를 판정하고, 문서 끝 처리에서 `section.paragraphs[para]` 를 직접 접근했다.

따라서 `para=602` 처럼 본문 문단 수 451을 넘지만 렌더 문단으로는 유효한 미주 가상
문단이 들어오면, 문서 끝 폴백에서 직접 인덱싱으로 panic 했다.

## 3. 수정

`handle_body_boundary()` 를 렌더 문단 인덱스 공간 기준으로 수정했다.

- 렌더 문단 수 = 본문 문단 수 + 미주 가상 문단 수
- 현재 문단이 범위를 벗어난 경우 마지막 렌더 문단으로 clamp
- 구역 시작/끝 경계도 렌더 문단 수 기준으로 이동
- 본문 문단은 기존 `enter_paragraph()` 경로 유지
- 미주 가상 문단은 새 `enter_render_paragraph()` 경로로 진입
- 본문 문단 사이에서만 기존 column preferred-x 변환을 적용

수정 파일:

- `src/document_core/queries/cursor_nav.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

## 4. 회귀 테스트

추가 테스트:

```text
issue_1139_endnote_virtual_paragraph_vertical_move_does_not_panic
```

테스트 내용:

- `samples/3-09월_교육_통합_2022.hwp`
- 로그에서 확인된 미주 가상 문단 `para=602`
- `charOffset=u32::MAX`, `delta=+1` 로 문단 경계를 밟게 함
- 기존이면 `section.paragraphs[602]` 접근으로 panic
- 수정 후 JSON cursor result 반환

## 5. 검증

통과:

```bash
cargo fmt --all -- --check
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_virtual_paragraph_vertical_move_does_not_panic -- --nocapture
cargo check
cargo check --target wasm32-unknown-unknown --lib
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap -- --nocapture
docker compose --env-file .env.docker run --rm wasm
```

WASM 산출물:

- `pkg/rhwp_bg.wasm`
- `pkg/rhwp.js`
- `pkg/rhwp.d.ts`

