---
PR: #751
제목: fix — 문단 정렬 Alt+Shift 단축키 한국어 IME 매핑 추가 (Part of #223)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 19번째 PR)
처리: 옵션 A — 1 commit cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: cc5177d4
---

# PR #751 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `cc5177d4` (--no-ff merge) |
| Cherry-pick commit | `193579e4` |
| Part of | Issue #223 (macOS 단축키 지원) |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same |

## 2. 정정 본질 — 1 file, +3/-0

`rhwp-studio/src/command/shortcut-map.ts`:

```typescript
[{ key: 'ㅗ', alt: true, shift: true }, 'format:align-right'],
[{ key: 'ㅊ', alt: true, shift: true }, 'format:align-center'],
[{ key: 'ㅇ', alt: true, shift: true }, 'format:align-distribute'],
```

기존 영문 매핑 (Alt+Shift+H/C/D) 직후 IME 매핑 추가.

## 3. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `defaultShortcuts` 한국어 IME 매핑 패턴 | 줄간격 (`ㅁ`/`ㅋ`) + 글꼴 (`ㄷ`/`ㄱ`) + 본 PR 정렬 (`ㅗ`/`ㅊ`/`ㅇ`) 동일 패턴 |
| `format:align-right` / `align-center` / `align-distribute` 커맨드 | 기존 정렬 커맨드 재호출 |

→ 신규 인프라 도입 부재.

## 4. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (1 commit) | ✅ auto-merge 충돌 0건 |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |

## 5. 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과
- 한글 IME 활성 상태 영역 영역 Alt+Shift+ㅗ → 오른쪽 정렬
- Alt+Shift+ㅊ → 가운데 정렬
- Alt+Shift+ㅇ → 배분 정렬

## 6. 영향 범위

### 6.1 변경 영역
- rhwp-studio editor 영역 영역 단축키 매핑 (shortcut-map.ts 영역 3 매핑)

### 6.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- 정렬 커맨드 자체 — 기존 동작
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 19번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (기존 IME 매핑 패턴 + 정렬 커맨드) — 신규 인프라 도입 부재 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과 |

## 8. 잔존 후속

- Issue #223 OPEN 유지 — Cmd+Arrow / Opt+Arrow / 한컴 주요 단축키 영역 의 후속 단계 잔존
- 본 PR 본질 정정의 잔존 결함 부재

---

작성: 2026-05-10
