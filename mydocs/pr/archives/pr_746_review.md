---
PR: #746
제목: Task #260 — Ctrl/Cmd+Arrow 커서 이동 (줄/문서 시작·끝)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 15번째 PR)
base / head: devel / contrib/ctrl-arrow-navigation
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS
변경 규모: +33 / -1, 1 file
검토일: 2026-05-10
---

# PR #746 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #746 |
| 제목 | Task #260 — Ctrl/Cmd+Arrow 커서 이동 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 15번째 PR) |
| base / head | devel / contrib/ctrl-arrow-navigation |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS |
| 변경 규모 | +33 / -1, 1 file |
| 커밋 수 | 2 (Task + Copilot 리뷰) |
| Part of | #260 |

## 2. 결함 본질 (Issue #260)

macOS 영역 영역 `Cmd+Arrow` 조합 미처리 → 줄 시작/끝, 문서 시작/끝 영역 영역 이동 부재. Windows 영역 영역 `Ctrl+Arrow` 동일.

### 2.1 채택 접근

기존 `Ctrl+Home/End` 패턴 정합 — `handleCtrlKey()` 영역 영역 Arrow 키 처리 추가. 인프라 (cursor 메서드) 재사용.

| 키 조합 | 동작 | 기존 대응 키 |
|---------|------|-------------|
| Ctrl/Cmd + ← | 줄 시작 | Home |
| Ctrl/Cmd + → | 줄 끝 | End |
| Ctrl/Cmd + ↑ | 문서 시작 | Ctrl+Home |
| Ctrl/Cmd + ↓ | 문서 끝 | Ctrl+End |

모든 방향에 `Shift` 조합으로 선택 범위 확장 지원.

## 3. PR 의 정정 — `rhwp-studio/src/engine/input-handler-keyboard.ts` (+33/-1)

```typescript
case 'arrowleft': {
    e.preventDefault();
    if (e.shiftKey) this.cursor.setAnchor();
    else this.cursor.clearSelection();
    this.cursor.moveToLineStart();
    this.updateCaret();
    break;
}
// arrowright / arrowup / arrowdown 동일 패턴
```

### 3.1 Copilot 리뷰 반영 (commit `c99e4923`)
- `updateSelection()` 중복 호출 제거

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `handleCtrlKey()` 패턴 (Ctrl+Home/End) | Arrow 키 처리 동일 패턴 |
| `cursor.moveToLineStart` / `moveToLineEnd` | 기존 메서드 재호출 |
| `cursor.moveToDocumentStart` / `moveToDocumentEnd` | 기존 메서드 재호출 |
| `cursor.setAnchor` / `clearSelection` | Shift 영역 영역 선택 범위 확장 |

→ `feedback_process_must_follow` 정합 — 신규 인프라 도입 부재.

## 5. 후속 분리 (PR 본문 명시)

- **본 PR 영역 영역 Issue #260 의 첫 단계** — Ctrl/Cmd+Arrow 만 구현
- **Option+Arrow** (단어 단위 이동) — 단어 경계 감지 로직 필요 영역 영역 후속 작업

## 6. 본 환경 점검

- merge-base: `30351cdf` (5/9 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 격리: TypeScript 단일 파일 — Rust/렌더링 경로 무관

## 7. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 5/10 사이클 진전, 본 PR 단일 파일 영역 영역 충돌 부재

## 8. 처리 옵션

### 옵션 A — 2 commits cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task260 3159c575
git cherry-pick c482970f c99e4923
git checkout local/devel
git merge --no-ff local/task260
```

→ **옵션 A 추천**.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과 (Rust 변경 부재 영역 영역 영향 없음)
- [ ] `cargo test --release` ALL GREEN
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (TypeScript 영역 영역 SVG 무영향 보장)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질은 **rhwp-studio editor 키보드 인터랙션**:
- WASM 빌드 후 dev server 영역 영역 4가지 Arrow 조합 + Shift 조합 점검
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 15번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 파일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (handleCtrlKey 패턴 + cursor 메서드) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 + 후속 분리 (Option+Arrow) 명시 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 작업지시자 인터랙션 검증 권장 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test + tsc + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (4가지 Arrow + Shift 조합)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #746 close (Part of #260 — 후속 작업 영역 영역 issue 유지)

---

작성: 2026-05-10
