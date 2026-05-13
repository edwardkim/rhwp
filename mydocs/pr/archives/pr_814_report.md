---
PR: #814
제목: fix — searchAllText API 추가, rhwpDev.search() 다중 매치 근본 정정 (closes #692)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 13번째 PR
처리: 옵션 A — 2 commits cherry-pick + 충돌 수동 해결 + 중복 import 제거 commit + no-ff merge
처리일: 2026-05-11
머지 commit: 306159c3
---

# PR #814 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 PR commits + 1 정정 commit + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `306159c3` (--no-ff merge) |
| Cherry-pick commits | 2 PR + 1 정정 (중복 import 제거) |
| closes | #692 |
| 시각 판정 | ✅ 작업지시자 DEV mode 콘솔 검증 통과 |
| 자기 검증 | tsc + cargo test/clippy ALL GREEN |
| WASM 재빌드 | 4.6 MB |

## 2. 본질 (Issue #692)

`rhwpDev.search(text)` 반복 호출 방식 영역 다중 매치 불완전. PR #684 영역 4 가드 (wrapped 필드 + 가드 + 반복 횟수 한계 + 후진 가드) 적용 후에도 미정합 (작업지시자 직접 검증 영역 발견).

### 근본 결함 진단
- `search_text_native` 단건 검색 영역 영역 `h.char_offset > from_char` (strict greater-than)
- → 위치 0 영역 시작 매치 누락
- 반복 호출 방식 영역 영역 wrap-around / 후진 / 반복 횟수 한계 종료 조건 정합 부재

## 3. 정정 본질 — 6 files, +202/-0

### 3.1 Rust WASM API
- `src/document_core/queries/search_query.rs` (+45) — `search_all_text_native`:
  - `search_all` 내부 함수 영역 영역 모든 매치 일괄 반환 (Vec → JSON)
  - `include_cells = false` 영역 영역 본문 매치만 / true 영역 영역 셀 포함
  - `query.is_empty()` 영역 영역 빈 배열 반환 가드
- `src/wasm_api.rs` (+13) — `searchAllText` 바인딩

### 3.2 TypeScript
- `rhwp-studio/src/core/types.ts` (+16) — `SearchHit` 인터페이스 (sec/para/charOffset/length/cellContext)
- `rhwp-studio/src/core/wasm-bridge.ts` (+5) — `searchAllText` 래퍼
- `rhwp-studio/src/core/rhwp-dev.ts` (+121, 재작성) — `initRhwpDev(wasm)` + `search(text, includeCells)` + `showAllIds(pageNum?)` + `findNearest(targetId, pageNum?)` + `help()`
- `rhwp-studio/src/main.ts` (+2) — DEV 모드 영역 영역 `initRhwpDev(wasm)` 호출

### 3.3 리뷰 반영 commit (`2f4ba942`)
- 인코딩 깨짐 수정
- `SearchHit.para` 의미 문서화 (본문 vs 셀)

## 4. 본 환경 충돌 수동 해결

### 4.1 5 파일 충돌
| 파일 | 정합 전략 |
|------|----------|
| `rhwp-dev.ts` | **incoming (PR 측) 우선** — searchAllText 기반 재작성 (PR #684 의 4 가드 + `SearchResult` 폐기) |
| `types.ts` / `wasm-bridge.ts` / `main.ts` / `search_query.rs` / `wasm_api.rs` | auto-merge 성공 |

### 4.2 정정 commit (`4f763f15`)
`main.ts` 영역 영역 `initRhwpDev` 중복 import 정정 — devel 측 영역 영역 이미 라인 24 영역 영역 import 존재. cherry-pick auto-merge 영역 영역 라인 29 영역 영역 중복 추가됨 → 라인 29 제거 + 라인 24 보존.

## 5. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `search_all` (기존 내부 함수) | 모든 매치 일괄 반환 |
| `SearchHit` Rust struct (기존) | 매치 위치 |
| `format_search_hit` 패턴 (기존) | 수동 JSON 생성 |

→ 신규 인프라 도입 부재 — 기존 `search_all` 내부 함수 영역 영역 WASM 직접 노출.

## 6. 영역 좁힘 (회귀 부재 가드)

- `searchAllText` 신규 API (opt-in) — 기존 `searchText` (단건) 동작 보존
- `rhwpDev` DEV 모드 (`import.meta.env.DEV`) 영역 영역만 등록 — 프로덕션 영향 부재
- `include_cells` 옵션 영역 본문/셀 분리 (기본 false)
- `query.is_empty()` 빈 배열 가드

## 7. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 2 commits + 1 충돌 수동 해결 (`rhwp-dev.ts` incoming 우선) | ✅ |
| 정정 commit (`main.ts` 중복 import 제거) | ✅ |
| `tsc --noEmit` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| `cargo clippy --release --lib -- -D warnings` | ✅ 통과 |
| 광범위 sweep | 면제 (search 영역 영역 layout/렌더링 경로 무관 자명) |
| WASM 빌드 (Docker) | ✅ 4.6 MB |

## 8. 작업지시자 DEV mode 콘솔 검증 ✅ 통과

- `rhwpDev.help()` toolkit 안내 출력
- `rhwpDev.search("text")` 모든 매치 console.table (위치 0 매치 포함 정합)
- `rhwpDev.search("text", true)` 표 셀/글상자 포함 매치
- `rhwpDev.findNearest(id, page)` / `rhwpDev.showAllIds(page)` 기존 동작 보존
- 기존 `searchText` (단건) API 회귀 부재

## 9. CI 결과 부재 (DIRTY 영역)

mergeStateStatus = `DIRTY` 영역 CI 미실행. 본 환경 자기 검증 + 작업지시자 시각 판정 통과 영역 보완.

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 13번째 PR) |
| `feedback_image_renderer_paths_separate` | Rust search 변경 영역 영역 layout/렌더링 경로 무관 |
| `feedback_process_must_follow` | 인프라 재사용 (`search_all` 내부 함수) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | `include_cells` 옵션 본문/셀 분리 + `query.is_empty()` 가드 영역 좁힘 |
| `feedback_diagnosis_layer_attribution` | `h.char_offset > from_char` strict greater-than 본질 진단 (Issue #692 명시) — PR #684 4 가드 한계 정확 점검 |
| `feedback_visual_judgment_authority` | Issue #692 영역 작업지시자 직접 검증 영역 영역 발견 — 본 PR 영역 영역 DEV mode 콘솔 검증 통과 |
| `feedback_pr_supersede_chain` | PR #684 (Issue #449 후속 + 4 가드, 미정합) → **PR #814** (searchAllText 일괄 호출 영역 근본 정정) — (a) 패턴 |

## 11. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #692 close 완료
- **후속 요청 — `rhwpDev.goto(hit)` 메서드** (작업지시자 요청): SearchHit 영역 영역 cursor 이동 + 화면 스크롤. 별 Issue + 후속 PR 영역 영역 진행 예정.

---

작성: 2026-05-11
