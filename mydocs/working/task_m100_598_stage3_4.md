# Task #598 Stage 3-4 완료보고서 — 1차 작업 검증 정리

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `local/task598`
- **기준 커밋**: `upstream/devel` `9b49063`
- **단계 범위**: 1차 작업인 본문 각주 마커 hit test + 커서 이동 정합 검증

본 단계에서는 Stage 3-1~3-3 구현 결과를 최종 확인했다. 이슈 #598의 두 번째 축인 삭제 API/UI는 아직 구현하지 않았다.

## 구현 요약

### Rust/WASM

- `ComposedParagraph.footnote_positions` 에 실제 `para.controls` 인덱스를 보존하도록 확장했다.
- `FootnoteMarkerNode.control_index` 가 배열 순번이 아니라 실제 각주 control index 를 가리키도록 보정했다.
- `hit_test_body_footnote_marker_native()` / `hitTestBodyFootnoteMarker()` 를 추가했다.
- `Control::Footnote` / `Control::Endnote` 를 본문 inline cursor unit 으로 취급했다.
- `get_cursor_rect_native()` 가 `FootnoteMarker` bbox 기준으로 마커 왼쪽/오른쪽 caret rect 를 반환하도록 보정했다.

### rhwp-studio

- `WasmBridge.hitTestBodyFootnoteMarker()` 래퍼를 추가했다.
- 마우스 클릭 처리에서 본문 각주 마커 hit 를 각주 영역 hit test 보다 먼저 검사하도록 연결했다.
- 본문 각주 마커 클릭 시 `enterFootnoteMode()` 로 각주 편집 모드에 진입하도록 했다.

## 검증 결과

### 자동 검증

실행 명령:

```bash
cargo test --test issue_598_footnote_marker_nav
cargo test navigable_text_len_counts_trailing_footnote_marker
cd rhwp-studio && npm run build
cargo build
git diff --check
```

결과:

- `cargo test --test issue_598_footnote_marker_nav`: 2 passed
- `cargo test navigable_text_len_counts_trailing_footnote_marker`: 1 passed
- `npm run build`: 통과
- `cargo build`: 통과
- `git diff --check`: 통과

참고:

- `cargo test navigable_text_len_counts_trailing_footnote_marker` 실행 시 기존 테스트 코드의 warning 이 함께 출력됐다. 이번 변경과 무관한 기존 warning 이며 테스트는 통과했다.
- `npm run build` 실행 시 Vite chunk size warning 이 출력됐다. 기존 번들 크기 경고이며 빌드는 성공했다.

### 수동 검증

작업지시자가 `http://localhost:7700/` 에서 rhwp-studio 를 실행해 다음 동작을 확인했다.

- `samples/footnote-01.hwp` 로드
- 1페이지 본문 각주 마커 클릭
- 하단 각주 영역으로 caret 이동
- 각주 편집 모드 진입

수동 확인 결과는 정상으로 보고받았다.

## 산출물

- 신규 테스트: `tests/issue_598_footnote_marker_nav.rs`
- Stage 3-1 보고서: `mydocs/working/task_m100_598_stage3_1.md`
- Stage 3-2 보고서: `mydocs/working/task_m100_598_stage3_2.md`
- Stage 3-3 보고서: `mydocs/working/task_m100_598_stage3_3.md`
- Stage 3-4 보고서: `mydocs/working/task_m100_598_stage3_4.md`

## 남은 작업

이슈 #598의 다음 작업은 삭제 API/UI 구현이다.

후속 구현 범위:

1. 본문 각주 마커 앞/뒤 위치에서 Delete/Backspace 로 각주 control 삭제
2. 삭제 후 각주 번호/페이지 각주 목록 리플로우
3. rhwp-studio Undo/Redo 명령 연결
4. 각주 삭제 UI 및 회귀 테스트 추가
