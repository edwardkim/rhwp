---
PR: #799
제목: Task #571 — 문서 비교·이력 (compare + history) 및 diff-engine 정합 (closes #571)
컨트리뷰터: @xogh3198 — 3번째 시도 (PR #571 → #623 → #799, 본질적으로 첫 머지 시도)
base / head: devel / local/task571-rhwp-studio
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: 결과 부재
변경 규모: +8815 / -8, 26 files (대형 PR)
검토일: 2026-05-11
---

# PR #799 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #799 |
| 제목 | Task #571 — 문서 비교·이력 + diff-engine 정합 |
| 컨트리뷰터 | @xogh3198 — **3번째 시도** (이전 PR #571 / #623 close 후 재시도) |
| base / head | devel / local/task571-rhwp-studio |
| mergeStateStatus | DIRTY, mergeable: CONFLICTING |
| CI | 결과 부재 |
| 변경 규모 | **+8815 / -8, 26 files** (대형 PR) |
| 커밋 수 | 5 |
| closes | #571 |

## 2. 이전 시도 이력 점검

### 2.1 PR #571 (5/4) — 첫 시도 → close
- "DIFF update — 문서 비교 + 이력 + 페이지네이션 보조"
- close 사유 (mydocs/pr/archives/pr_571_report.md 영역 영역 기록):
  - **Base skew**: PR base = `b84c5e9` (9 commit 전) — 머지 시 PR #553/551/562/581/582/583/558 의 본질 정정 600+ lines 회귀
  - **paragraph.rs::stable_id** parser+WASM 노출 부재 → identity 전략 미동작
  - **Rust pagination/typeset 신규 함수** 시각 판정 게이트 필요
- 처리 결정 (옵션 C): **분리 PR + base 동기화 권장**

### 2.2 PR #623 (5/6) — 두번째 시도 → close
- "분리 PR (1/3 - 프론트엔드 TS/UI 본질)" — 메인테이너 권장 가이드 정합
- close 사유 (archives 부재) — 머지 부재 (작업지시자 결정 영역 영역 close)

### 2.3 PR #799 (현재) — 세번째 시도
- PR #623 본문 영역 영역 거의 동일 ("Base Skew 해결" + "분리 PR 1/3") — PR #623 close 후 재시도

## 3. 본질 (Task #571 분리 PR 1/3)

**프론트엔드 TS/UI 본질만 (Rust 변경 부재)** — Compare + History 다이얼로그 + diff-engine + WasmBridge 보조 API.

### 3.1 주요 변경 영역
| 영역 | 위치 |
|------|------|
| **diff-engine** | `src/compare/diff-engine.ts` (+3105) — 정렬/비교 로직 |
| **Compare dialog** | `src/ui/compare-dialog.ts` (+564) + `compare-result-window.ts` (+526) + `compare-dialog.css` (+368) |
| **History dialog** | `src/ui/history-dialog.ts` (+530) |
| **IndexedDB 연동** | `src/history/idb-store.ts` (+217) |
| **session/types/debug** | `src/compare/session.ts` (+49) + `compare/types.ts` (+124) + `compare-debug.ts` (+28) |
| **diff-location-label** | `src/compare/diff-location-label.ts` (+46) |
| **WasmBridge 확장** | `src/core/wasm-bridge.ts` (+93/-2) — `hasLoadedDocument` / `releaseDocument` / `getTableSignature` / `getParagraphStableId` (graceful fallback) |
| **메뉴 + 단축키** | `index.html` (+6) + `command/commands/edit.ts` (+41) + `shortcut-map.ts` (+2) — Alt+Shift+V / Ctrl+Shift+H |
| **diff-engine 문서** | `src/compare/diff-engine-readme.md` (+471) |
| **golden SVG** | `tests/golden_svg/*/page-*.actual.svg` (+2611) **⚠️ 문제 — 후술** |

### 3.2 WasmBridge 신규 API
- `hasLoadedDocument()` — 단순 helper
- `releaseDocument()` — 보조 인스턴스 메모리 누수 방지
- `getTableSignature` / `getParagraphStableId` — try/catch graceful fallback (WASM 미구현 영역 영역 JS 폴백)
- `refreshLayout` — 본문 명시

## 4. ⚠️ 문제 영역 — `actual.svg` 파일 잘못 commit

PR 영역 영역 `tests/golden_svg/*/page-*.actual.svg` **6 파일 추가 (+2611)**:
```
tests/golden_svg/form-002/page-0.actual.svg     (952)
tests/golden_svg/issue-147/aift-page3.actual.svg (669)
tests/golden_svg/issue-157/page-1.actual.svg    (507)
tests/golden_svg/issue-267/ktx-toc-page.actual.svg (277)
tests/golden_svg/table-text/page-0.actual.svg   (206)
```

### 본질 진단
- 기존 golden_svg 영역 영역 `*.svg` 형식 (예: `page-0.svg`) — golden (expected) 파일
- `*.actual.svg` 영역 영역 **테스트 실행 시 생성된 실제 출력** — golden 비교용 임시 파일
- **commit 영역 영역 정합 부재** — 컨트리뷰터 환경 영역 영역 테스트 실행 후 임시 생성된 actual.svg 영역 영역 잘못 staging 후 commit

→ 본 환경 영역 영역 cherry-pick 시 제거 필요 (작업지시자 결정 영역 영역 머지 시 영역 영역 제거 commit 추가).

## 5. 충돌 분석

mergeStateStatus = `DIRTY`, mergeable = `CONFLICTING`. PR #623 base 영역 영역 5/6 시점 영역 영역, 본 PR base 영역 영역 갱신됐으나 devel 5/10 + 5/11 사이클 영역 영역 다수 변경 누적:
- `rhwp-studio/index.html` 영역 영역 PR #756 (보기 메뉴 토글) + PR #742 (글꼴 드롭다운) 등 변경
- `command/commands/edit.ts` 영역 영역 PR #752 (edit:delete) 변경
- `command/shortcut-map.ts` 영역 영역 PR #749/#750/#751/#752/#754 누적
- `core/wasm-bridge.ts` 영역 영역 PR #739/#750/#786 영역 영역 누적

→ 충돌 가능성 영역 영역 다수.

## 6. 본 환경 점검

### 6.1 변경 격리
- **순수 TypeScript/CSS** — Rust / WASM / 렌더링 경로 무관 (PR #571 의 핵심 문제 해결 — 분리 PR 1/3 정합)
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 예상 — TypeScript 영역 영역 SVG 무영향)

### 6.2 신규 API 영역 영역 graceful fallback
- `getTableSignature` / `getParagraphStableId` 영역 영역 try/catch — WASM 미구현 영역 영역 JS 폴백 정합
- → 본 PR 머지 시 WASM 부재 영역 영역도 동작 (PR 2/3 영역 영역 후속 WASM API 도입)

### 6.3 CI 결과 부재
mergeable=CONFLICTING 영역 영역 CI 미실행. 충돌 해결 후 자기 검증 필수.

## 7. 처리 옵션

### 옵션 A — 5 commits 개별 cherry-pick + 충돌 수동 해결 + .actual.svg 제거 + no-ff merge

```bash
git checkout local/devel
git cherry-pick d0457363 0656b889 a3f6ff21 51a22175 238ffd19  # 5 commits + 충돌 수동 해결
# .actual.svg 제거 commit
git rm tests/golden_svg/*/page-*.actual.svg tests/golden_svg/*/aift-page3.actual.svg tests/golden_svg/*/ktx-toc-page.actual.svg
git commit -m "fix: PR #799 정정 — actual.svg 임시 파일 제거"
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash cherry-pick + 충돌 수동 해결 + .actual.svg 제거

```bash
git cherry-pick --no-commit d0457363..238ffd19
# 충돌 + actual.svg 제거 + 단일 commit
```

### 옵션 C — 컨트리뷰터에 정정 요청 후 재시도

`.actual.svg` 제거 + base 갱신 + 충돌 해결 영역 영역 컨트리뷰터에 요청 후 추가 commit 영역 영역 재제출

본 환경 영역 영역 옵션 A 권장 — 컨트리뷰터 commit 이력 보존 + actual.svg 제거 정정.

## 8. 검증 게이트

### 8.1 자기 검증
- [ ] cherry-pick 5 commits + 충돌 수동 해결
- [ ] `.actual.svg` 제거 commit
- [ ] tsc + cargo test ALL GREEN (Rust 변경 부재 영역 영역 cargo 영향 부재)
- [ ] 광범위 sweep 170/170 same (TypeScript 영역 영역 SVG 무영향 자명)

### 8.2 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질 영역 영역 rhwp-studio editor 신규 기능 (Compare + History):
- WASM 빌드 후 dev server 영역 영역:
  - **Alt+Shift+V** — 문서 비교 대화상자 표시 + diff-engine 정렬/비교
  - **Ctrl+Shift+H** — 문서 이력 관리 대화상자 + IndexedDB
  - 메뉴 영역 영역 "편집 → 문서 비교 / 이력 관리"
  - WasmBridge graceful fallback 동작 확인 (getTableSignature / getParagraphStableId 미구현 영역 영역 JS 폴백)
  - 기존 기능 회귀 부재

## 9. 후속 분리 (PR 본문 명시)

**분리 PR 2/3, 3/3** — Rust 렌더링 + 파서 stable_id 영역 영역 별 PR (작업지시자 권장 영역 영역 시각 판정 게이트 영역 영역 분리):
- PR 2 — Rust pagination/typeset 신규 함수 (PR #571 의 페이지네이션 본질)
- PR 3 — paragraph.rs::stable_id parser+WASM 노출 (identity 전략 영역 영역 완성)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @xogh3198 **3번째 시도** (PR #571 → #623 → #799) — 작업지시자 영역 영역 "처음 기여" 영역 영역 사실 부재, 그러나 첫 진정한 머지 시도 |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 (분리 PR 1/3 정합) |
| `feedback_pr_supersede_chain` | PR #571 → #623 → **#799** 동일 본질 영역 영역 점진적 정합 (base skew 해결 + 분리 PR) |
| `feedback_process_must_follow` | 분리 PR 1/3 (TypeScript) + 분리 PR 2/3, 3/3 (Rust 후속) — 권장 가이드 정합 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 신규 기능 영역 영역 작업지시자 인터랙션 검증 권장 |
| `feedback_pr_comment_tone` | 신규 컨트리뷰터 영역 영역 차분 톤 + 정정 요청 (actual.svg 제거 안내) |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 옵션 A cherry-pick (5 commits) + 충돌 수동 해결
2. `.actual.svg` 제거 commit (메인테이너 정정)
3. 자기 검증 (tsc + cargo test + 광범위 sweep)
4. WASM 빌드 + 작업지시자 인터랙션 검증 (Alt+Shift+V / Ctrl+Shift+H + 기존 기능 회귀 부재)
5. 인터랙션 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #571 close
6. PR #799 close + 컨트리뷰터에 actual.svg 정정 안내 + 분리 PR 2/3, 3/3 후속 안내

---

작성: 2026-05-11
