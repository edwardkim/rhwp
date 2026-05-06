# Task #598 Stage 3-2 완료보고서 — Cursor navigation / rect 보정

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `local/task598`
- **기준 커밋**: `upstream/devel` `9b49063`
- **단계 범위**: 본문 각주 마커를 좌우 커서 이동 단위로 취급하고, 마커 앞/뒤 caret rect 를 반환하도록 보정

본 단계에서는 Rust 쪽 커서 이동과 caret rect 만 처리했다. rhwp-studio `WasmBridge`/마우스 이벤트 연결은 Stage 3-3 범위로 남겨두었다.

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/document_core/helpers.rs` | `navigable_text_len()` 이 `Footnote`/`Endnote` 위치 뒤 offset 을 허용하도록 확장 |
| `src/document_core/queries/doc_tree_nav.rs` | `classify_navigable()` 에서 `Footnote`/`Endnote` 를 1칸 inline unit 으로 분류 |
| `src/document_core/queries/cursor_rect.rs` | `FootnoteMarker` bbox 기반으로 마커 왼쪽/오른쪽 caret rect 반환 |
| `src/model/paragraph.rs` | `char_offsets` 없는 폴백 경로에서도 `Footnote`/`Endnote` 를 1칸 inline control 로 배치 |
| `tests/issue_598_footnote_marker_nav.rs` | `samples/footnote-01.hwp` 기반 회귀 테스트 추가 |

## 구현 내용

### 1. 탐색 가능한 문단 길이 보정

`navigable_text_len()` 의 inline control 대상에 `Control::Footnote(_)` 와 `Control::Endnote(_)` 를 추가했다.

이로써 각주 마커가 문단 끝에 있는 경우에도 `marker_pos + 1` offset 이 문단 내 유효한 커서 위치가 된다.

### 2. 좌우 커서 이동 단위 보정

`doc_tree_nav.rs` 의 `classify_navigable()` 에 `Footnote`/`Endnote` 를 `Some(false)` 로 추가했다.

따라서 기존 Shape/Picture/Equation 과 같은 방식으로 동작한다.

```text
offset == marker_pos      → 마커 앞
ArrowRight / next         → marker_pos + 1
offset == marker_pos + 1  → 마커 뒤
ArrowLeft / previous      → marker_pos
```

### 3. caret rect 보정

`get_cursor_rect_native()` 에서 현재 문단의 note marker control position 을 미리 계산하고, 렌더 트리의 `RenderNodeType::FootnoteMarker` 와 대조한다.

반환 기준:

| offset | 반환 좌표 |
|--------|-----------|
| `marker_pos` | `FootnoteMarker` bbox 왼쪽 |
| `marker_pos + 1` | `FootnoteMarker` bbox 오른쪽 |

이 처리는 TextRun 일반 탐색보다 먼저 수행되므로, 마커 뒤 offset 이 후속 TextRun 안쪽 위치로 흡수되지 않는다.

### 4. 샘플 회귀 테스트 추가

`tests/issue_598_footnote_marker_nav.rs` 를 추가했다.

검증 항목:

- `samples/footnote-01.hwp` 문단 0.3의 첫 번째 본문 각주 마커
  - `hit_test_body_footnote_marker_native()` 가 `paragraphIndex=3`, `controlIndex=0`, `footnoteIndex=0` 반환
  - offset `7`/`8` 의 caret x 좌표가 마커 왼쪽/오른쪽 순서로 반환
  - `navigateNextEditable(7, +1)` → `8`, `navigateNextEditable(8, -1)` → `7`
- 문단 0.7의 두 번째 본문 각주 마커
  - `hit_test_body_footnote_marker_native()` 가 `paragraphIndex=7`, `controlIndex=0`, `footnoteIndex=1` 반환
  - offset `6`/`7` 의 caret x 좌표와 좌우 이동 검증
- synthetic 문단 단위 테스트
  - 문단 끝 `Footnote` 가 `navigable_text_len()` 을 `text_len + 1` 로 확장하는지 검증

## 검증

실행 결과:

```bash
cargo test --test issue_598_footnote_marker_nav
cargo test navigable_text_len_counts_trailing_footnote_marker
cargo build
git diff --check
```

결과:

- `cargo test --test issue_598_footnote_marker_nav`: 2 passed
- `cargo test navigable_text_len_counts_trailing_footnote_marker`: 1 passed
- `cargo build`: 통과
- `git diff --check`: 통과

참고:

- `cargo test navigable_text_len_counts_trailing_footnote_marker` 실행 중 기존 테스트 코드의 warning 이 함께 출력됐지만, 이번 변경과 무관한 기존 warning 이며 테스트는 통과했다.
- rhwp-studio 웹 서버 검증은 아직 필요하지 않다. 다음 Stage 3-3에서 `WasmBridge` 와 마우스 입력 처리를 연결한 뒤 웹 서버 실행 검증을 요청한다.

## 남은 작업

다음 승인을 받은 뒤 Stage 3-3에서 진행한다.

1. `rhwp-studio/src/core/wasm-bridge.ts` 에 `hitTestBodyFootnoteMarker()` 래퍼 추가
2. `rhwp-studio/src/engine/input-handler-mouse.ts` 에 본문 각주 마커 클릭 처리 연결
3. 마커 클릭 시 기존 각주 편집 모드 진입 경로와 연결
4. 웹 서버 실행 후 실제 클릭/커서 동작 검증
