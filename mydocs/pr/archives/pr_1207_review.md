# PR #1207 검토 — exam_social.hwp 중첩 표 셀 붙여넣기 경로 보존 (cellPath)

- **작성일**: 2026-06-01
- **PR**: #1207 (OPEN)
- **컨트리뷰터**: @postmelee (핵심 컨트리뷰터 — #1190/#1185/#1175/#1174/#1163/#1019 머지)
- **연결 이슈**: #1198 (`Related:` — 컨트리뷰터 규칙상 자동 종료 안 함, OPEN 유지)
- **base/head**: `devel` ← `task-1198` (cross-repo)
- **규모**: 13 파일, +996/−239 (소스 4 + TS 2 + 테스트 1 + 작업문서 6)
- **mergeable**: MERGEABLE / BEHIND
- **CI**: **전부 PASS** (Build & Test 11m49s / Canvas visual diff / Analyze ×3 / CodeQL)
- **라벨**: enhancement / 마일스톤 v1.0.0

## 1. 문제 (이슈 #1198 + 코드 확인)

`exam_social.hwp` 1쪽 `성명` 입력칸(중첩 표 안쪽 셀)에 rhwp-studio 내부 클립보드/HTML 붙여넣기
시, 커서가 안쪽 셀에 있어도 결과가 **바깥 셀**에 삽입됨. 일반 키보드 입력은 정상.
원인: 붙여넣기 API 가 단일 셀(`control, cell`) 4파라미터만 받아 **중첩 경로(cellPath)** 를
표현 못 함 → 안쪽 셀이 바깥 셀로 해석됨.

## 2. 수정 내용 검토

| 파일 | 변경 |
|------|------|
| `wasm_api.rs` | `pasteInternalInCellByPath` / `pasteHtmlInCellByPath` 신규 바인딩 (+ parse_cell_path) |
| `clipboard.rs` | `paste_*_in_cell_by_path_native` 추가, 공통 헬퍼 `paste_paragraphs_into_cell_paragraphs` 추출. **기존 단일 셀 API 시그니처 보존**(본문만 리팩토링) |
| `text_editing.rs` | `get_cell_paragraphs_mut_by_path`(복수) 분리 + `get_cell_paragraph_mut_by_path` 가 path `[(control,cell,cell_para),…]` 중첩 경로 지원 |
| `html_import.rs` | path 경유 HTML 삽입 |
| `input-handler-keyboard.ts` | **`cellPath.length > 1`이면** path API 라우팅 + 커서 cellPath/cellParaIdx 갱신 |
| `wasm-bridge.ts` | 신규 API 타입 바인딩 |
| `tests/issue_1198_nested_cell_paste.rs` | exam_social 성명 칸 path `[(4,0,3),(0,1,0)]` 붙여넣기 회귀 |

설계 평가:
- **opt-in·하위 호환**: 신규 path API 추가, 기존 단일 셀 API 보존. `cellPath.length > 1`일 때만
  path 경로 → 일반 셀은 기존 동작 그대로. 회귀면 낮음.
- **샘플/좌표 비의존**: cellPath 유무로 동작(특정 controlIndex 하드코딩 없음) —
  `feedback_hancom_compat_specific_over_general` 취지 부합.
- **범위 명확**: paste target 경로 보존만. copy source path API(copySelectionInCellByPath 등)는
  후속 이슈로 분리 명시.

## 3. 위험 평가

- **낮음.** 신규 API 추가 + cellPath.length>1 게이트. 기존 단일 셀/일반 붙여넣기 경로 불변.
- 트러블슈팅 `table_paste_file_corruption`(붙여넣기 후 HWP 저장 손상) 경고 영역이나, 본 PR 은
  표 삽입이 아니라 **셀 라우팅(어느 셀에 넣을지)** 변경 → 표 구조/저장 직렬화 무관. 손상 위험 낮음.

## 4. 검증 결과 (로컬 머지 시뮬 `pr1207-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ CLEAN (orders 자동 머지 — 우리 기록 + 컨트리뷰터 #1198 기록 공존) |
| fmt / build / clippy(lib) | ✅ CLEAN |
| 전체 테스트 `cargo test --tests` | ✅ **1924 passed, 0 failed** |
| issue_1198 통합 테스트 | ✅ 2 passed (성명 칸 path 붙여넣기 + HTML, get_text_in_cell_by_path 검증) |
| issue_850 회귀(성명 칸 경로) | ✅ 3 passed |
| rhwp-studio 빌드 (TS) | ✅ `tsc && vite build` (신규 API 타입 정합 포함) |
| CI(PR) | ✅ 전부 PASS |

## 5. 판단 — 머지 권고 (편집기 동작 판정 게이트)

- 진단 정확, opt-in 하위 호환 설계, 통합 테스트로 실제 성명 칸 삽입 위치 검증, 회귀 0, CI green.
- rhwp-studio 편집기 인터랙션이 본질 → 통합 테스트로 코어 동작은 확증되나, **실제 편집기 E2E**
  (성명 칸 클릭 → 붙여넣기 → 안쪽 셀 삽입)는 작업지시자 수동 확인이 정확.
- 승인 + 동작 확인 시 메인테이너 로컬 `--no-ff` 머지 + push + WASM 빌드(신규 API 노출).
  이슈 #1198 은 `Related:` 라 자동 종료 안 됨 → 머지 후 수동 클로즈.

## 6. 비고

- copy source path API 대칭화(copySelectionInCellByPath 등)는 후속 이슈(PR 본문 명시).
- @postmelee #1212(입력 편집 재렌더) 동시 OPEN — 본 PR 과 독립(붙여넣기 라우팅 vs 재렌더).
- 신규 WASM API 추가 → 머지 후 WASM 재빌드 필요(편집기가 쓰는 export).
