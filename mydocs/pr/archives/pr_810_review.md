---
PR: #810
제목: feat — 메뉴 열린 상태 단일 키 hotkey 항목 활성 인프라 (closes #792)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 10번째 PR
base / head: devel / contrib/menu-hotkey-infra
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +26 / -2, 1 file
커밋: 2
검토일: 2026-05-11
---

# PR #810 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #810 |
| 제목 | feat: 메뉴 열린 상태 단일 키 hotkey 항목 활성 인프라 (#792) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 핵심 (5/11 사이클 **10번째 PR** — #786 → #787 → #788 → #794 → #795 → #796 → #807 → #808 → #809 → **#810**) |
| base / head | devel / contrib/menu-hotkey-infra |
| mergeable | MERGEABLE (BEHIND — base 갱신만) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +26 / -2, 1 file |
| 커밋 수 | 2 (1 본질 + 1 Copilot 리뷰 반영) |
| closes | #792 |
| 관련 | PR #758 (5/10 머지, H/W shortcutLabel 도입) — 본 PR 영역 그 단축키 영역 영역 메뉴 hotkey 인프라 정합 |

## 2. 본질 (Issue #792)

PR #758 영역 영역 `table:cell-height-equal` (H) + `table:cell-width-equal` (W) 영역 영역 `shortcutLabel` 영역 영역 한컴 메뉴 hotkey 의도 — 그러나 rhwp-studio 영역 영역 메뉴 hotkey 인프라 미구현 영역 영역 단축키 미동작.

### 메뉴 hotkey 메커니즘 (한컴 표준)
- 메뉴 열린 상태 영역 영역 단일 키 (modifier 없음) → 해당 라벨 영역 영역 항목 활성
- 예: "표" 메뉴 열린 후 `H` → "셀 높이를 같게" 항목 활성 + 메뉴 닫기

### 본 환경 두 종류 단축키 메커니즘 (Issue #792 명시)
| 영역 | 의미 | 본 환경 처리 |
|------|------|------------|
| `shortcutLabel` (UI 라벨) | command-palette / context-menu 영역 표시만 | 본 PR 영역 영역 hotkey 매칭 추가 |
| `shortcut-map.ts` (전역) | InputHandler 영역 영역 dispatch (Ctrl/Alt/Shift) | 본 PR 무관 |

## 3. 정정 본질 — `menu-bar.ts` +26/-2 (1 file)

### 3.1 `setupKeyboardClose` 영역 영역 hotkey 매칭 추가
기존: Escape 키 영역 영역만 처리 (메뉴 닫기).
정정: 메뉴 열린 상태 영역 영역 modifier 없는 단일 키 영역 영역 `.md-shortcut` 텍스트 매칭.

```typescript
private setupKeyboardClose(): void {
  document.addEventListener('keydown', (e) => {
    if (!this.openMenu) return;
    if (e.key === 'Escape') {
      this.closeAll();
      return;
    }
    // 메뉴 열린 상태에서 단일 키 (modifier 없음) → shortcutLabel 매칭
    if (e.ctrlKey || e.altKey || e.metaKey || e.shiftKey) return;
    if (e.key.length !== 1) return;
    const key = e.key.toUpperCase();
    const items = this.openMenu.querySelectorAll('.md-item[data-cmd]:not(.disabled)');
    for (const item of items) {
      const shortcut = item.querySelector('.md-shortcut');
      if (shortcut && shortcut.textContent?.toUpperCase() === key) {
        e.preventDefault();
        const el = item as HTMLElement;
        const cmd = el.dataset.cmd;
        if (cmd) {
          const params: Record<string, unknown> = { anchorEl: item };
          for (const [k, v] of Object.entries(el.dataset)) {
            if (k !== 'cmd') params[k] = v;
          }
          this.dispatcher.dispatch(cmd, params);
        }
        this.closeAll();
        return;
      }
    }
  });
}
```

### 3.2 제약 조건 (영역 좁힘)
- **modifier 키 동반 시 무시** — 전역 단축키 (`shortcut-map.ts` 영역) 영역 영역 충돌 방지
- **단일 문자 키만** (`e.key.length === 1`) — Tab/Arrow 등 특수 키 제외
- **disabled 항목 제외** (`:not(.disabled)` selector)
- **대소문자 무시** (`.toUpperCase()` 비교)

### 3.3 Copilot 리뷰 반영 commit (`b65fefd8`)
hotkey 경로 영역 영역 `data-*` params 전달 — 메뉴 클릭 경로 영역 영역 동일 params 전달 영역 영역 일관성 (예: 표 영역 영역 셀 정보 등 클릭/hotkey 영역 영역 동일 dispatch 결과).

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

→ 신규 인프라 도입 부재 — 기존 menu-bar 구조 영역 영역 hotkey 매칭 로직 추가만.

## 5. 본 환경 점검

### 5.1 변경 격리
- **순수 TypeScript `menu-bar.ts` 단일** — WASM/Rust 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 무영향 자명)
- 다른 컴포넌트 변경 부재 — 본 영역 영역만 격리

### 5.2 CI 통과
- ✅ Build & Test
- ✅ CodeQL (js-ts / python / rust)
- ✅ Canvas visual diff

### 5.3 mergeStateStatus = BEHIND
base 갱신만 필요 — 충돌 부재 (MERGEABLE). menu-bar.ts 영역 영역 devel 최근 변경 부재 영역 영역 cherry-pick auto-merge 예상.

## 6. 영역 좁힘 (회귀 부재 가드)

- modifier 키 동반 시 무시 — `Ctrl+H` 등 전역 단축키 영역 충돌 부재
- 단일 문자 키만 — Tab/Arrow/Function 키 영역 회귀 부재
- 메뉴 닫힌 상태 (`!this.openMenu`) 영역 early return — 일반 텍스트 입력 영역 영향 부재
- disabled 항목 제외 — 비활성 영역 영역 실행 부재
- Escape 영역 영역 기존 동작 보존 (early return + closeAll)

## 7. 처리 옵션

### 옵션 A (권장) — 2 commits cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick 22dba189 b65fefd8
git checkout devel
git merge local/devel --no-ff
```

본질 commit + Copilot 리뷰 반영 commit 영역 이력 보존.

### 옵션 B — squash cherry-pick (단일 commit)

본 환경 영역 영역 commit 이력 보존 권장 옵션 A.

## 8. 검증 게이트

### 8.1 자기 검증
- [ ] cherry-pick 2 commits (auto-merge 예상)
- [ ] tsc --noEmit
- [ ] cargo test (Rust 변경 부재 영역 영역 회귀 자명, 형식 점검)
- [ ] WASM 재빌드 불필요 (TypeScript 단일)

### 8.2 시각/인터랙션 판정 게이트 — **작업지시자 인터랙션 검증 권장**
- 메뉴 영역 영역 표 메뉴 열기 → `H` 키 → "셀 높이를 같게" 활성 + 메뉴 닫힘
- 메뉴 영역 영역 표 메뉴 열기 → `W` 키 → "셀 너비를 같게" 활성 + 메뉴 닫힘
- 메뉴 닫힌 상태 영역 영역 일반 텍스트 입력 (`H` / `W`) — 회귀 부재 (메뉴 비활성 영역 영역 텍스트 입력 정합)
- 메뉴 열린 상태 영역 영역 `Ctrl+H` — 전역 단축키 영역 동작 (메뉴 hotkey 영역 충돌 부재)
- 메뉴 영역 영역 disabled 항목 hotkey — 실행 부재
- Escape — 기존 동작 (메뉴 닫기) 보존

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 10번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (기존 menu-bar 구조 + dispatcher) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | modifier/length/disabled 3중 가드 영역 영역 영역 좁힘 — 전역 단축키 충돌 부재 |
| `feedback_diagnosis_layer_attribution` | shortcutLabel UI 라벨 vs 메뉴 hotkey 모달 경로 vs 전역 shortcut-map 세 본질 분리 (Issue #792 명시) |
| `feedback_visual_judgment_authority` | 메뉴 hotkey 영역 영역 작업지시자 인터랙션 검증 권장 |
| `feedback_pr_supersede_chain` | PR #758 (H/W shortcutLabel 도입, 단축키 미동작 발견) → **PR #810** (메뉴 hotkey 인프라 정합) — (c) 패턴 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick 2 commits (`22dba189` + `b65fefd8`)
2. 자기 검증 (tsc + cargo test)
3. 작업지시자 웹 에디터 인터랙션 검증 (표 메뉴 H/W + 전역 단축키 회귀 부재)
4. 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #792 close
5. PR #810 close

---

작성: 2026-05-11
