# Task #598 Stage 4-4 완료보고서 — 각주 앞 Backspace anchor/Undo 보정

## 작업 범위

- 본문 각주 마커 바로 앞 위치에서 `Backspace` 로 일반 텍스트를 삭제할 때 각주 마커가 줄 끝으로 이동하는 문제 보정
- 동일 상황에서 `Undo` 시 텍스트만 복원되고 각주 마커 위치가 복원되지 않는 문제 보정
- Rust 단위 테스트와 rhwp-studio E2E 테스트에 회귀 케이스 추가

## 원인

`Paragraph::delete_text_at()` 이 삭제 구간의 UTF-16 길이를 다음 `char_offset` 값으로 계산하고 있었다. 삭제 문자 뒤에 각주 컨트롤 슬롯이 있으면 다음 `char_offset` 에 컨트롤 gap 이 포함되어 일반 텍스트 삭제가 컨트롤 gap 까지 함께 당기는 결과가 됐다.

또한 `Paragraph::insert_text_at()` 은 각주 컨트롤과 같은 위치에 텍스트를 삽입할 때 해당 위치의 기존 `char_offsets` 값을 그대로 사용했다. 이 값은 컨트롤 뒤쪽 UTF-16 위치를 가리킬 수 있어 Undo 성격의 삽입이 각주 마커 뒤로 들어가며 anchor 복원이 깨졌다.

## 구현 내용

- `delete_text_at()` 의 `utf16_delta` 를 삭제 대상 문자들의 실제 UTF-16 길이 합으로 계산하도록 변경했다.
- `insert_text_at()` 에서 삽입 위치가 inline control position 과 같으면 이전 문자 끝 위치를 UTF-16 삽입 지점으로 사용하도록 보정했다.
- `tests/issue_598_footnote_marker_nav.rs` 에 `issue_598_backspace_before_marker_keeps_marker_anchor_and_undo_restores_it` 테스트를 추가했다.
- `rhwp-studio/e2e/footnote-delete-confirm.test.mjs` 에 각주 앞 `Backspace` 일반 텍스트 삭제 및 Undo anchor 복원 검증을 추가했다.

## 검증

실행 명령:

```bash
cargo test --test issue_598_footnote_marker_nav
docker-compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/footnote-delete-confirm.test.mjs --mode=headless
```

결과:

- `cargo test --test issue_598_footnote_marker_nav`: 4 passed
- WASM 빌드: 통과
- rhwp-studio build: 통과
- `footnote-delete-confirm.test.mjs`: 통과

## 수동 확인 요청 항목

작업지시자가 `http://localhost:7700/` 에서 다음 흐름을 재확인할 수 있다.

1. `samples/footnote-01.hwp` 를 연다.
2. 첫 번째 본문 각주 마커 바로 앞, 즉 `액체|1)와` 위치에 caret 을 둔다.
3. `Backspace` 를 누른다.
4. 각주 삭제 확인창이 뜨지 않고, 직전 일반 텍스트만 삭제되는지 확인한다.
5. 각주 마커가 줄 끝으로 이동하지 않고 남은 텍스트와 다음 텍스트 사이에 유지되는지 확인한다.
6. `Cmd+Z` 를 눌러 텍스트와 각주 마커 위치가 함께 원래대로 복원되는지 확인한다.
