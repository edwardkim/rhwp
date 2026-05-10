---
PR: #746
제목: Task #260 — Ctrl/Cmd+Arrow 커서 이동 (줄/문서 시작·끝)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 15번째 PR)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: (다음 commit 영역 영역 점검)
---

# PR #746 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `20c0e5d4` (--no-ff merge) |
| Cherry-pick commits | `0dc16331` + `ad7e6b4d` (Copilot 리뷰) |
| Part of | Issue #260 (macOS 커서 이동 UX) |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 4.66 MB |

## 2. 정정 본질 — 1 file, +33/-1

`rhwp-studio/src/engine/input-handler-keyboard.ts` `handleCtrlKey()` 영역 영역 Arrow 키 처리 추가:

| 키 조합 | 동작 |
|---------|------|
| Ctrl/Cmd + ← / → | 줄 시작 / 끝 (`moveToLineStart` / `moveToLineEnd`) |
| Ctrl/Cmd + ↑ / ↓ | 문서 시작 / 끝 (`moveToDocumentStart` / `moveToDocumentEnd`) |

모든 방향에 `Shift` 조합으로 선택 범위 확장 (`setAnchor` / `clearSelection`).

## 3. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `handleCtrlKey()` (Ctrl+Home/End 패턴) | Arrow 키 처리 동일 패턴 |
| `cursor.moveTo*` (기존 메서드) | 재호출 |
| `cursor.setAnchor` / `clearSelection` | Shift 영역 영역 선택 범위 확장 |

→ 신규 인프라 도입 부재 영역 영역 위험 좁힘 (`feedback_process_must_follow` 정합).

## 4. Copilot 리뷰 반영 (commit `ad7e6b4d`)
`updateSelection()` 중복 호출 제거.

## 5. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ 충돌 0건 |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (TypeScript 영역 영역 SVG 무영향) |
| WASM 빌드 (강제 재빌드) | ✅ 4.66 MB |

## 6. 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과
4가지 Arrow + Shift 조합 정합 동작.

## 7. 후속 분리 (PR 본문 명시)

- Issue #260 의 첫 단계 — 본 PR 영역 영역 Ctrl/Cmd+Arrow 만
- **Option+Arrow** (단어 단위 이동) — 단어 경계 감지 로직 필요 영역 영역 후속 작업

## 8. 영향 범위

### 8.1 변경 영역
- rhwp-studio editor 영역 영역 Ctrl/Cmd+Arrow 키보드 인터랙션

### 8.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 영역 영역 15번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 파일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (handleCtrlKey 패턴 + cursor 메서드) — 위험 좁힘 + 후속 분리 (Option+Arrow) 명시 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과 |

## 10. 잔존 후속

- Issue #260 OPEN 유지 — Option+Arrow 후속 작업 영역 영역
- 본 PR 본질 정정의 잔존 결함 부재

---

작성: 2026-05-10
