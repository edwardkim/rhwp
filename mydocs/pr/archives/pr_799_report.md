---
PR: #799
제목: Task #571 — 문서 비교·이력 (compare + history) 및 diff-engine 정합 (분리 PR 1/3)
컨트리뷰터: @xogh3198 (thlee2) — 3번째 시도 (PR #571 → #623 → #799)
처리: 옵션 A — 5 commits cherry-pick + 충돌 수동 해결 + .actual.svg 제거 + no-ff merge
처리일: 2026-05-11
머지 commit: 6b9ad9e0
Refs: #571 (Issue) — 후속 분리 PR 2/3, 3/3 영역 영역 close
---

# PR #799 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (5 commits cherry-pick + 충돌 수동 해결 + .actual.svg 제거 commit + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `6b9ad9e0` (--no-ff merge) |
| Cherry-pick commits | 4 PR commits + 1 정정 commit (총 5) |
| Refs | Issue #571 (후속 분리 PR 2/3, 3/3 영역 영역 close) |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + WASM 4.5 MB |

## 2. 본질 (Task #571 분리 PR 1/3)

**TypeScript/UI 본질만** — Compare + History 다이얼로그 + diff-engine + WasmBridge 보조 API.
Rust 변경 부재 (분리 PR 1/3 정합 — PR #571 base skew 문제 해결).

### 2.1 주요 변경 영역
| 영역 | 위치 + 규모 |
|------|------------|
| **diff-engine** | `src/compare/diff-engine.ts` (+3105) — 정렬/비교 로직 |
| **Compare dialog** | `src/ui/compare-dialog.ts` (+564) + `compare-result-window.ts` (+526) + `styles/compare-dialog.css` (+368) |
| **History dialog** | `src/ui/history-dialog.ts` (+530) |
| **IndexedDB 연동** | `src/history/idb-store.ts` (+217) + `history/types.ts` |
| **session/types/debug** | `src/compare/session.ts` (+49) + `compare/types.ts` (+124) + `compare-debug.ts` (+28) |
| **diff-location-label** | `src/compare/diff-location-label.ts` (+46) |
| **WasmBridge 확장** | `src/core/wasm-bridge.ts` (+93/-2) — 4 신규 API |
| **메뉴 + 단축키** | `index.html` (+6) + `command/commands/edit.ts` (+41) + `shortcut-map.ts` (+2) — Alt+Shift+V / Ctrl+Shift+H |
| **diff-engine 문서** | `src/compare/diff-engine-readme.md` (+471) |

### 2.2 WasmBridge 신규 API
- `hasLoadedDocument()` — 메인 뷰 문서 존재 점검 (단순 helper)
- `releaseDocument()` — 보조 인스턴스 메모리 누수 방지
- `getTableSignature(sec, parentPara, controlIdx)` — try/catch graceful fallback
- `getParagraphStableId(sec, para)` — WASM 미구현 영역 영역 JS 폴백 (`paragraph.rs::stable_id` 영역 영역 분리 PR 3/3 영역 영역 도입 예정)

## 3. 본 환경 충돌 수동 해결

mergeStateStatus = `DIRTY`. 1 cherry-pick 영역 영역 충돌 (`wasm-bridge.ts`) + 1 영역 영역 빈 commit (`package-lock.json` 영역 영역 ours 보존).

### 3.1 `wasm-bridge.ts` 충돌 영역 (#739/#750/#786 누적 vs PR #799 신규 API)

| 영역 | HEAD (devel) | incoming (PR #799) | 본 환경 해결 |
|------|--------------|---------------------|--------------|
| 라인 116~149 | `populateExternalImagesFromDevServer` (Task #741 후속) | (해당 영역 부재) | HEAD 보존 |
| 라인 150~153 | (해당 영역 부재) | `hasLoadedDocument()` | incoming 적용 |

→ 두 영역 모두 포함되도록 정합 (둘 다 독립적 메소드).
→ 다른 신규 API 3개 (`releaseDocument` / `getTableSignature` / `getParagraphStableId`) 영역 영역 auto-merge 성공.

### 3.2 `package-lock.json` (ours)
PR 측 변경 무시 (devel 측 보존). 차후 npm install 영역 영역 자연 sync.

## 4. 메인테이너 정정 — `.actual.svg` 제거 commit

PR 영역 영역 잘못 staging 된 테스트 임시 출력 파일 **5개 (+2611 lines)** 제거:
```
tests/golden_svg/form-002/page-0.actual.svg
tests/golden_svg/issue-147/aift-page3.actual.svg
tests/golden_svg/issue-157/page-1.actual.svg
tests/golden_svg/issue-267/ktx-toc-page.actual.svg
tests/golden_svg/table-text/page-0.actual.svg
```

### 본질
- `*.svg` (예: `page-0.svg`) — golden (expected) 파일, 저장소 포함
- `*.actual.svg` — 테스트 실행 시 생성된 실제 출력 (golden 비교용 임시 파일), 저장소 미포함
- 컨트리뷰터 환경 영역 영역 테스트 실행 후 임시 생성된 영역 영역 잘못 staging

→ 정정 commit `867a9078` 영역 영역 5 파일 제거. 차후 컨트리뷰터 안내 — 로컬 `.gitignore` 영역 영역 추가 권장.

## 5. 컨트리뷰터 사이클 — 3번째 시도

| PR | 시점 | 처리 | 사유 |
|----|------|------|------|
| PR #571 | 5/4 | close | Base skew (`b84c5e9` 9 commits 뒤) + paragraph.rs::stable_id 부재 + Rust 함수 시각 게이트 필요 |
| PR #623 | 5/6 | close | 분리 PR 1/3 가이드 정합 영역 영역 close (작업지시자 결정) |
| **PR #799** | **5/11** | **머지** | 분리 PR 1/3 (TypeScript/UI) 본질 정합 |

작업지시자께서 "처음 기여" 표현 사용 — 사이클 점검 결과 3번째 시도 (그러나 첫 머지 시도).

## 6. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 5 commits | ✅ 1 충돌 수동 해결 + 1 빈 commit skip |
| `.actual.svg` 제거 commit | ✅ 5 파일 제거 (-2611 lines) |
| `tsc --noEmit` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| WASM 빌드 (Docker) | ✅ 4.5 MB |
| 광범위 sweep | 면제 (TypeScript 단일 영역 영역 SVG 무영향 자명) |

## 7. 작업지시자 시각/인터랙션 검증 ✅ 통과

- **Alt+Shift+V** — 문서 비교 대화상자 정상
- **Ctrl+Shift+H** — 문서 이력 관리 대화상자 정상 (IndexedDB 동작)
- 메뉴 "편집 → 문서 비교 / 문서 이력" 정상
- diff-engine 정렬/비교 동작
- WasmBridge graceful fallback 동작 (콘솔 오류 부재)
- 기존 기능 회귀 부재 (Alt+←/→, Alt+Backspace/Delete 등 PR #794 단축키 보존)

## 8. WasmBridge graceful fallback 설계

```typescript
// getParagraphStableId
const d = this.doc as unknown as { getParagraphStableId?: (a: number, b: number) => string };
if (typeof d.getParagraphStableId !== 'function') return '';
return d.getParagraphStableId(sec, para) ?? '';

// getTableSignature
const d = this.doc as unknown as { getTableSignature?: (a: number, b: number, c: number) => string };
if (typeof d.getTableSignature !== 'function') {
  throw new Error('getTableSignature API unavailable');
}
return d.getTableSignature(sec, parentPara, controlIdx);
```

→ WASM 측 API 미구현 영역 영역 typeof 점검 + 폴백 — 본 PR 영역 영역 머지 가능 (분리 PR 3/3 영역 영역 후속 도입 예정).

## 9. 후속 분리 PR (PR 본문 명시)

- **PR 2/3** — Rust pagination/typeset 신규 함수 (PR #571 의 페이지네이션 본질)
- **PR 3/3** — `paragraph.rs::stable_id` parser + WASM 노출 (identity 전략 완성 — `getParagraphStableId` 영역 영역 실제 WASM 측 구현)

→ 본 머지 commit `6b9ad9e0` 영역 영역 후속 PR 2/3, 3/3 영역 영역 베이스.

## 10. 영향 범위

### 10.1 변경 영역
- rhwp-studio TypeScript: compare/, history/, ui/ 신규 모듈 + wasm-bridge.ts 신규 API + 메뉴/단축키
- 문서: `diff-engine-readme.md`

### 10.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재 (분리 PR 1/3 정합)
- HWP3/HWPX 변환본 시각 정합 (TypeScript 단일 영역 영역 SVG 무영향 자명)
- 기존 단축키 / 기존 기능 회귀 부재

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @xogh3198 **3번째 시도** (PR #571 → #623 → #799). 작업지시자 "처음 기여" 표현 영역 영역 사실 확인 — 첫 머지 시도지만 3번째 PR. **사이클 점검 의무 영역 영역 메인테이너 필수 절차 입증** |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 (분리 PR 1/3 정합) |
| `feedback_pr_supersede_chain` | PR #571 → #623 → **#799** 점진적 정합 (base skew 해결 + 분리 PR 가이드 정합) — 세 시도 영역 영역 첫 머지 |
| `feedback_process_must_follow` | 분리 PR 1/3 (TypeScript) 머지 + 분리 PR 2/3, 3/3 (Rust) 후속 — 메인테이너 권장 가이드 정합 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 신규 기능 영역 영역 작업지시자 인터랙션 검증 ✅ 통과 |
| `feedback_pr_comment_tone` | "수고하셨습니다" 톤 — 정정 사항 (actual.svg 제거 + package-lock 보존) 차분 안내 |

## 12. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #571 영역 영역 후속 분리 PR 2/3, 3/3 영역 영역 close 예정
- 컨트리뷰터 안내 — 로컬 `.gitignore` 영역 영역 `tests/golden_svg/**/*.actual.svg` 추가 권장

---

작성: 2026-05-11
