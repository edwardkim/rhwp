# Task #850 Stage 3 완료보고서

## 단계

Stage 3 — Studio 입력 경로 기준 검증

## 확인 대상

Studio 입력 경로:

- `rhwp-studio/src/engine/command.ts`
- `rhwp-studio/src/engine/input-handler-text.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`

WASM path 기반 API:

- `src/wasm_api.rs`
- `src/wasm_api/tests.rs`

## Studio 입력 분기 확인

Studio 텍스트 입력 경로는 중첩 표 여부를 다음 조건으로 판단한다.

```ts
(pos.cellPath?.length ?? 0) > 1
```

해당 조건이 참이면 일반 셀 입력 API가 아니라 path 기반 API를 호출한다.

```ts
wasm.insertTextInCellByPath(
  pos.sectionIndex,
  pos.parentParaIndex!,
  JSON.stringify(pos.cellPath),
  pos.charOffset,
  text,
);
```

Stage 2 수정 후 #850 대상 hit-test 결과는 다음 조건을 만족한다.

- `exam_social.hwp`: `parentParaIndex=0`, `cellPath=[(4,0,3),(0,1,0)]`
- `exam_science.hwp`: `parentParaIndex=0`, `cellPath=[(6,0,3),(0,1,0)]`
- 두 문서 모두 `cellPath.length == 2`

따라서 Studio는 기존 코드 그대로 `insertTextInCellByPath` 분기를 탄다.

## WASM API 확인

`src/wasm_api.rs`의 `insertTextInCellByPath`는 JSON `cellPath`를 `DocumentCore::parse_cell_path()`로 파싱한 뒤 `insert_text_in_cell_by_path()`를 호출한다.

```rust
let path = DocumentCore::parse_cell_path(path_json)?;
self.insert_text_in_cell_by_path(
    section_idx as usize,
    parent_para_idx as usize,
    &path,
    char_offset as usize,
    text,
)
```

#850 신규 테스트는 이 Rust path 기반 삽입 경로를 직접 검증한다.

- hit-test 결과에서 `path_tuples()` 추출
- `insert_text_in_cell_by_path(0, 0, &path, char_offset, "홍")`
- `get_text_in_cell_by_path(0, 0, &path, char_offset, 1)`로 `"홍"` 확인

## 실행 결과

### #850 회귀 테스트

명령:

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

결과:

```text
running 2 tests
test issue_850_exam_science_answer_sheet_name_cell_keeps_outer_path ... ok
test issue_850_exam_social_answer_sheet_name_cell_keeps_outer_path ... ok

test result: ok. 2 passed; 0 failed
```

### 기존 중첩 표 path API 테스트

명령:

```bash
cargo test --lib test_task105_nested_table_path_api -- --nocapture
```

결과:

```text
test wasm_api::tests::test_task105_nested_table_path_api ... ok

test result: ok. 1 passed; 0 failed
```

## TypeScript 수정 여부

수정 불필요.

이유:

- Studio는 이미 `cellPath.length > 1`이면 path 기반 API를 사용한다.
- #850 수정 후 hit-test 반환값이 `cellPath.length == 2`를 만족한다.
- Rust path 기반 삽입 API가 #850 신규 테스트에서 실제 삽입/읽기까지 통과했다.

이번 결함은 Studio 입력 라우터 문제가 아니라 Rust `hit_test_native()`가 중첩 표 경로를 잘못 반환한 문제다. 따라서 Studio TypeScript를 우회 수정하지 않는 것이 맞다.

## 판정

Stage 3 목표 달성.

- Studio 기존 입력 분기와 Stage 2 반환값 정합 확인
- WASM path 기반 API 정상 확인
- TypeScript 변경 없음

## 다음 단계

Stage 4에서 회귀 검증을 진행한다.

예정 검증:

- #850 신규 테스트
- #717 기존 테스트
- 관련 WASM API 테스트
- 전체 `cargo test`

## 승인 요청

Stage 4 진행 승인 요청.

