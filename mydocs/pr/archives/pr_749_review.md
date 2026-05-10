---
PR: #749
제목: Task #223 — Ctrl/Cmd+O 파일 열기 단축키 추가
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 17번째 PR)
base / head: devel / contrib/keyboard-shortcuts-extend
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: SUCCESS (Build & Test + CodeQL js/ts/py/rust + Canvas visual diff)
변경 규모: +10 / -0, 2 files
검토일: 2026-05-10
---

# PR #749 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #749 |
| 제목 | Task #223 — Ctrl/Cmd+O 파일 열기 단축키 추가 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 17번째 PR) |
| base / head | devel / contrib/keyboard-shortcuts-extend |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 부재 |
| CI | ✅ Build & Test + CodeQL (js/ts/py/rust) + Canvas visual diff 통과 |
| 변경 규모 | +10 / -0, 2 files |
| 커밋 수 | 2 (Task + Copilot 리뷰) |
| Part of | Issue #223 (macOS 단축키 지원) |

## 2. 결함 본질 (Issue #223)

`Ctrl+O (Windows)` / `Cmd+O (macOS)` 단축키로 파일 열기 미지원. 문서 편집기 표준 단축키 부재.

### 2.1 채택 접근 — Alt+N (새 문서) 패턴 정합

기존 `setupGlobalShortcuts` 의 Alt+N (새 문서) 패턴 동일 적용:
- 문서 미로드 상태에서도 동작 (InputHandler 비활성 시점)
- 한글 IME 매핑 (`O` → `ㅐ`) 정합 — Alt+N 의 `N` → `ㅜ` 매핑과 동일 패턴
- shortcut-map 등록 + `setupGlobalShortcuts` 핸들러 직접 처리 이중 배선

### 2.2 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `file:open` 커맨드 (`command/commands/file.ts:117`) | File System Access API + file-input fallback 기존 구현 재호출 |
| `setupGlobalShortcuts` (Alt+N 패턴) | Ctrl+O 핸들러 동일 패턴 추가 |
| `shortcut-map.ts` 한글 IME 매핑 (`ㅜ` 패턴) | `ㅐ` 매핑 동일 정합 |

→ `feedback_process_must_follow` 정합 — 신규 인프라 도입 부재.

## 3. PR 의 정정 — 2 files, +10/-0

### 3.1 `rhwp-studio/src/command/shortcut-map.ts` (+2)

```typescript
[{ key: 'o', ctrl: true }, 'file:open'],
[{ key: 'ㅐ', ctrl: true }, 'file:open'],
```

기존 Alt+N / Alt+ㅜ 패턴 정합 — Ctrl+O / Ctrl+ㅐ.

### 3.2 `rhwp-studio/src/main.ts` `setupGlobalShortcuts()` (+8)

```typescript
// Ctrl/Cmd+O → 열기 (문서 미로드 상태에서도 동작)
if (ctrlOrMeta && !e.altKey && !e.shiftKey) {
  if (e.key === 'o' || e.key === 'O' || e.key === 'ㅐ') {
    e.preventDefault();
    dispatcher.dispatch('file:open');
    return;
  }
}
```

Alt+N 핸들러 (line 215-221) 동일 패턴.

### 3.3 Copilot 리뷰 반영 (commit `a246d962`)
한글 IME 매핑 (`Ctrl+ㅐ`) 추가 — Alt+N ↔ ㅜ 패턴 정합.

## 4. 충돌 / mergeable

mergeStateStatus = `BEHIND` (devel 영역 변경 누적), mergeable = `MERGEABLE` — base 비교 시점 이후 devel 영역 누적 변경 영역 영역 동일 파일 충돌 부재.

본 환경 점검:
- `rhwp-studio/src/command/shortcut-map.ts` — devel 영역 5/10 사이클 영역 영역 변경 부재 (PR #746/#748 영역 input-handler 영역, shortcut-map 무관)
- `rhwp-studio/src/main.ts` — `setupGlobalShortcuts` 영역 변경 부재 (Alt+N 패턴 안정 보존)

→ cherry-pick 충돌 0건 예상.

## 5. 본 환경 점검

### 5.1 변경 격리
- TypeScript 단일 영역 (rhwp-studio editor)
- Rust / WASM / 렌더링 경로 무관 (`feedback_image_renderer_paths_separate` 정합)

### 5.2 CI 결과
- Build & Test ✅
- CodeQL (js/ts/py/rust) ✅
- Canvas visual diff ✅
- WASM Build SKIPPED (변경 무관)

### 5.3 Issue #223 의 위치
Issue #223 = "macOS 단축키 지원" 의 큰 우산 — Cmd+Arrow / Opt+Arrow / 한컴 주요 단축키 영역 의 점진적 추가. 본 PR 영역 영역 Ctrl/Cmd+O 만 영역 영역 부분 정정 (PR 본문 명시: "Issue #223 의 일부").

## 6. 처리 옵션

### 옵션 A — 2 commits 개별 cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick 154ea229 a246d962
git checkout devel
git merge local/devel --no-ff -m "Merge PR #749 (Part of #223): Ctrl/Cmd+O 파일 열기 단축키 추가"
```

→ **권장**.

## 7. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] cherry-pick 충돌 0건
- [ ] `cargo build --release` 통과 (Rust 변경 부재)
- [ ] `cargo test --release` ALL GREEN
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (TypeScript 영역 영역 SVG 무영향 입증)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질은 **rhwp-studio editor 단축키 인터랙션**:
- WASM 빌드 후 dev server 영역 영역:
  - `Ctrl+O` (Windows) / `Cmd+O` (macOS) → 파일 열기 대화상자
  - 한글 IME 입력 상태 영역 영역 `Ctrl+ㅐ` 동일 동작
  - 문서 미로드 상태에서도 동작
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 17번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (Alt+N 패턴 + file:open 커맨드) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 작업지시자 인터랙션 검증 권장 |

## 9. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 옵션 A cherry-pick (2 commits)
2. 자기 검증 (cargo test + tsc + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (Ctrl+O / Cmd+O / Ctrl+ㅐ + 문서 미로드 상태)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #749 close (Issue #223 OPEN 유지 — Cmd+Arrow / Opt+Arrow 등 후속 단계 잔존)

---

작성: 2026-05-10
