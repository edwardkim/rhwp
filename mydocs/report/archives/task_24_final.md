# 타스크 24: 표 안의 텍스트 입력 처리 - 최종 결과 보고서

## 개요

표(Table) 셀 내부에서 텍스트 입력/삭제가 동작하도록 구현하였다. 셀 클릭 → 히트테스트 → 캐럿 표시 → 키보드 입력/삭제가 셀 문맥으로 자동 전달되는 전체 파이프라인을 완성하였다.

## 변경 파일 요약

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/render_tree.rs` | TextRunNode에 셀 식별 필드 4개 추가 |
| `src/renderer/layout.rs` | CellContext 구조체, layout_composed_paragraph에 cell_ctx 파라미터, layout_table에 section_index 파라미터 |
| `src/wasm_api.rs` | insertTextInCell/deleteTextInCell API, reflow_cell_paragraph, getPageTextLayout 셀 정보 포함, 테스트 6개 추가 |
| `web/text_selection.js` | getDocumentPos/setCaretByDocPos/getSelectionDocRange 셀 컨텍스트 지원 |
| `web/editor.js` | 본문/셀 분기 헬퍼 함수, handleTextInsert/handleTextDelete 셀 API 분기 |

## 구현 아키텍처

```
[사용자 클릭] → hitTest() → TextRun(셀 식별 정보 포함) → 캐럿 설정
[키 입력] → getDocumentPos() → {secIdx, charOffset, parentParaIdx, controlIdx, cellIdx, cellParaIdx}
         → _doInsertText() → WASM insertTextInCell()
         → reflow_cell_paragraph(셀 폭 기반) → compose_section → paginate
         → renderCurrentPage() → _restoreCaret(셀 컨텍스트)
```

## 제외 범위 (후속 타스크)

- 셀 내 문단 분리/병합 (Enter 키 → 셀 내 새 문단 생성)
- 셀 크기 자동 조절
- 표 구조 편집 (행/열 추가/삭제/병합)

## 테스트 결과

- 전체 테스트: **344개 통과** (기존 338 + 신규 6)
- 빌드: 성공
- SVG 내보내기: 정상
