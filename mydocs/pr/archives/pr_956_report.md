---
PR: #956
제목: fix — 쪽 테두리 paper-based outline 강제 (#920 비트 해석 회귀 정정, closes #952)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 1번째)
처리: 옵션 A — 1 commit cherry-pick + 자기 검증 + WASM 재빌드 + no-ff merge
처리일: 2026-05-18
머지 commit: b31e38ff
---

# PR #956 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + 자기 검증 + WASM 재빌드 + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `b31e38ff` (--no-ff merge) |
| Cherry-pick commit | `bb3f40ad` (충돌 0건, CLEAN) |
| closes | #952 (Issue 1 만, Issue 2/3 별도 task 분리 — Issue OPEN 유지) |
| 시각 판정 | ✅ 작업지시자 시각 검증 통과 |
| 자기 검증 | cargo test 1288 passed + clippy 통과 + sweep 169/169 same + WASM 4.4 MB |
| 연속 5 PR | **#956 (1번째)** → #958 → #961 → #963 → #964 |

## 2. 사전 작업 — 원격 동기화 (작업지시자 지시)

본 세션 영역 영역 5/11 작업 후 정체 — origin/devel 영역 영역 5/12~17 PR #919~#954 누적.
작업지시자 지시 영역 영역 동기화 선행:
- 본 세션 5/11 작업 (PR #786~#818 + v0.7.11) origin 반영 확인
- devel 3자 동기화: `local/devel` = `devel` = `origin/devel` = `016e694c` (+19 commits FF)
- WASM 빌드 4.4 MB (LTO PR #818 적용 + 5/12~17 누적)

`feedback_release_sync_check` 정합 — PR 작업 전 동기화 필수 절차 입증.

## 3. 본질 (Issue #952 Issue 1)

페이지 외곽선 (page border) 이 paper-based 가 아닌 body-based 로 잘못 렌더 → 본문 돌출.

### 회귀 source 확정 (bisect)
| Commit | 외곽선 |
|--------|--------|
| task877 baseline (`8514e68a`) | paper-based ✓ |
| `4bb11289` (#920) | body-based ⚠️ 회귀 도입 |
| 현재 devel `layout.rs:764` | body-based ⚠️ 회귀 잔존 (본 환경 확인) |

### 진단 — bit 0 은 outline 위치 결정 비트 아님
5+ samples 한컴 viewer 실측 — attr 0/1, textBorder PAPER/CONTENT 양쪽 다 paper-based.
회귀 history: task877 (`!= 0`) → #920 (`== 0`) → 본 PR (`true`, 모든 sample 정합).

## 4. 정정 본질 — `src/renderer/layout.rs` +16/-2 (1 곳)

```rust
let paper_based = true;  // #920 회귀 코드 (attr & 0x01) == 0 대체
if std::env::var("RHWP_DEBUG_PAGE_BORDER").is_ok() { eprintln!(...); }
```

- `paper_based = true` 강제
- `RHWP_DEBUG_PAGE_BORDER` 진단 영구화
- 회귀 history 코멘트 (task877/#920/#952) 영구 보존 — 재회귀 방지

## 5. 추가 fixture / PDF (13 files)

| 분류 | 파일 |
|------|------|
| 시험지 fixture | `3-09월/3-09월(2023)/3-10월/3-11월` HWP+HWPX (8) |
| 한컴 2022 권위 PDF | `pdf/3-09월~3-11월` (4) |

→ `reference_authoritative_hancom` 정합 — 한컴 2022 PDF 권위 영구 보존 + 회귀 가드.

## 6. 본 PR 범위 외 (별도 task 분리 — PR 본문 명시)

| Issue | 본질 | 분리 사유 |
|-------|------|----------|
| Issue 2 | sample16 page 18 본문 다음 페이지 밀림 | typeset multi-TAC-shape cursor over-advance ~430px (typeset.rs 2700+ 줄 multi-state 복잡도) |
| Issue 3 | 시험지 page 1 문9 vertical 처짐 | HWP5 column layout (다른 root cause) |

→ 부분 해결 + 명확한 분리 (archive/task936 "9회 시도 + 5회 revert" 대조 교훈).
Issue #952 영역 영역 Issue 2/3 잔존 영역 영역 OPEN 유지 — 연속 PR #958 = Issue 2 정정 예정.

## 7. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 1 commit | ✅ 충돌 0건 (CLEAN) |
| `cargo test --release --lib` | ✅ **1288 passed, 0 failed** (PR 본문 정합) |
| `cargo clippy --release --lib -- -D warnings` | ✅ 통과 |
| **광범위 sweep 7 fixture / 169 페이지** | ✅ **169 same / 0 diff** (기존 fixture 회귀 부재) |
| WASM 재빌드 | ✅ 4.4 MB |
| 작업지시자 시각 판정 | ✅ **통과** (sample16 + 시험지 page border paper-based) |

sweep fixture 영역 영역 sample16/시험지 미포함 → 작업지시자 시각 검증 영역 영역 본 PR 핵심 게이트.

## 8. 작업지시자 시각 판정 ✅ 통과

- sample16 (HWP3/HWP5/HWPX) page 17 외곽선 — paper-based 정합 (task877 baseline)
- 시험지 (3-09월/3-11월) HWP5/HWPX page 1 외곽선 — paper-based 정합 (한컴 2022 PDF 권위)
- 다른 sample page border 회귀 부재
- Issue 2/3 잔존 확인 (본 PR 범위 외)

## 9. CI 통과

✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (HWP3/WMF/page border 핵심) — 연속 5 PR 1번째 |
| `feedback_release_sync_check` 권위 사례 강화 | PR 작업 전 원격 동기화 선행 (5/11 → 5/17 격차 +19 commits FF) |
| `feedback_image_renderer_paths_separate` | layout.rs page border 분기 단일 — 다른 렌더 경로 무관 |
| `feedback_hancom_compat_specific_over_general` 권위 사례 강화 | bit 0 추측 해석 (task877/#920) → 5+ sample 한컴 실측 영역 영역 `true` 강제 — 추측보다 실측 |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | 회귀 source `4bb11289` (#920) bisect 정확 + Issue 1/2/3 분리 진단 |
| `feedback_pr_supersede_chain` | task877 → #920 (`4bb11289` 회귀 도입) → **#956** (회귀 정정) — (c) 패턴 |
| `reference_authoritative_hancom` | 시험지 한컴 2022 PDF (`pdf/`) 권위 영구 보존 + 회귀 fixture |
| `feedback_v076_regression_origin` | #920 추측 bit 해석 → 작업지시자 환경 회귀 — 한컴 실측 영역 영역 정정 패턴 |

## 11. 잔존 후속

- 본 PR 본질 정정 (Issue 1) 의 잔존 결함 부재
- Issue #952 OPEN 유지 — Issue 2/3 별도 task 분리
- 연속 PR #958 = Issue 2 (sample16 page 18 typeset cursor over-advance) 정정 예정

---

작성: 2026-05-18
