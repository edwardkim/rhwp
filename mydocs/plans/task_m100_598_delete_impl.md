# Task #598 2차 구현 계획서 — 본문 각주 삭제 API/UI

## 이슈 정합

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **마일스톤**: M100 — v1.0.0 조판 엔진 체계화
- **작업 브랜치**: `local/task598`
- **기준 커밋**: `upstream/devel` `9b49063`
- **선행 완료 범위**:
  - 1차: 본문 각주 마커 hit test
  - 1차: 각주 마커 앞/뒤 cursor unit 및 caret rect
  - 1차: rhwp-studio 본문 마커 클릭 → 각주 편집 모드 진입

## 구현 범위

이번 2차 작업은 본문 각주 컨트롤 자체를 삭제하는 API와 UI 연결이다.

대상:

- 본문 `Control::Footnote` 삭제
- Delete/Backspace 키 처리
- Undo/Redo 연결
- 삭제 후 각주 번호 재계산, 문단 리플로우, 페이지네이션 갱신

비대상:

- 표 셀/글상자 내부 각주 삭제
- 미주(`Endnote`) 삭제
- 각주 영역 내부 텍스트 편집 Undo 정밀화

## 현재 구조 요약

### Rust

- 각주 삽입은 `insert_footnote_native()` 가 담당한다.
- 각주 내용 편집은 `footnote_ops.rs` 의 `insert_text_in_footnote_native()` / `delete_text_in_footnote_native()` 가 담당한다.
- 본문 `Control::Footnote` 자체를 삭제하는 API는 아직 없다.
- 그림/도형/표 삭제 API는 control 제거 후 `char_offsets`, `char_count`, `line_segs`, `pagination` 을 갱신하는 패턴을 갖고 있다.

### rhwp-studio

- 본문 Backspace/Delete 는 `input-handler-text.ts` 에서 `DeleteTextCommand` 로 라우팅된다.
- 현재는 footnote marker 위치에서도 일반 텍스트 삭제로 처리되어 실제 `Control::Footnote` 를 제거하지 못한다.
- 복잡한 구조 변경은 `SnapshotCommand` 경로로 Undo/Redo 를 처리할 수 있다.

## API 설계

### 신규 WASM API

```rust
#[wasm_bindgen(js_name = deleteFootnote)]
pub fn delete_footnote(
    &mut self,
    section_idx: u32,
    para_idx: u32,
    control_idx: u32,
) -> Result<String, JsValue>
```

반환 JSON:

```json
{
  "ok": true,
  "sectionIndex": 0,
  "paragraphIndex": 3,
  "charOffset": 7,
  "deletedNumber": 1
}
```

오류:

- 구역/문단/control index 범위 초과
- 대상 control 이 `Control::Footnote` 가 아닌 경우
- body source 가 아닌 위치에서 호출한 경우는 이번 UI에서 호출하지 않음

### WasmBridge 래퍼

```ts
deleteFootnote(
  sectionIndex: number,
  paragraphIndex: number,
  controlIndex: number,
): {
  ok: boolean;
  sectionIndex: number;
  paragraphIndex: number;
  charOffset: number;
  deletedNumber: number;
}
```

### 삭제 대상 조회

UI에서 Backspace/Delete 시 다음 로직으로 삭제 대상을 찾는다.

- Backspace:
  - 커서가 각주 마커 뒤 위치(`marker_pos + 1`)면 해당 각주 삭제
- Delete:
  - 커서가 각주 마커 앞 위치(`marker_pos`)면 해당 각주 삭제

이를 위해 Rust 쪽에 현재 커서 주변 각주 조회 API를 추가한다.

```rust
#[wasm_bindgen(js_name = getFootnoteAtCursor)]
pub fn get_footnote_at_cursor(
    &self,
    section_idx: u32,
    para_idx: u32,
    char_offset: u32,
    direction: &str,
) -> Result<String, JsValue>
```

반환 JSON:

```json
{
  "hit": true,
  "sectionIndex": 0,
  "paragraphIndex": 3,
  "controlIndex": 0,
  "charOffset": 7,
  "footnoteNumber": 1
}
```

miss:

```json
{ "hit": false }
```

`direction` 의미:

| direction | 조건 |
|-----------|------|
| `backward` | `char_offset == marker_pos + 1` |
| `forward` | `char_offset == marker_pos` |

## 구현 상세

### 1. Rust 삭제 API

대상 파일:

- `src/document_core/commands/footnote_ops.rs`
- `src/document_core/commands/object_ops.rs` 또는 공용 helper 후보
- `src/model/event.rs`
- `src/wasm_api.rs`

구현:

