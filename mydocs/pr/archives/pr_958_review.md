---
PR: #958
제목: fix — 빈 caption phantom advance 정정 (sample16 page 18 본문 다음 페이지 밀림 해소, closes #957)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 2번째)
base / head: devel / local/task957
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +666 / -1, 9 files (코드 1 / 문서 8)
커밋: 1
검토일: 2026-05-18
---

# PR #958 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #958 |
| 제목 | fix: 빈 caption phantom advance 정정 (sample16 page 18 본문 밀림 해소, closes #957) |
| 컨트리뷰터 | @jangster77 — **24+ 사이클** (연속 5 PR **2번째**, #956 직후) |
| base / head | devel / local/task957 |
| mergeable | CONFLICTING (DIRTY — `orders/20260517.md` 1건 충돌) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +666 / -1, 9 files (코드 1 / 문서 8) |
| 커밋 수 | 1 (`38fb2728`) |
| closes | #957 (Issue #952 영역 영역 Issue 2 분리 task) |
| 연속 5 PR | #956 ✅ → **#958 (2번째)** → #961 → #963 → #964 |

## 2. 본질 (Issue #957 = Issue #952 Issue 2)

`samples/hwp3-sample16.hwp` page 18 "나. 주요 과업내용" 후 본문 (pi=395~401
"○ 통합모델...") 이 다음 페이지로 밀려 시각 누락. 한컴 viewer 는 같은 페이지 표시.

### Root cause (RHWP_DEBUG_TAC_CURSOR 추적)
```
Shape pi=394 ci=1 y_in=767.3 y_out=1197.9 dy=430.6 ⚠️
```

`layout.rs:3470-3475` 영역 영역 pi=394 ci=1 picture 의 **빈 caption**
(`dir=Bottom width=0 paras=1 text=""`) 영역 영역 phantom +430.6px 누적:
- pic_y = 767.3 (has_prior_tac 로 갱신된 잘못된 para_start_y)
- image_bottom = 767.3 + 411.89 = 1179.19
- cap_y = 1179.19, caption_h = 18.7 (empty paragraph default)
- cap_bottom = 1197.89 → result_y advance (phantom)
- pi=395 가 1197.89 부터 시작 → body 외 영역 emit → 다음 페이지 밀림

## 3. 정정 본질 — `src/renderer/layout.rs` (caption advance 가드)

```rust
// [Task #957] 빈 caption (text 없음 + controls 없음) 은 SVG 에 invisible.
// pic_y 가 has_prior_tac 로 후속 위치로 갱신되면 image_bottom 이 페이지 바깥
// 위치로 계산되어 result_y 가 phantom +caption_h 누적 → 후속 paragraph 밀림.
let caption_is_empty = caption.paragraphs.iter().all(|p|
    p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}')
        && p.controls.is_empty()
);
if !caption_is_empty && matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    if cap_bottom > result_y { result_y = cap_bottom; }
}
```

- `caption_is_empty` 가드 — caption 의 모든 paragraph 영역 영역 무의미 문자
  (≤U+001F control char, U+FFFC object replacement) + `controls.is_empty()`
- 빈 caption 영역 영역 `result_y` advance skip → phantom +430.6px 제거
- `RHWP_DEBUG_TAC_CURSOR` 진단 영구화 (PR #956 `RHWP_DEBUG_PAGE_BORDER` 패턴 정합)

## 4. 영역 좁힘 (PR 본문 명시)

| 영역 | 영향 |
|------|------|
| Empty caption picture | result_y advance 제거 (회귀 fix) |
| Non-empty caption picture | 영향 없음 (caption_is_empty false) |
| Caption None | 영향 없음 |
| Caption Top direction | 영향 없음 (Bottom 한정) |

→ 빈 caption + Bottom 한정 — `feedback_hancom_compat_specific_over_general` 정합
(케이스별 명시 가드, 일반화 없음).

## 5. 본 환경 충돌 분석

| 파일 | 충돌 | 정합 전략 |
|------|------|----------|
| `mydocs/orders/20260517.md` | changed in both | 본 환경 PR #956 처리 섹션 + PR #958 컨트리뷰터 작업 일지 — 양측 보존 |
| `src/renderer/layout.rs` | 충돌 없음 | PR #956 (page border :762) + PR #958 (caption :3467) 다른 영역 → auto-merge |
| `mydocs/plans/task_m100_957*` 등 8 | added in remote | 신규 추가 (충돌 없음) |

→ cherry-pick 시 `orders/20260517.md` 1건 수동 해결 (양측 보존), layout.rs auto-merge 예상.

## 6. 본 환경 점검

### 6.1 PR #956 정합
PR #956 (Issue #952 Issue 1, 머지 `b31e38ff`) 영역 영역 layout.rs:762 page border 정정.
본 PR 영역 영역 layout.rs:3467 caption advance — 다른 영역, 무관. Issue #952 영역 영역
Issue 2 (본 PR) 영역 영역 분리 task #957 정합.

### 6.2 CI 통과
- ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff

### 6.3 검증 (PR 본문)
- cargo test --release --lib: 1288 passed, 0 failed
- sample16 page 18: pi=395~401 본문 같은 페이지 정상 emit ✓ 한컴 viewer page 16 정합
- hwp3-sample14 (non-empty caption "Cut&Paste 할 영역" + empty 다수): 정상
- hwp3-sample10/11/13, exam_kor/math: 회귀 없음

## 7. 처리 옵션

### 옵션 A (권장) — 1 commit cherry-pick + 충돌 수동 해결 + 자기 검증 + WASM 재빌드

```bash
git checkout local/devel
git cherry-pick 38fb2728
# orders/20260517.md 충돌 수동 해결 (양측 보존)
# cargo test + 광범위 sweep (typeset cursor 변경 → sweep 필수)
# WASM 재빌드 (layout.rs 변경)
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash (이미 1 commit, A 정합)

## 8. 검증 게이트

### 8.1 자기 검증
- [ ] cherry-pick 1 commit + orders 충돌 수동 해결 (양측 보존)
- [ ] cargo test --release --lib ALL GREEN (PR 본문 1288 passed)
- [ ] cargo clippy --release -- -D warnings
- [ ] **광범위 sweep 7 fixture / 169 페이지** — typeset cursor advance 변경 영역 영역 회귀 점검 필수
- [ ] WASM 재빌드 (layout.rs 변경)

### 8.2 시각 판정 게이트 — **작업지시자 시각 검증 권장**
- sample16 (HWP3) page 18 — pi=395~401 본문 "○ 통합모델..." 같은 페이지 정상 emit (한컴 viewer page 16 정합)
- hwp3-sample14 (non-empty caption "Cut&Paste 할 영역") — 회귀 부재
- hwp3-sample10/11/13, exam_kor/math — 회귀 부재
- 다른 caption picture sample — 빈 caption skip 정상 / non-empty 정상

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 2번째) |
| `feedback_image_renderer_paths_separate` | layout.rs caption advance 단일 — 다른 렌더 경로 무관 |
| `feedback_hancom_compat_specific_over_general` | 빈 caption + Bottom 한정 가드 — 케이스별 명시 (일반화 없음) |
| `feedback_diagnosis_layer_attribution` | RHWP_DEBUG_TAC_CURSOR 추적 영역 영역 pi=394 ci=1 phantom +430.6px 정확 진단 |
| `feedback_pr_supersede_chain` | archive/task936 (미완 시도) + PR #918 (close) → **#958** (정확 정정) — (c) 패턴 |
| `feedback_diagnosis_layer_attribution` (Issue 분리) | Issue #952 → Issue 1 (#956) + Issue 2 (#957/본 PR) + Issue 3 (별도) 분리 진단 정합 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick `38fb2728` + `orders/20260517.md` 충돌 수동 해결 (양측 보존)
2. 자기 검증 — cargo test + clippy + 광범위 sweep + WASM 재빌드
3. 작업지시자 시각 검증 (sample16 page 18 본문 같은 페이지 + sample14/10/11/13 회귀 부재)
4. 검증 통과 → no-ff merge + push + archives + 5/17 orders
5. Issue #957 close + Issue #952 영역 영역 Issue 2 해결 코멘트 (Issue 3 잔존)
6. PR #958 close + 연속 PR #961 진행

---

작성: 2026-05-18
