---
PR: #958
제목: fix — 빈 caption phantom advance 정정 (sample16 page 18 본문 다음 페이지 밀림 해소, closes #957)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 2번째)
처리: 옵션 A — 1 commit cherry-pick + orders 충돌 수동 해결 + 자기 검증 + WASM 재빌드 + no-ff merge
처리일: 2026-05-18
머지 commit: 0b630773
---

# PR #958 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + orders 충돌 수동 해결 + 자기 검증 + WASM 재빌드 + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `0b630773` (--no-ff merge) |
| Cherry-pick commit | `ee079382` (orders 1건 충돌 수동 해결, layout.rs auto-merge) |
| closes | #957 (Issue #952 영역 영역 Issue 2 분리 task) |
| 시각 판정 | ✅ 작업지시자 시각 검증 통과 |
| 자기 검증 | cargo test 1288 passed + clippy 통과 + sweep 169/169 same + WASM 4.4 MB |
| 연속 5 PR | #956 ✅ → **#958 (2번째)** → #961 → #963 → #964 |

## 2. 본질 (Issue #957 = Issue #952 Issue 2)

sample16 page 18 "나. 주요 과업내용" 후 본문 (pi=395~401 "○ 통합모델...")
다음 페이지 밀림 — 한컴 viewer 는 같은 페이지.

### Root cause (RHWP_DEBUG_TAC_CURSOR 추적)
```
Shape pi=394 ci=1 y_in=767.3 y_out=1197.9 dy=430.6 ⚠️
```
pi=394 ci=1 picture 의 빈 caption (`dir=Bottom width=0 paras=1 text=""`) 영역 영역
phantom +430.6px 누적 → pi=395 가 body 외 영역 emit → 다음 페이지 밀림.

## 3. 정정 본질 — `src/renderer/layout.rs` (:3491)

```rust
let caption_is_empty = caption.paragraphs.iter().all(|p|
    p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}')
        && p.controls.is_empty()
);
if !caption_is_empty && matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    if cap_bottom > result_y { result_y = cap_bottom; }
}
```

- `caption_is_empty` 가드 — caption 모든 paragraph 영역 영역 무의미 문자
  (≤U+001F control, U+FFFC object replacement) + `controls.is_empty()`
- 빈 caption 영역 영역 `result_y` advance skip → phantom +430.6px 제거
- `RHWP_DEBUG_TAC_CURSOR` 진단 영구화 (PR #956 `RHWP_DEBUG_PAGE_BORDER` 패턴 정합)

## 4. 영역 좁힘

| 영역 | 영향 |
|------|------|
| Empty caption picture | advance 제거 (회귀 fix) |
| Non-empty caption picture | 영향 없음 |
| Caption None / Top direction | 영향 없음 (Bottom 한정) |

→ 빈 caption + Bottom 한정 — `feedback_hancom_compat_specific_over_general` 정합.

## 5. 본 환경 충돌 수동 해결

| 파일 | 충돌 | 정합 |
|------|------|------|
| `mydocs/orders/20260517.md` | changed in both | 본 환경 PR #956 처리 섹션 + PR #958 Task #952/#957 작업 일지 양측 보존 통합 |
| `src/renderer/layout.rs` | auto-merge | PR #956 (`paper_based=true` :770) + PR #958 (`caption_is_empty` :3491) 다른 영역 — 양립 확인 |
| `task_m100_957*` 8 | added in remote | 신규 추가 (충돌 없음) |

## 6. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 1 commit + orders 수동 해결 | ✅ |
| PR #956 + #958 양립 | ✅ layout.rs :770 + :3491 공존 확인 |
| `cargo test --release --lib` | ✅ **1288 passed, 0 failed** (PR 본문 정합) |
| `cargo clippy --release --lib -- -D warnings` | ✅ 통과 |
| **광범위 sweep 7 fixture / 169 페이지** | ✅ **169 same / 0 diff** (회귀 부재) |
| WASM 재빌드 | ✅ 4.4 MB |
| 작업지시자 시각 판정 | ✅ **통과** |

sweep fixture 영역 영역 sample16 미포함 → 작업지시자 시각 검증 영역 영역 핵심 게이트.

## 7. 작업지시자 시각 판정 ✅ 통과

- sample16 (HWP3) page 18 — pi=395~401 본문 같은 페이지 정상 emit (한컴 viewer page 16 정합)
- hwp3-sample14 (non-empty caption "Cut&Paste 할 영역") — 회귀 부재
- hwp3-sample10/11/13, exam_kor/math — 회귀 부재
- PR #956 page border (paper-based) — 양립 정상

## 8. CI 통과

✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 2번째) |
| `feedback_image_renderer_paths_separate` | layout.rs caption advance 단일 — 다른 렌더 경로 무관 |
| `feedback_hancom_compat_specific_over_general` | 빈 caption + Bottom 한정 가드 — 케이스별 명시 (일반화 없음) |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | RHWP_DEBUG_TAC_CURSOR 추적 영역 영역 pi=394 ci=1 phantom +430.6px 정확 진단 + Issue #952 → Issue 1/2/3 분리 |
| `feedback_pr_supersede_chain` | archive/task936 (미완 시도) + PR #918 (close) → **#958** (정확 정정) — (c) 패턴 |

## 10. 잔존 후속

- 본 PR 본질 정정 (Issue 2) 의 잔존 결함 부재
- Issue #957 close 완료, Issue #952 OPEN 유지 (Issue 3 잔존)
- 연속 PR #961 진행 예정 (Issue #952 Issue 3 = 시험지 page 1 문9 vertical 아닐 수 있음 — 메타 점검 필요)

---

작성: 2026-05-18
