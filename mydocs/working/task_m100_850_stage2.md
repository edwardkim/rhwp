# Task #850 Stage 2 완료보고서

## 단계

Stage 2 — `cursor_rect.rs` hit-test 경로 보존 로직 정정

## 작업 내용

수정 파일:

- `src/document_core/queries/cursor_rect.rs`

신규 테스트:

- `tests/issue_850_answer_sheet_name_hit_test.rs`

## 원인 확인

`exam_social.hwp` 상단 답안지 영역의 렌더 트리를 확인한 결과, 외곽 상단 표 안의 TAC 중첩 표가 다음처럼 렌더된다.

- 외곽 상단 표: `Table pi=0 ci=4`
- 외곽 셀: `cellIndex=0`
- 내부 `성명` 표: `Table pi=3 ci=0`
- 내부 `성명` 빈 입력칸 TextRun: `CellContext { parent_para_index: 3, path: [(0, 1, 0)] }`

즉 내부 TAC 표의 `pi=3, ci=0`은 문서 루트 문단/컨트롤이 아니라, 외곽 표 셀 내부의 `cellParaIndex=3`, 내부 표 `controlIndex=0`이다. 그러나 기존 `hit_test_native()`는 이 로컬 메타를 문서 루트 기준처럼 그대로 사용해서 다음 잘못된 경로를 반환했다.

```json
{
  "parentParaIndex": 3,
  "controlIndex": 0,
  "cellPath": [
    { "controlIndex": 0, "cellIndex": 1, "cellParaIndex": 0 }
  ]
}
```

이 값이 Studio 입력 경로로 전달되면 `insertTextInCell(0, 3, 0, 1, 0, 0, text)`가 호출되어 루트 문단 3의 컨트롤 0을 찾다가 `컨트롤 인덱스 0 범위 초과`가 발생한다.

## 수정 방식

`hit_test_native()`의 렌더 트리 수집 단계에서 조상 표/셀 컨텍스트를 함께 전파하도록 보정했다.

추가된 보정:

1. `table_ctx_from_node()`
   - Table 노드가 셀 내부에 있고 `para_index/control_index`를 가진 경우, 이를 루트 메타가 아니라 `현재 셀의 cellParaIndex + 내부 표 controlIndex`로 해석한다.
   - 예: 외곽 `[(4, 0, 0)]` + 내부 `pi=3, ci=0` → `[(4, 0, 3), (0, 0, 0)]`
2. `cell_ctx_for_table_cell()`
   - 현재 TableCell 진입 시 해당 표의 `cellIndex`, `cellParaIndex`, `textDirection`을 반영한 `CellContext`를 만든다.
3. `effective_cell_context()`
   - TextRun 자체의 `cell_context`가 내부 표 로컬 경로만 가진 경우, 조상 traversal context가 더 깊으면 traversal context를 우선 사용한다.
   - 기존처럼 TextRun 경로가 더 깊은 정상 중첩 표 케이스는 TextRun 경로를 유지한다.

이 방식은 Studio TypeScript 입력 경로를 수정하지 않고, Rust hit-test 반환값을 문서 구조에 맞게 바로잡는다.

## 검증 결과

### #850 신규 회귀 테스트

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

검증된 기대값:

- `exam_social.hwp`: `parentParaIndex=0`, `controlIndex=4`, `cellPath=[(4,0,3),(0,1,0)]`
- `exam_science.hwp`: `parentParaIndex=0`, `controlIndex=6`, `cellPath=[(6,0,3),(0,1,0)]`
- 두 문서 모두 `insert_text_in_cell_by_path()` 삽입 성공

### #717 기존 회귀 테스트

명령:

```bash
cargo test --test issue_717_table_cell_hit_test -- --nocapture
```

결과:

```text
running 3 tests
test issue_717_exam_social_view_table_empty_area_stays_in_clicked_table ... ok
test issue_717_exam_social_title_empty_area_stays_in_clicked_table ... ok
test issue_717_exam_social_nested_header_empty_area_returns_editable_path ... ok

test result: ok. 3 passed; 0 failed
```

## 판정

Stage 2 목표 달성.

- #850 신규 테스트 GREEN
- #717 기존 테스트 GREEN
- `rhwp-studio` TypeScript 입력 경로 수정 없이 Rust hit-test 반환값 정정으로 해결

## 남은 확인

Stage 3에서 Studio 입력 경로 기준 검증을 진행한다.

확인 항목:

- 반환된 `cellPath.length == 2`가 Studio의 `insertTextInCellByPath` 분기를 타는지 확인
- 관련 WASM API 테스트 범위 확인
- TypeScript 수정이 불필요하다는 근거 정리

## 승인 요청

Stage 3 진행 승인 요청.

