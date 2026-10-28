# Task M100-493 최종 보고서

- 이슈: #493
- 브랜치: `local/task_m100_493`
- 기준 브랜치: `upstream/devel`
- 작성일: 2026-06-19

## 1. 작업 요약

한컴오피스 표 셀 속성 중 `셀 보호`, `필드 이름`, `양식 모드에서 편집 가능`을 보존하고,
Studio에서 실제 셀 보호 UX가 동작하도록 구현했다.

주요 변경:

- HWP/HWPX 셀 속성 파싱/직렬화에 `protect`, `editable`, `name` 보존 경로 추가
- `getCellProperties`, `setCellProperties`, `getFieldList` API에 셀 필드 속성 노출
- Studio 표/셀 속성 대화상자에 필드 이름과 양식 모드 편집 가능 UI 반영
- 보호 셀 hover 시 진입 불가 표시와 `not-allowed` 커서 표시
- 보호 셀 클릭 시 텍스트 커서 진입을 막고 셀 선택 상태로 전환
- 보호 셀 선택 상태에서 일반 문자 입력 차단
- 보호 셀 선택 상태에서 `셀 속성...` 진입 가능
- 표 외곽선 클릭 시 표 객체 선택 지원
- 표 객체 선택 상태에서 `표 속성...` 진입 가능
- 표/셀 속성 대화상자 탭 전환 시 모달 크기 고정

## 2. 커밋

- `4f8a81d8 task 493: 셀 보호 속성 보존`
- `e7c84130 task 493: 셀 보호 입력 차단 UX 구현`
- `3fb104eb task 493: 표 셀 속성 대화상자 크기 고정`

## 3. 변경 파일

주요 소스:

- `src/model/table.rs`
- `src/parser/hwpx/section.rs`
- `src/serializer/hwpx/table.rs`
- `src/document_core/commands/table_ops.rs`
- `src/document_core/queries/field_query.rs`
- `src/wasm_api.rs`
- `rhwp-studio/src/core/types.ts`
- `rhwp-studio/src/command/commands/table.ts`
- `rhwp-studio/src/engine/cursor.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/input-handler-mouse.ts`
- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/styles/table-selection.css`
- `rhwp-studio/src/styles/table-cell-props.css`
- `rhwp-studio/src/ui/table-cell-props-dialog.ts`

테스트/샘플/문서:

- `tests/issue_493_cell_attrs.rs`
- `samples/셀보호.hwp`
- `samples/셀보호.hwpx`
- `mydocs/plans/task_m100_493.md`
- `mydocs/plans/task_m100_493_impl.md`
- `mydocs/working/task_m100_493_stage1.md`
- `mydocs/working/task_m100_493_stage2.md`
- `mydocs/working/task_m100_493_stage3.md`

## 4. 검증

자동/로컬 검증:

- `cargo test --test issue_493_cell_attrs`
- `cargo test --test issue_493_hwpx_cell_field_name`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test set_cell_field_text_updates_text_metadata --lib`
- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `wasm-pack build --target web --out-dir pkg`
- `git diff --check`
- `npm run build` (`rhwp-studio`)

시각/동작 검증:

- `samples/셀보호.hwp` 로드 후 보호 셀 hover 표시 확인
- 보호 셀 클릭 후 셀 선택 상태 진입 확인
- 보호 셀 입력 차단 확인
- 보호 셀 선택 상태에서 `셀 속성...` 진입 확인
- 표 외곽선 클릭 후 표 객체 선택 확인
- 표 객체 선택 상태에서 `표 속성...` 진입 확인
- 작업지시자 시각 검증으로 표/셀 속성 대화상자 탭 전환 시 모달 크기 유지 확인

## 5. 참고

- 작업 모드: 기여자 모드
- 기여자 모드 지침에 따라 오늘할일 문서는 생성하지 않았다.
