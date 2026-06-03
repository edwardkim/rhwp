# 구현계획서 — Task M100-1198: 중첩 표 셀 붙여넣기 `cellPath` 보존

## 설계 요약

- 중첩 표에서는 `DocumentPosition.controlIndex/cellIndex/cellParaIndex`가 하위 호환을 위해 최외곽 표 좌표를 유지한다.
- 실제 편집 대상은 `DocumentPosition.cellPath`의 마지막 엔트리로 판단한다.
- 기존 얕은 셀 API(`pasteInternalInCell`, `pasteHtmlInCell`)는 유지하고, 중첩 표 전용 path API를 추가한다.
- 구현은 `exam_social.hwp`나 특정 좌표/컨트롤 번호에 의존하지 않고 `cellPath.length > 1` 조건으로 라우팅한다.

## Stage 1 — Rust native path 기반 내부 클립보드 붙여넣기

**목표**: 내부 클립보드 붙여넣기를 최종 중첩 셀 문단 벡터에 적용하는 native API를 만든다.

변경 대상:

- `src/document_core/commands/text_editing.rs`
  - `get_cell_paragraphs_mut_by_path(...) -> &mut Vec<Paragraph>` 헬퍼 추가.
  - 기존 `get_cell_paragraph_mut_by_path(...)`와 동일한 path 순회 규칙을 사용하되, 마지막 셀의 문단 목록을 반환한다.
- `src/document_core/commands/clipboard.rs`
  - 셀 문단 목록에 클립보드 문단을 삽입하는 공통 헬퍼 추가.
  - 기존 `paste_internal_in_cell_native(...)`는 얕은 셀 조회 후 공통 헬퍼를 호출하도록 정리한다.
  - 신규 `paste_internal_in_cell_by_path_native(section, parentPara, path, charOffset)` 추가.
  - path API는 최외곽 표를 dirty 처리하고 `raw_stream` 무효화, section dirty, pagination 갱신을 수행한다.
- `src/wasm_api.rs`
  - `pasteInternalInCellByPath(section, parentPara, pathJson, charOffset)` 바인딩 추가.

회귀 테스트:

- `tests/issue_1198_nested_cell_paste.rs` 신규 추가.
- `samples/exam_social.hwp`에서 `성명` 칸 hit-test path `[(4,0,3),(0,1,0)]`를 사용한다.
- 문서 안의 첫 유효 텍스트 1글자를 내부 클립보드에 복사한 뒤, 신규 path API로 `성명` 칸에 붙여넣고 `get_text_in_cell_by_path`로 실제 삽입 위치를 검증한다.
- 기존 #850 테스트도 함께 실행해 일반 입력 path 회귀를 확인한다.

검증:

```text
cargo test --test issue_1198_nested_cell_paste -- --nocapture
cargo test --test issue_850_answer_sheet_name_hit_test issue_850_exam_social_answer_sheet_name_cell_keeps_outer_path -- --nocapture
```

보고서:

```text
mydocs/working/task_m100_1198_stage1.md
```

## Stage 2 — HTML 붙여넣기 path API와 rhwp-studio 라우팅

**목표**: 내부 클립보드와 HTML 붙여넣기 모두 중첩 표에서는 path API를 사용하게 한다.

변경 대상:

- `src/document_core/commands/html_import.rs`
  - HTML 파싱 결과 문단을 셀 문단 목록에 삽입하는 공통 헬퍼 추가.
  - 신규 `paste_html_in_cell_by_path_native(section, parentPara, path, charOffset, html)` 추가.
- `src/wasm_api.rs`
  - `pasteHtmlInCellByPath(section, parentPara, pathJson, charOffset, html)` 바인딩 추가.
- `rhwp-studio/src/core/wasm-bridge.ts`
  - `pasteInternalInCellByPath(...)`, `pasteHtmlInCellByPath(...)` 래퍼 추가.
- `rhwp-studio/src/engine/input-handler-keyboard.ts`
  - `cellPath.length > 1`이면 내부 클립보드 붙여넣기에서 `pasteInternalInCellByPath` 호출.
  - 외부 HTML 붙여넣기도 중첩 셀에서는 `pasteHtmlInCellByPath` 호출.
  - 붙여넣기 결과의 `cellParaIdx`를 읽어 `cellParaIndex`와 `cellPath` 마지막 엔트리를 함께 갱신한다.
  - 본문과 얕은 표 셀 경로는 기존 API를 그대로 유지한다.

생성 산출물:

- `pkg/`는 로컬 WASM 빌드 산출물이다.
- `wasm-pack build --target web --out-dir pkg`로 타입 선언을 최신 API에 맞춰 확인하되, Git 추적 대상이 아니므로 PR에는 포함하지 않는다.

검증:

```text
cd rhwp-studio
npm test
npm run build
```

보고서:

```text
mydocs/working/task_m100_1198_stage2.md
```

## Stage 3 — 통합 검증과 최종 보고

**목표**: path 기반 붙여넣기와 기존 얕은 셀/본문 붙여넣기 회귀를 함께 확인한다.

검증 항목:

- 신규 #1198 테스트 GREEN.
- #850 `exam_social.hwp` hit-test/일반 입력 테스트 GREEN.
- `cargo fmt --check`.
- 가능하면 `cargo test --lib`.
- `cargo test --lib` 통과 여부를 확인한다.
- rhwp-studio `npm test`, `npm run build`.

최종 보고서:

```text
mydocs/report/task_m100_1198_report.md
```

## 제외 범위

- `copySelectionInCellByPath`, `exportSelectionInCellHtmlByPath`는 이번 기본 범위에서 제외한다.
- 이번 재현은 붙여넣기 대상 경로 손실이 핵심이며, 복사 source가 중첩 표인 경우까지 같은 PR에 확장하면 API 표면과 검증 범위가 커진다.
- 구현 중 중첩 셀 source 복사가 동일 결함으로 확인되면 별도 이슈 또는 v2 계획으로 분리한다.

## 승인 요청

위 구현계획으로 소스 수정을 시작한다.
