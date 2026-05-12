---
PR: #810
제목: feat — 메뉴 열린 상태 단일 키 hotkey 항목 활성 인프라 (closes #792)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 10번째 PR
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-11
머지 commit: e1be8d9e
---

# PR #810 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `e1be8d9e` (--no-ff merge) |
| Cherry-pick commits | 2 (본질 + Copilot 리뷰 반영) |
| closes | #792 |
| 시각 판정 | ✅ 작업지시자 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN |
| WASM 재빌드 | 불필요 (TypeScript 단일 변경) |

## 2. 본질 (Issue #792)

PR #758 (5/10 머지) 영역 `table:cell-height-equal` (H) + `table:cell-width-equal` (W) 의 `shortcutLabel` 영역 한컴 메뉴 hotkey 의도 — 그러나 rhwp-studio 영역 영역 메뉴 hotkey 인프라 미구현 → 본 PR 영역 정합.

### 메뉴 hotkey 메커니즘 (한컴 표준)
메뉴 열린 상태 영역 영역 단일 키 (modifier 없음) → 해당 라벨 영역 영역 항목 활성. 예: "표" 메뉴 열린 후 `H` → "셀 높이를 같게" 활성 + 메뉴 닫기.

### 세 본질 분리 (Issue #792 명시)
| 영역 | 의미 | 본 환경 처리 |
|------|------|------------|
| `shortcutLabel` (UI 라벨) | command-palette / context-menu 영역 표시 | 본 PR 영역 영역 hotkey 매칭 추가 |
| `shortcut-map.ts` (전역) | InputHandler 영역 영역 dispatch (Ctrl/Alt/Shift) | 본 PR 무관 |
| 메뉴 hotkey (모달) | 메뉴 열린 상태 영역 영역만 활성 | 본 PR 영역 영역 신규 |

## 3. 정정 본질 — `menu-bar.ts` +26/-2 (1 file)

### 3.1 `setupKeyboardClose` 영역 영역 hotkey 매칭 추가
기존: Escape 키만 처리 (메뉴 닫기).
정정: 메뉴 열린 상태 영역 영역 modifier 없는 단일 키 영역 `.md-shortcut` 텍스트 매칭.

### 3.2 제약 조건 (영역 좁힘)
- modifier 키 동반 시 무시 (`Ctrl/Alt/Meta/Shift`) — 전역 단축키 충돌 방지
- 단일 문자 키만 (`e.key.length === 1`) — Tab/Arrow/Function 키 제외
- disabled 항목 제외 (`:not(.disabled)` selector)
- 대소문자 무시 (`.toUpperCase()` 비교)
- 메뉴 닫힌 상태 (`!this.openMenu`) early return

### 3.3 Copilot 리뷰 반영 commit (`3cdf7a6b`)
hotkey 경로 영역 영역 `data-*` params 전달 — 메뉴 클릭 경로 영역 동일 params 일관성:
```typescript
const params: Record<string, unknown> = { anchorEl: item };
for (const [k, v] of Object.entries(el.dataset)) {
  if (k !== 'cmd') params[k] = v;
}
this.dispatcher.dispatch(cmd, params);
```

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `MenuBar.openMenu` (기존) | 메뉴 열린 상태 점검 |
| `.md-item[data-cmd]` / `.md-shortcut` (기존 DOM 구조) | 매칭 selector |
| `dispatcher.dispatch(cmd, params)` (기존) | 커맨드 호출 |
| `closeAll()` (기존) | 메뉴 닫기 |

→ 신규 인프라 도입 부재 — 기존 menu-bar 구조 영역 hotkey 매칭 로직 추가만.

## 5. 영역 좁힘 (회귀 부재 가드)

- modifier 키 동반 시 무시 — `Ctrl+H` 등 전역 단축키 충돌 부재
- 단일 문자 키만 — Tab/Arrow/Function 키 회귀 부재
- 메뉴 닫힌 상태 early return — 일반 텍스트 입력 영향 부재
- disabled 항목 제외 — 비활성 영역 실행 부재
- Escape 기존 동작 보존

## 6. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 2 commits | ✅ auto-merge 충돌 0건 |
| `tsc --noEmit` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep | 면제 (TypeScript 단일 영역 SVG 무영향 자명) |
| WASM 재빌드 | 불필요 |

## 7. 작업지시자 인터랙션 검증 ✅ 통과

- 표 메뉴 열기 → `H` → "셀 높이를 같게" 활성 + 메뉴 닫힘
- 표 메뉴 열기 → `W` → "셀 너비를 같게" 활성 + 메뉴 닫힘
- 메뉴 닫힌 상태 영역 일반 텍스트 입력 (`H` / `W`) — 회귀 부재
- 메뉴 열린 상태 영역 `Ctrl+H` — 전역 단축키 동작 (메뉴 hotkey 충돌 부재)
- disabled 항목 hotkey — 실행 부재
- Escape — 기존 동작 (메뉴 닫기) 보존

## 8. CI 통과

✅ Build & Test + CodeQL (js-ts / python / rust) + Canvas visual diff (PR 머지 전 검증)

## 9. 영향 범위

### 9.1 변경 영역
- `rhwp-studio/src/ui/menu-bar.ts` (+26/-2)

### 9.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (TypeScript 단일 영역 SVG 무영향)
- 전역 `shortcut-map.ts` (별 메커니즘)
- 다른 컴포넌트

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 10번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (MenuBar 구조 + dispatcher) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | modifier/length/disabled 3중 가드 영역 영역 좁힘 — 전역 단축키 충돌 부재 |
| `feedback_diagnosis_layer_attribution` | shortcutLabel UI vs 메뉴 hotkey 모달 vs 전역 shortcut-map 세 본질 분리 (Issue #792 명시) |
| `feedback_visual_judgment_authority` | 작업지시자 인터랙션 검증 ✅ 통과 |
| `feedback_pr_supersede_chain` | PR #758 (H/W shortcutLabel 도입, 단축키 미동작 발견) → **PR #810** (메뉴 hotkey 인프라 정합) — (c) 패턴 |
| `feedback_small_batch_release_strategy` | 신규 인프라 (opt-in, 메뉴 닫힌 상태 영향 부재) 영역 PATCH cycle 머지 정합 |

## 11. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #792 close 완료
- 다른 메뉴 항목 영역 영역 `shortcutLabel` 영역 영역 동일 패턴 영역 자동 정합 (본 PR 인프라 활용)

---

작성: 2026-05-11
