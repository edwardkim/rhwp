# PR #1207 처리 보고서 — exam_social.hwp 중첩 표 셀 붙여넣기 경로 보존 (cellPath)

- **작성일**: 2026-06-01
- **PR**: #1207 → **MERGED** (devel, 로컬 `--no-ff` 머지)
- **컨트리뷰터**: @postmelee (핵심 컨트리뷰터)
- **연결 이슈**: #1198 → **CLOSED** (`Related:` — 수동 클로즈)
- **판단**: **머지** ✅ (작업지시자 승인)

## 결정 사유

중첩 표 안쪽 셀 붙여넣기가 바깥 셀로 해석되던 결함을, cellPath 기반 native/WASM API 추가로 해결.
opt-in·하위 호환(기존 단일 셀 API 보존, cellPath.length>1 게이트), 통합 테스트로 성명 칸 삽입
위치 검증, 회귀 0(1924 passed), CI green.

## 변경 요약 (13 파일, +996/−239)

| 파일 | 변경 |
|------|------|
| `wasm_api.rs` | pasteInternalInCellByPath / pasteHtmlInCellByPath 바인딩 + parse_cell_path |
| `clipboard.rs` | paste_*_in_cell_by_path_native + 공통 헬퍼 추출(기존 단일 셀 API 보존) |
| `text_editing.rs` | get_cell_paragraphs_mut_by_path 분리 + 중첩 path [(control,cell,cell_para),…] 지원 |
| `html_import.rs` | path 경유 HTML 삽입 |
| `input-handler-keyboard.ts` | cellPath.length>1 시 path API 라우팅 + 커서 갱신 |
| `wasm-bridge.ts` | 신규 API 타입 바인딩 |
| `tests/issue_1198_nested_cell_paste.rs` | 성명 칸 path [(4,0,3),(0,1,0)] 붙여넣기 회귀 |
| mydocs ×6 | 컨트리뷰터 작업 문서 |

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge `--no-ff` | ✅ CLEAN (orders 자동 머지 — 우리 기록 + #1198 기록 공존) |
| fmt / build / clippy(lib) | ✅ CLEAN |
| 전체 테스트 `cargo test --tests` | ✅ **1924 passed, 0 failed** |
| issue_1198 통합 테스트 | ✅ 2 passed (성명 칸 path 붙여넣기 + HTML) |
| issue_850 회귀 | ✅ 3 passed |
| rhwp-studio 빌드(TS) | ✅ tsc && vite build |
| CI(PR) | ✅ 전부 PASS (Build&Test/Canvas visual diff/Analyze×3/CodeQL) |
| **동작 테스트** | ✅ **통과** (작업지시자, rhwp-studio 에서 성명 칸 붙여넣기 → 안쪽 셀 삽입 확인) |
| WASM | ✅ pkg 빌드 — 신규 API(pasteInternalInCellByPath/pasteHtmlInCellByPath) JS 바인딩 노출 확인 |

## 위험 평가

- 낮음. opt-in 신규 API + cellPath.length>1 게이트, 기존 단일 셀/일반 붙여넣기 불변.
- `table_paste_file_corruption`(저장 손상) 영역이나 본 PR 은 셀 라우팅(어느 셀)이지 표 삽입/직렬화
  무관 → 손상 위험 낮음.

## 처리 절차

1. PR 정보 — MERGEABLE/BEHIND, CI green, 13파일. @postmelee 사이클(#1207/#1212). 트러블슈팅 검색(paste/cell).
2. 핵심 diff 검토 — cellPath API 추가, 기존 API 보존, TS cellPath.length>1 라우팅, 통합 테스트 실효.
3. 로컬 `pr1207-verify`: fmt/build/clippy(lib)/test 1924 / issue_1198·850 / rhwp-studio 빌드.
4. 작업지시자 머지 승인.
5. devel `--no-ff` 머지 + push + WASM 빌드. 이슈 #1198 클로즈.

## 비고

- copy source path API 대칭화(copySelectionInCellByPath 등)는 후속 이슈(PR 본문 명시).
- 신규 WASM API → 머지 후 WASM 재빌드(편집기 export).
