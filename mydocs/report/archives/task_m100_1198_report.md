# 최종 보고서 — Task M100-1198: `exam_social.hwp` 중첩 표 셀 붙여넣기 위치 보존

## 요약

`samples/exam_social.hwp` 상단 `성명` 입력칸은 바깥 표 안의 중첩 표 셀이다.
hit-test는 전체 `cellPath`를 반환하지만 붙여넣기 경로가 얕은 셀 좌표만 사용해, 내부 클립보드/HTML 붙여넣기가 바깥 셀에 삽입됐다.

이번 수정은 샘플 전용 좌표나 컨트롤 번호를 사용하지 않고, `DocumentPosition.cellPath.length > 1`이면 전체 경로 기반 API로 붙여넣는 일반 규칙을 추가했다.

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/document_core/commands/text_editing.rs` | `cellPath`가 가리키는 최종 셀의 문단 목록을 얻는 공통 헬퍼 추가 |
| `src/document_core/commands/clipboard.rs` | 내부 클립보드 셀 붙여넣기 공통 헬퍼와 `paste_internal_in_cell_by_path_native(...)` 추가 |
| `src/document_core/commands/html_import.rs` | HTML 셀 붙여넣기 공통 헬퍼와 `paste_html_in_cell_by_path_native(...)` 추가 |
| `src/wasm_api.rs` | `pasteInternalInCellByPath`, `pasteHtmlInCellByPath` WASM 바인딩 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | 신규 WASM API 래퍼 추가 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | 중첩 셀 붙여넣기 라우팅과 붙여넣기 후 커서 `cellPath` 갱신 보정 |
| `tests/issue_1198_nested_cell_paste.rs` | `exam_social.hwp` 기반 내부/HTML 붙여넣기 회귀 테스트 추가 |

## 검증

```text
cargo fmt --check
cargo test --test issue_1198_nested_cell_paste -- --nocapture
cargo test --test issue_850_answer_sheet_name_hit_test issue_850_exam_social_answer_sheet_name_cell_keeps_outer_path -- --nocapture
cargo test --lib
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npm run build
```

결과:

```text
cargo fmt --check: success
issue_1198_nested_cell_paste: 2 passed
issue_850_answer_sheet_name_hit_test: 1 passed
cargo test --lib: 1491 passed, 0 failed, 6 ignored
npm test: 49 passed
wasm-pack build: success, rhwp v0.7.13
npm run build: success
```

## 업스트림 상태

- 브랜치: `task-1198`
- 업스트림 기준: `upstream/devel` `c884205d`
- 버전: `0.7.13`
- `HEAD...upstream/devel`: `0 0`

## 제외

- 중첩 셀 source 복사용 `copySelectionInCellByPath` / `exportSelectionInCellHtmlByPath`는 이번 범위에서 제외했다.
- 이번 재현은 붙여넣기 대상 경로 손실이 핵심이므로, 복사 source path 대칭화는 별도 이슈로 분리하는 편이 안전하다.
