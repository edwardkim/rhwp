---
PR: #749
제목: Task #223 — Ctrl/Cmd+O 파일 열기 단축키 추가
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 17번째 PR)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: 55cc65d6
---

# PR #749 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `55cc65d6` (--no-ff merge) |
| Cherry-pick commits | `d114a56b` (Task) + `0390b9bb` (Copilot 리뷰) |
| Part of | Issue #223 (macOS 단축키 지원) |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 4.66 MB |

## 2. 정정 본질 — 2 files, +10/-0

### 2.1 `rhwp-studio/src/command/shortcut-map.ts` (+2)

```typescript
[{ key: 'o', ctrl: true }, 'file:open'],
[{ key: 'ㅐ', ctrl: true }, 'file:open'],
```

기존 Alt+N / Alt+ㅜ 패턴 정합.

### 2.2 `rhwp-studio/src/main.ts` `setupGlobalShortcuts()` (+8)

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

Alt+N 핸들러 동일 패턴.

### 2.3 Copilot 리뷰 반영 (commit `0390b9bb`)
한글 IME 매핑 (`Ctrl+ㅐ`) 추가 — Alt+N ↔ ㅜ 패턴 정합.

## 3. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `file:open` 커맨드 (`command/commands/file.ts:117`) | File System Access API + file-input fallback 기존 구현 재호출 |
| `setupGlobalShortcuts` (Alt+N 패턴) | Ctrl+O 핸들러 동일 패턴 추가 |
| `shortcut-map.ts` 한글 IME 매핑 (`ㅜ` 패턴) | `ㅐ` 매핑 동일 정합 |

→ `feedback_process_must_follow` 정합 — 신규 인프라 도입 부재 영역 위험 좁힘.

## 4. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ 충돌 0건 |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (TypeScript 영역 영역 SVG 무영향) |
| WASM 빌드 (Docker) | ✅ 4.66 MB |

## 5. 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과
- Ctrl+O (Windows) / Cmd+O (macOS) → 파일 열기 대화상자
- 한글 IME 입력 상태 영역 영역 Ctrl+ㅐ 동일 동작
- 문서 미로드 상태에서도 동작

## 6. 후속 분리 (PR 본문 명시)

Issue #223 OPEN 유지 — Cmd+Arrow / Opt+Arrow / 한컴 주요 단축키 영역 의 후속 단계:
- PR #746 (Part of #260) 영역 Ctrl/Cmd+Arrow 정합 (5/10 사이클)
- Option+Arrow (단어 단위 이동) 영역 별 후속 PR (Issue #260 OPEN)

## 7. 영향 범위

### 7.1 변경 영역
- rhwp-studio editor 영역 영역 단축키 인터랙션 (shortcut-map + main.ts)

### 7.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 17번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (Alt+N 패턴 + file:open 커맨드) — 위험 좁힘 + 후속 분리 (Cmd+Arrow / Opt+Arrow) 명시 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과 |

## 9. 잔존 후속

- Issue #223 OPEN 유지 — 후속 단축키 단계
- 본 PR 본질 정정의 잔존 결함 부재

---

작성: 2026-05-10
