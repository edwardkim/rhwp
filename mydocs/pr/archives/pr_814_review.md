---
PR: #814
제목: fix — searchAllText API 추가, rhwpDev.search() 다중 매치 정합성 수정 (closes #692)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 13번째 PR
base / head: devel / contrib/fix-search-all
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: 결과 부재
변경 규모: +202 / -0, 6 files
커밋: 2
검토일: 2026-05-11
---

# PR #814 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #814 |
| 제목 | fix: searchAllText API 추가 — rhwpDev.search() 다중 매치 수정 (#692) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 (5/11 사이클 **13번째 PR**) |
| base / head | devel / contrib/fix-search-all |
| mergeable | CONFLICTING (DIRTY — 5 파일 충돌) |
| CI | 결과 부재 |
| 변경 규모 | +202 / -0, 6 files |
| 커밋 수 | 2 (1 본질 + 1 리뷰 반영) |
| closes | #692 |
| 관련 | PR #684 (Issue #449 후속) + Issue #692 (작업지시자 직접 검증 영역 영역 발견) |

## 2. 본질 (Issue #692)

`rhwpDev.search(text)` 영역 영역 반복 호출 방식 영역 영역 다중 매치 불완전.

### 결함 본질
`search_text_native` 영역 단건 검색 영역 영역 `h.char_offset > from_char` (strict greater-than) 비교 → **위치 0 영역 영역 시작 매치 누락**. 반복 호출 방식 영역 영역 wrap-around 가드 / 후진 가드 / 반복 횟수 한계 / `wrapped` 필드 등 복잡한 종료 조건 영역 영역 정합되지 않음.

### Issue #692 영역 영역 명시
- PR #684 영역 영역 옵션 B 정정 (commit `a1ea189`) — 4 가드 추가 (`wrapped` 필드 + 가드 + 반복 횟수 한계 + 후진 가드)
- 위 4 가드 영역 영역 적용 후에도 동작 미정합 (작업지시자 직접 검증)

## 3. 정정 본질 — 6 files, +202/-0

### 3.1 Rust WASM API (2 files)

**`src/document_core/queries/search_query.rs` (+45)** — `search_all_text_native`:
```rust
pub fn search_all_text_native(
    &self, query: &str, case_sensitive: bool, include_cells: bool,
) -> Result<String, HwpError>
```
- 내부 `search_all()` 영역 영역 모든 매치 일괄 반환 (Vec → JSON)
- `include_cells = false` 영역 영역 본문 매치만 (cell_context 없음)
- `include_cells = true` 영역 영역 표 셀/글상자 포함
- `query.is_empty()` 영역 영역 빈 배열 반환

**`src/wasm_api.rs` (+13)** — `searchAllText` 바인딩

### 3.2 TypeScript (4 files)

**`rhwp-studio/src/core/types.ts` (+16)** — `SearchHit` 인터페이스:
```typescript
export interface SearchHit {
  sec: number;
  /** 본문 매치: 문단 인덱스. 셀 매치: 부모(호스트) 문단 인덱스 (= cellContext.parentPara) */
  para: number;
  charOffset: number;
  length: number;
  cellContext?: {
    parentPara: number;
    ctrlIdx: number;
    cellIdx: number;
    cellPara: number;
  };
}
```

**`rhwp-studio/src/core/wasm-bridge.ts` (+5)** — `searchAllText` 래퍼:
```typescript
searchAllText(query: string, caseSensitive: boolean, includeCells: boolean = false): SearchHit[] {
  if (!this.doc || typeof (this.doc as any).searchAllText !== 'function') return [];
  return JSON.parse((this.doc as any).searchAllText(query, caseSensitive, includeCells));
}
```

**`rhwp-studio/src/core/rhwp-dev.ts` (+121, 재작성)** — `initRhwpDev(wasm)` 영역 영역 export:
- `showAllIds(pageNum?)` — 페이지 영역 영역 모든 문단 ID console.table
- `search(text, includeCells)` — `searchAllText` 기반 일괄 호출 (반복 호출 → 일괄 호출 교체)
- `findNearest(targetId, pageNum?)` — 가장 가까운 paraIdx 검색
- `help()` — DevTools toolkit 안내

**`rhwp-studio/src/main.ts` (+2)** — DEV 모드 영역 영역 `initRhwpDev(wasm)` 호출

### 3.3 리뷰 반영 commit (`f299e962`)
- 인코딩 깨짐 수정
- `SearchHit.para` 의미 문서화 (본문 vs 셀)

## 4. 본 환경 충돌 분석

### 4.1 5 파일 충돌

| 파일 | base | our (devel) | their (PR) |
|------|------|-------------|------------|
| `rhwp-studio/src/core/rhwp-dev.ts` | (devel 영역 영역 PR #684 영역 영역 존재) | bede66e1 | cef783cad |
| `rhwp-studio/src/core/types.ts` | 1541b960 | 839a4975 | ffbe5b78 |
| `rhwp-studio/src/core/wasm-bridge.ts` | 874d01a0 | 773b03db | 5c7b834e |
| `rhwp-studio/src/main.ts` | bdc1a0a3 | 8a3b0ad7 | 4b18c7c8 |
| `src/document_core/queries/search_query.rs` | 4bb09ba9 | 0115bf29 | 7461ae07 |
| `src/wasm_api.rs` | f442f5e5 | 3261bfd3 | 96bd15460 |

### 4.2 정합 전략

- `rhwp-dev.ts` — **incoming (PR) 측 우선 — 본 PR 영역 영역 재작성** (반복 호출 → 일괄 호출). PR #684 영역 영역 4 가드 + `SearchResult` 인터페이스 영역 영역 폐기됨.
- `types.ts` — PR 측 `SearchHit` 신규 추가 + devel 측 기존 영역 영역 양측 보존 정합.
- `wasm-bridge.ts` — PR 측 `searchAllText` 메서드 추가 + devel 측 기존 영역 영역 양측 보존 정합.
- `main.ts` — PR 측 `initRhwpDev` 호출 추가 + devel 측 기존 영역 영역 양측 보존 정합.
- `search_query.rs` / `wasm_api.rs` — PR 측 신규 API 추가 + devel 측 기존 영역 영역 양측 보존 정합 (auto-merge 예상).

## 5. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `search_all` (기존 내부 함수) | 모든 매치 일괄 반환 |
| `SearchHit` Rust struct (기존) | 매치 위치 |
| `format_search_hit` (기존) | JSON 변환 (수동 JSON 생성) |

→ 신규 인프라 도입 부재 — 기존 `search_all` 내부 함수 영역 영역 WASM 직접 노출.

## 6. 영역 좁힘 (회귀 부재 가드)

- `searchAllText` 영역 영역 신규 API (opt-in) — 기존 `searchText` (단건) 동작 보존
- `rhwpDev` 영역 영역 DEV 모드 (`import.meta.env.DEV`) 영역 영역만 등록 — 프로덕션 영향 부재
- `include_cells` 옵션 영역 영역 셀 포함/제외 분리 — 기본 false (본문만)
- `query.is_empty()` 영역 영역 빈 배열 반환 가드

## 7. CI 결과 부재

mergeStateStatus = `DIRTY` 영역 CI 미실행. 충돌 해결 후 자기 검증 필수.

## 8. 처리 옵션

### 옵션 A (권장) — 2 commits cherry-pick + 5 파일 충돌 수동 해결 + no-ff merge

```bash
git checkout local/devel
git cherry-pick 0244be96 f299e962
# 5 파일 충돌 수동 해결:
#   - rhwp-dev.ts: incoming (PR 측) 우선 — searchAllText 기반 재작성
#   - types.ts / wasm-bridge.ts / main.ts: 양측 보존 정합
#   - search_query.rs / wasm_api.rs: auto-merge 예상
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash cherry-pick + 충돌 수동 해결

본 환경 영역 영역 commit 이력 보존 권장 옵션 A.

## 9. 검증 게이트

### 9.1 자기 검증
- [ ] cherry-pick 2 commits + 5 파일 충돌 수동 해결
- [ ] tsc --noEmit
- [ ] cargo test + cargo clippy
- [ ] WASM 재빌드 (Rust 신규 API 영역 영역 재빌드 필수)
- [ ] 광범위 sweep (Rust 변경 영역 영역 점검 — search 영역 영역 layout 무관 영역 영역 면제 가능)

### 9.2 시각/인터랙션 판정 게이트 — **작업지시자 인터랙션 검증 권장**
- DEV mode 영역 영역 브라우저 콘솔 영역 `rhwpDev.search("text")` → 모든 매치 console.table
- 위치 0 영역 영역 시작 매치 포함 정합
- `rhwpDev.search("text", true)` → 표 셀 포함 매치
- `rhwpDev.help()` → toolkit 안내 출력
- `rhwpDev.findNearest(id, page)` 기존 동작 회귀 부재
- `rhwpDev.showAllIds(page)` 기존 동작 회귀 부재

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 13번째 PR) |
| `feedback_image_renderer_paths_separate` | Rust search 변경 영역 영역 layout/렌더링 경로 무관 — sweep 면제 가능 |
| `feedback_process_must_follow` | 인프라 재사용 (`search_all` 내부 함수) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | `include_cells` 옵션 영역 영역 본문/셀 분리 + `query.is_empty()` 가드 영역 영역 영역 좁힘 |
| `feedback_diagnosis_layer_attribution` | `h.char_offset > from_char` (strict greater-than) 본질 진단 (Issue #692 명시) — PR #684 영역 영역 4 가드 영역 영역 한계 점검 |
| `feedback_visual_judgment_authority` | DEV mode 영역 영역 콘솔 검증 권장 — Issue #692 영역 영역 작업지시자 직접 검증 영역 영역 발견 |
| `feedback_pr_supersede_chain` | PR #684 (Issue #449 후속 + 4 가드, 미정합) → **PR #814** (searchAllText 일괄 호출 영역 영역 근본 정정) — (a) 패턴 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick 2 commits + 5 파일 충돌 수동 해결
2. 자기 검증 (tsc + cargo test + cargo clippy)
3. WASM 재빌드
4. 작업지시자 DEV mode 콘솔 인터랙션 검증
5. 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #692 close
6. PR #814 close

---

작성: 2026-05-11
