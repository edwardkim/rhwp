# Task M100 #1470 구현계획서

## 1. 분석 순서

1. 한컴 도움말 기준 동작을 스타일/캡션/표 생성 옵션으로 나누어 정리한다.
2. 스타일 적용 경로를 확인한다.
   - `src/document_core/commands/formatting.rs`
   - `src/wasm_api.rs`
   - `rhwp-studio/src/engine/input-handler.ts`
   - `rhwp-studio/src/ui/style-edit-dialog.ts`
3. 표 생성 옵션 경로를 확인한다.
   - `rhwp-studio/src/ui/table-create-dialog.ts`
   - `rhwp-studio/src/command/commands/table.ts`
   - `rhwp-studio/src/core/wasm-bridge.ts`
   - `src/wasm_api.rs`
   - `src/document_core/commands/object_ops.rs`
4. 표 캡션 속성 경로를 확인한다.
   - `rhwp-studio/src/ui/table-cell-props-dialog.ts`
   - `src/document_core/commands/table_ops.rs`
   - serializer/parser의 AutoNumber 처리
5. 기존 회귀 테스트를 확인한다.
   - `tests/issue_1172_para_margin_roundtrip.rs`
   - `src/wasm_api/tests.rs`의 표/캡션/붙여넣기 테스트

## 2. 스타일 구현 방향

- 스타일 적용 후 `para_shape_id`와 `char_shapes`가 바뀌면 기존 `LineSeg`를 버리고 현재 `DocInfo` 기준 스타일 resolver로 reflow한다.
- 본문 문단과 셀 문단 모두 처리한다.
- 셀 문단은 표 dirty 플래그를 세워 셀 높이 재측정이 일어나도록 한다.
- 일반 스타일은 스타일 정의의 `para_shape_id`를 우선 적용한다.
- 번호/개요 문단은 기존 번호 문맥 보존 로직을 유지한다.
- 스타일 편집 API는 해당 스타일을 사용하는 본문 문단과 표 셀 문단 목록을 먼저 수집한 뒤 shape ID 갱신과 reflow를 수행한다.

## 3. 스타일 UI/API 방향

- `createStyle` JSON에 optional `baseParaShapeId`, `baseCharShapeId`를 허용한다.
- Studio의 스타일 추가 대화상자는 현재 커서의 문단/글자 속성을 초기값으로 사용한다.
- 추가된 스타일은 현재 커서 문단의 shape ID를 기반으로 생성한다.
- 블록 선택 상태에서 스타일 적용 시 기존 문단 서식 타깃 수집 로직을 재사용해 여러 문단에 적용한다.
- Undo/Redo는 다중 스타일 적용의 구조 변경을 안전하게 묶기 위해 snapshot command를 사용한다.

## 4. 표 생성 옵션 구현 방향

- `TableCreateDialog.onApply`에 optional 옵션 객체를 추가한다.
- 상세 대화상자에서 다음 옵션을 전달한다.
  - `treatAsChar`
  - 직접 지정 너비: 전체 너비를 열 수만큼 분배한 `colWidths`
  - 직접 지정 높이: 행 수만큼 반복한 `rowHeights`
- `WasmBridge.createTableEx` 래퍼를 추가한다.
- Rust `createTableEx`는 `rowHeights` JSON 배열을 파싱한다.
- native `create_table_ex_native`는 기존 호출 호환성을 유지하면서 optional row heights를 받는다.
- 그리드 피커의 단순 선택은 기존 `createTable` 경로를 유지해 동작 변화를 줄인다.

## 5. 표 캡션 구현 방향

- 표 캡션 생성 시 literal `"표 N "` 텍스트로 치환하지 않는다.
- 캡션 문단은 AutoNumber inline 컨트롤 모델을 유지한다.
  - `text = "  "`
  - `char_count = 10`
  - `control_mask` AutoNumber bit 설정
  - `char_offsets = [0, 8]`
  - `controls = [AutoNumber(Table)]`
- `Caption` 스타일이 있으면 style/shape ID를 캡션 문단에 적용하고, 없으면 기존 기본값을 사용한다.
- `hasCaption=false`는 기존 캡션 삭제와 attr bit 29 해제로 처리한다.
- 캡션 생성 후 `assign_auto_numbers`를 실행해 번호는 배정하되 컨트롤은 보존한다.

## 6. 테스트 계획

- 스타일 테스트
  - 왼쪽 여백 15pt에 해당하는 raw 3000이 6000으로 변하지 않는지 확인
  - 스타일 줄간격 변경 후 `LineSeg.line_spacing`이 갱신되는지 확인
- 표 생성 테스트
  - `createTableEx`가 `treatAsChar`, 열 너비, 행 높이를 반영하는지 확인
- 표 캡션 테스트
  - 캡션 생성 후 AutoNumber 컨트롤이 유지되는지 확인
  - `hasCaption=false`가 캡션을 삭제하고 attr bit를 해제하는지 확인
- 주변 회귀 테스트
  - `issue_1172_para_margin_roundtrip`
  - `test_paste_picture_into_*`

## 7. 위험 관리

- 스타일 적용의 ParaShape 결정 로직은 번호/개요 문단에 영향을 줄 수 있으므로 번호 문맥 보존 분기는 유지한다.
- `LineSeg` 재계산은 페이지네이션을 바꾸므로 focused 테스트 외에도 기존 문단/표 회귀 테스트를 함께 확인한다.
- 표 캡션의 AutoNumber 모델 변경은 serializer/parser 계약에 민감하므로 literal 치환 제거 후 라운드트립 관련 테스트를 확인한다.
- Studio snapshot 기반 다중 스타일 적용은 Undo 메모리 사용량이 늘 수 있으나, 우선 정확성을 우선한다.

## 8. 산출물 계획

- 수행계획서: `mydocs/plans/task_m100_1470.md`
- 구현계획서: `mydocs/plans/task_m100_1470_impl.md`
- 단계 보고서: 승인 후 구현을 진행한 뒤 `mydocs/working/task_m100_1470_stage1.md` 작성
- 최종 보고서: 작업 완료 후 `mydocs/report/task_m100_1470_report.md` 작성