1. `find_control_text_positions(para)` 로 삭제 대상 각주의 marker position 을 찾는다.
2. 대상 control 이 `Control::Footnote` 인지 검증한다.
3. 기존 그림/도형 삭제와 같은 방식으로 UTF-16 control gap 8 code unit 을 제거한다.
4. `controls` 와 `ctrl_data_records` 에서 control 을 제거한다.
5. `char_count` 를 8 감소시킨다.
6. 남은 각주 번호를 문서 순서대로 재계산한다.
7. 본문 문단 `line_segs` 를 리플로우한다.
8. `raw_stream = None`, `recompose_section()`, `paginate_if_needed()`, `invalidate_page_tree_cache()` 를 수행한다.
9. `DocumentEvent::FootnoteDeleted` 를 추가하거나, 이벤트 추가가 과하면 기존 이벤트 로그에는 구조 변경 이벤트를 최소 기록한다. 기본 계획은 `FootnoteDeleted` 추가다.

### 2. Rust 조회 API

대상 파일:

- `src/document_core/queries/cursor_rect.rs` 또는 별도 query 모듈
- `src/wasm_api.rs`

구현:

- `get_footnote_at_cursor_native(section, para, char_offset, direction)` 추가
- 현재 문단의 `Control::Footnote` 와 control position 을 대조해 hit 여부 반환
- Backspace/Delete UI가 control index 를 직접 추론하지 않도록 Rust가 source of truth 역할을 담당

### 3. rhwp-studio UI 연결

대상 파일:

- `rhwp-studio/src/core/types.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/engine/input-handler-text.ts`
- 필요 시 `rhwp-studio/src/engine/command.ts`

구현:

1. `WasmBridge.getFootnoteAtCursor()` / `deleteFootnote()` 추가
2. `handleBackspace()` 의 본문 분기에서 일반 텍스트 삭제 전에 `getFootnoteAtCursor(..., 'backward')` 검사
3. `handleDelete()` 의 본문 분기에서 일반 텍스트 삭제 전에 `getFootnoteAtCursor(..., 'forward')` 검사
4. hit 시 `executeOperation({ kind: 'snapshot', operationType: 'deleteFootnote', operation })` 로 삭제
5. 삭제 후 커서는 반환된 `charOffset` 에 둔다

Undo/Redo:

- 각주 control 삭제는 내부 문단, control gap, 번호 재계산, 페이지네이션이 함께 바뀌므로 정밀 텍스트 command 보다 `SnapshotCommand` 가 안전하다.

### 4. 검증

Rust:

- `samples/footnote-01.hwp` 문단 0.3 offset 7/8 기준 조회 API 검증
- 삭제 API 호출 후:
  - 해당 문단의 footnote control 제거
  - `get_control_text_positions(0, 3)` 에서 해당 control 사라짐
  - 두 번째 각주가 번호 `1)` 로 재계산되는지 확인
  - `hit_test_body_footnote_marker_native()` 가 첫 마커 좌표에서 miss 또는 다른 결과를 반환하는지 확인

rhwp-studio:

- 본문 마커 앞에서 Delete → 각주 삭제
- 본문 마커 뒤에서 Backspace → 각주 삭제
- Undo → 각주 복원
- Redo → 각주 재삭제

명령:

```bash
cargo test --test issue_598_footnote_marker_nav
cargo test issue_598_delete_footnote
cargo build
docker-compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
git diff --check
```

## 구현 단계

### Stage 4-1 — Rust 삭제/조회 API

- `get_footnote_at_cursor_native()` 추가
- `delete_footnote_native()` 추가
- 각주 번호 재계산 및 리플로우 처리
- WASM export 추가
- Rust 단위/통합 테스트 추가

완료 기준:

- 삭제 전 조회 API hit
- 삭제 후 control 제거 및 번호 재계산 확인
- `cargo test issue_598_delete_footnote` 통과

### Stage 4-2 — rhwp-studio 키 처리 + Undo

- `WasmBridge` 래퍼 추가
- Backspace/Delete 에서 각주 삭제 우선 처리
- `SnapshotCommand` 로 Undo/Redo 연결
- TypeScript 빌드 통과

완료 기준:

- 본문 마커 앞 Delete 삭제
- 본문 마커 뒤 Backspace 삭제
- Undo/Redo 가능

### Stage 4-3 — WASM/browser 검증

- Docker WASM 빌드
- rhwp-studio 빌드
- Vite dev server 수동 확인 요청

완료 기준:

- 브라우저에서 `samples/footnote-01.hwp` 로 각주 삭제/복원 확인

### Stage 4-4 — 최종 보고

- 자동/수동 검증 결과 정리
- 오늘 할일 갱신
- 최종 보고서 또는 2차 완료보고서 작성

## 승인 요청

위 계획대로 Stage 4-1부터 진행한다.
