---
PR: #745
제목: Task #634 — 첫 NewNumber Page 발화 전 쪽번호 미표시 (한컴 호환)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 14번째 PR)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: 8ef9c86e
PR_supersede: (c) 패턴 — Issue #782/#783 별 후속 PR 처리
---

# PR #745 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge `8ef9c86e`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `8ef9c86e` (--no-ff merge) |
| Cherry-pick commits | `e244c08a` + `cc1e6541` (Copilot 리뷰) |
| closes | #634 |
| 자기 검증 | cargo test ALL GREEN + page_number tests 8 PASS + sweep 170/170 same |
| WASM 빌드 | 4.66 MB |

## 2. 정정 본질 — 3 files, +42/-4

### 2.1 `src/renderer/page_number.rs` (+36/-2)

`PageNumberAssigner` 영역 영역 `numbering_started: bool` 추가:
- `assign()` 영역 영역 첫 NewNumber 소비 시점 영역 영역 `true` 전환
- `should_hide_page_number()` — NewNumber 존재 + 미발화 시 `true`
- 신규 unit tests 2건

### 2.2 `src/renderer/pagination/engine.rs` + `src/renderer/typeset.rs` (+6/-2)

`page_number_pos` 할당 영역 영역 `should_hide_page_number()` 가드 — 두 경로 동기 정합 (Copilot 리뷰 반영, `feedback_image_renderer_paths_separate` 정합).

## 3. 본 환경 cherry-pick + 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ 충돌 0건 (auto-merge 정합) |
| `cargo build --release` | ✅ 통과 |
| `cargo test --lib renderer::page_number` | ✅ **8 PASS** (신규 2건 + 기존 6건) |
| `cargo test --release` (전체) | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (opt-in 정합 입증) |
| WASM 빌드 | ✅ 4.66 MB |

## 4. 작업지시자 시각 검증 — 두 별 결함 발견

### 4.1 PR #745 본질 (한컴 호환 NewNumber 미표시)
opt-in 정합 입증 + 단위 테스트 결정적 검증 통과.

### 4.2 발견된 별 결함 — `feedback_pr_supersede_chain` (c) 패턴 후속 처리

#### Issue #782 — aift.hwp 페이지 2 영역 영역 일부 표 셀 검정 바탕
- BEFORE/AFTER 동일 (PR #745 무관)
- HWP 영역 영역 셀 배경 ColorRef 처리 결함 가능성
- PR #744 (HWPX ColorRef alpha 보존) 영역 영역 동일 패턴 영역 영역 HWP 적용 가능 영역 영역 점검

#### Issue #783 — aift.hwp 페이지 6 영역 영역 한컴 4 vs rhwp 3 불일치
- 페이지 번호 카운터 자체 영역 영역 본 PR 영역 영역 다른 본질 (NewNumber 미표시 정정 영역 영역 카운터 정정 별)
- 다중 구역 영역 영역 페이지 번호 처리 흐름 한컴 호환 점검 필요

## 5. opt-in 정합

NewNumber 부재 문서 (대부분 문서) 영역 영역 `new_page_numbers` 빈 벡터 → `should_hide_page_number()` 항상 `false` → 기존 동작 100% 보존. 광범위 sweep 영역 영역 170/170 same 영역 영역 입증.

## 6. 영향 범위

### 6.1 변경 영역
- 한컴 호환 영역 영역 첫 NewNumber Page 발화 전 페이지 영역 영역 쪽번호 미표시
- 표지/요약문/목차 영역 영역 시각 정합 (NewNumber 사용 문서)

### 6.2 무변경 영역 (sweep 170/170 same 입증)
- NewNumber 부재 문서 (대부분 문서) — 기존 동작 100% 보존
- 다른 layout/render 경로

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 영역 영역 14번째 PR) |
| `feedback_image_renderer_paths_separate` | **권위 사례 강화** — pagination/engine + typeset 두 경로 동기 정합 (Copilot 리뷰 반영) |
| `feedback_hancom_compat_specific_over_general` | NewNumber 부재 영역 영역 기존 동작 보존 (opt-in case 가드) — 회귀 위험 좁힘 |
| `feedback_visual_judgment_authority` | 한컴 PDF 정합 영역 영역 권위 — 작업지시자 시각 판정 + 두 별 결함 발견 |
| `feedback_pr_supersede_chain` | **(c) 패턴 적용** — 본 PR 머지 유지 + Issue #782/#783 별 후속 PR 통합 처리 |

## 8. 잔존 후속

- **Issue #782 OPEN** — aift.hwp 페이지 2 일부 표 셀 검정 바탕 (HWP ColorRef alpha 처리 결함 가능)
- **Issue #783 OPEN** — aift.hwp 페이지 6 한컴 4 vs rhwp 3 불일치 (페이지 번호 카운터 자체 영역 영역 다른 본질)

---

작성: 2026-05-10
