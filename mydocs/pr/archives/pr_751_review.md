---
PR: #751
제목: fix — 문단 정렬 Alt+Shift 단축키 한국어 IME 매핑 추가 (Part of #223)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 19번째 PR)
base / head: devel / contrib/korean-ime-shortcuts
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: SUCCESS (Build & Test + CodeQL js/ts/py/rust + Canvas visual diff)
변경 규모: +3 / -0, 1 file
검토일: 2026-05-10
---

# PR #751 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #751 |
| 제목 | fix — 문단 정렬 Alt+Shift 단축키 한국어 IME 매핑 추가 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 19번째 PR) |
| base / head | devel / contrib/korean-ime-shortcuts |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 부재 |
| CI | ✅ Build & Test + CodeQL (js/ts/py/rust) + Canvas visual diff 통과 |
| 변경 규모 | +3 / -0, 1 file |
| 커밋 수 | 1 |
| Part of | Issue #223 (macOS 단축키 지원) |

## 2. 결함 본질

문단 정렬 단축키 (Alt+Shift+H/C/D) 영역 영역 한국어 IME 자모 매핑 누락 — 한글 입력 상태 영역 영역 오른쪽/가운데/배분 정렬 단축키 미동작.

기존 매핑과 비교:
- ✅ 줄간격: Alt+Shift+A → ㅁ / Alt+Shift+Z → ㅋ
- ✅ 글꼴 크기: Alt+Shift+E → ㄷ / Alt+Shift+R → ㄱ
- ❌ 정렬 (본 PR 정정 영역): Alt+Shift+H → ㅗ / Alt+Shift+C → ㅊ / Alt+Shift+D → ㅇ 매핑 부재

기존 패턴 정합 — `feedback_process_must_follow` 정합.

## 3. PR 의 정정 — 1 file, +3/-0

```typescript
[{ key: 'ㅗ', alt: true, shift: true }, 'format:align-right'],
[{ key: 'ㅊ', alt: true, shift: true }, 'format:align-center'],
[{ key: 'ㅇ', alt: true, shift: true }, 'format:align-distribute'],
```

기존 영문 매핑 직후 IME 매핑 추가.

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `defaultShortcuts` 한국어 IME 매핑 패턴 | 줄간격 (`ㅁ`/`ㅋ`) + 글꼴 (`ㄷ`/`ㄱ`) + 본 PR 정렬 (`ㅗ`/`ㅊ`/`ㅇ`) 동일 패턴 |
| `format:align-right` / `format:align-center` / `format:align-distribute` 커맨드 | 기존 정렬 커맨드 재호출 |

→ 신규 인프라 도입 부재.

## 5. 충돌 / mergeable

mergeStateStatus = `BEHIND` (devel 영역 누적 변경), mergeable = `MERGEABLE`.

본 환경 점검:
- `rhwp-studio/src/command/shortcut-map.ts` — devel 5/10 사이클 영역:
  - PR #749 (5/10): Ctrl+O / Ctrl+ㅐ → file:open 추가
  - PR #750 (5/10): Ctrl+Alt+Enter → page:col-settings 추가
  - 본 PR: Alt+Shift+ㅗ/ㅊ/ㅇ 추가 — 정렬 영역 영역 다른 영역 영역 충돌 부재

→ cherry-pick 충돌 0건 예상 (auto-merge 정합).

## 6. 본 환경 점검

### 6.1 변경 격리
- TypeScript 단일 파일 영역 영역 매핑 3개 추가
- Rust / WASM / 렌더링 경로 무관 (`feedback_image_renderer_paths_separate` 정합)

### 6.2 CI 결과
- Build & Test ✅
- CodeQL (js/ts/py/rust) ✅
- Canvas visual diff ✅
- WASM Build SKIPPED (CI 영역 자동 판정 — 변경 무관)

## 7. 처리 옵션

### 옵션 A — 1 commit cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick a5fcb3dd
git checkout devel
git merge local/devel --no-ff -m "Merge PR #751 (Part of #223): 정렬 단축키 한국어 IME 매핑 추가"
```

→ **권장**.

## 8. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] cherry-pick 충돌 0건
- [ ] `cargo build --release` 통과 (Rust 변경 부재)
- [ ] `cargo test --release` ALL GREEN
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (TypeScript 영역 SVG 무영향 자명)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질은 **rhwp-studio editor 한국어 IME 단축키**:
- WASM 빌드 후 dev server 영역 영역:
  - 한글 IME 활성 상태 영역 영역 Alt+Shift+ㅗ → 오른쪽 정렬
  - Alt+Shift+ㅊ → 가운데 정렬
  - Alt+Shift+ㅇ → 배분 정렬
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

> 매우 단순한 변경 (3 매핑 라인) — 시각 판정 면제 가능 영역 영역 작업지시자 결정.

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 19번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 — Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (기존 IME 매핑 패턴 + 정렬 커맨드) — 신규 인프라 도입 부재 |
| `feedback_visual_judgment_authority` | 매우 단순 변경 영역 영역 시각 판정 게이트 면제 가능 (작업지시자 결정) |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 옵션 A cherry-pick (1 commit)
2. 자기 검증 (cargo test + tsc + 광범위 sweep)
3. (선택) WASM 빌드 + 작업지시자 인터랙션 검증 — 매우 단순 변경 영역 영역 면제 가능
4. 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #751 close (Issue #223 OPEN 유지 — Cmd+Arrow / Opt+Arrow 등 후속 단계 잔존)

---

작성: 2026-05-10
