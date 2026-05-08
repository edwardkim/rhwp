# PR #638 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #638 |
| 제목 | Task #634: 한컴 호환 쪽번호 표시 동작 검증 + 회귀 방지 테스트 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 10번째 사이클 PR |
| base / head | `devel` ← `planet6897:task634-pr` |
| state / mergeable | OPEN / **UNKNOWN** (PR base 109 commits 뒤) |
| 변경 | 10 files, +1,643 / -0 (src 1 통합 테스트 +161 + 거버넌스 9) |
| commits | 1 (`870e430`, 단일 commit) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **closes #634** + Issue #637 분리 (이미 CLOSED) |
| 작성일 | 2026-05-06 05:36 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED

### 본 PR 의 정체성 영역
- **src 코드 변경 0** — Stage 2 fix revert 후 src 변경 부재 영역
- **회귀 방지 통합 테스트 8건** — `src/renderer/layout/integration_tests.rs` 영역
- **거버넌스 산출물 9 파일** — Stage 0~4 보고서 + 수행/구현 계획서 + 최종 보고서 + orders

---

## 2. Issue #634 권위 영역

### 가설 영역
첫 NewNumber Page 컨트롤 발화 전 페이지의 쪽번호 미표시 영역 가설 영역.

### Stage 4 영역 — 가설 깨짐 영역
새 한컴 PDF (1-up portrait) 정밀 측정 결과:
- 페이지 1 (cover disclaimer, NewNumber 발화 전) — 한컴 표시
- 페이지 6 (본문 시작, NewNumber 발화 전) — 한컴 표시

→ 가설 H1'' (NewNumber 게이팅) **잘못** → Stage 2 fix revert.

### 한컴 ↔ rhwp 일치 매트릭스 (PR 본문)

| Page | 한컴 | rhwp | 메커니즘 |
|------|------|------|---------|
| 1 (cover disclaimer) | 표시 | 표시 ✓ | PageNumberPos 등록 후 |
| 2 (35x27 표 cover) | 미표시 | 표시 ✗ | Issue #637 분리 |
| 3 (14x17 표 요약문) | 미표시 | 표시 ✗ | Issue #637 분리 |
| 4, 5 (목차/별첨) | 미표시 | 미표시 ✓ | PageHide page_num=true |
| 6 (본문 시작) | 표시 | 표시 ✓ | 정상 |
| 7+ (NewNumber 후) | 표시 | 표시 ✓ | 정상 |

→ src 변경 없이 본 환경 영역의 동작이 한컴과 정합 영역 (페이지 2, 3 영역 외).

### Issue #634 assignee 영역
- assignee 미지정 — 컨트리뷰터 자기 등록 영역.

### Issue #637 영역 (이미 CLOSED)
- aift.hwp 페이지 2, 3 (큰 표만 있는 cover-style) 쪽번호 미표시 메커니즘 분석 영역
- PR 본문 시점에 별도 issue 분리 영역 → 이미 CLOSED 영역 (현 시점 영역)

---

## 3. 본 환경 정합 상태 점검

### 본 환경 src 영역
- `src/renderer/page_number.rs` — `PageNumberAssigner` 영역 (PR 본문 명시)
- `src/renderer/pagination.rs` — `PageContent` 영역
- `src/renderer/layout.rs:1086` — `build_page_number` 영역
→ Stage 2 fix revert 영역으로 변경 부재 영역.

### 본 환경 회귀 방지 가드 영역 (PR 의 8 통합 테스트 영역)
- `test_634_aift_page1_shows_page_number`
- `test_634_aift_page4_pagehide_no_page_number`
- `test_634_aift_page5_pagehide_no_page_number`
- `test_634_aift_page6_shows_page_number`
- `test_634_aift_page7_shows_page_number`
- `test_634_gukrip_page1_pagehide_no_page_number`
- `test_634_gukrip_page3_shows_page_number`
- `test_634_no_newnumber_doc_shows_page_numbers_from_page1`

→ 8건 영역 영구 보존 영역 → 향후 동일 결함 영역 발생 시 즉시 검출 영역.

### 본 환경 영역의 영역 영역 충돌 영역 발견
- `src/renderer/layout/integration_tests.rs` — 본 환경 영역의 PR #627 영역의 `test_624_textbox_inline_shape_y_on_line2_p2_q7` 영역과 PR #638 의 8 통합 테스트 영역 동시 영역 추가 영역 → **content 충돌** 발생

### 거버넌스 영역 본 환경 명명 규약 정합
- `task_m100_634*` 영역 — 본 환경 명명 규약 정합 영역 (PR #622/#627/#632/#636 패턴)

---

## 4. 본 PR 영역의 가치 영역

### 4.1 src 변경 0 영역 — 본 환경 동작 영역의 정합성 영역 검증
PR 본문 매트릭스 영역 — 본 환경 영역의 쪽번호 동작 영역이 한컴과 정합 영역 (페이지 2, 3 외).

### 4.2 회귀 방지 통합 테스트 8건 — 영구 보존 영역
8 건 영역의 통합 테스트 영역으로 향후 동일 결함 영역 발생 시 즉시 검출 영역 가능.

### 4.3 가설 시행 착오 영역의 권위 사례 영역
- Stage 0~3 영역 — 가설 H1'' (NewNumber 게이팅) 영역 영역 진행 영역
- Stage 2 fix 영역 시도 (9 파일 영역 src 변경 영역)
- Stage 4 영역 — 새 한컴 PDF 측정 영역에서 가설 깨짐 영역 → revert 영역

→ **가설 시행 착오 영역의 학습 영역의 권위 사례 영역** — `feedback_essential_fix_regression_risk` 영역의 권위 영역.

### 4.4 본 사이클 영역의 영역 영역 패턴
- PR #636 (Task #630) 영역의 Stage 4 회귀 발견 → 철회 영역과 동일 패턴 영역
- 본 PR 영역의 Stage 4 가설 깨짐 → revert 영역
- → 본 사이클의 **TDD 흐름 영역의 회귀 발견 → 철회 영역의 권위 패턴 영역 누적** 영역

---

## 5. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr638_test`) 에서 진행:

### cherry-pick + 충돌 처리
- 1 commit (`870e430`) cherry-pick — **2 영역 충돌 발생** 영역
- `mydocs/orders/20260506.md` add/add 충돌 → `git checkout --ours` 본 환경 영역 보존 (PR #622/#627/#632/#636 패턴)
- **`src/renderer/layout/integration_tests.rs` content 충돌** — 본 환경 영역의 PR #627 영역의 `test_624` 영역과 PR #638 영역의 8 영역 동시 영역 추가 영역 → 충돌 marker 제거 영역으로 두 영역 모두 영역 보존 영역 (test_624 1 + test_634 8 = 9 정합)

### 결정적 검증 결과 (모두 통과)

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1165 passed** (1157 + test_634 8 신규 정합) |
| `cargo test --lib --release test_634` | ✅ **8/8 passed** |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |

### 본 환경 영역의 cargo test 영역
- 본 환경 영역의 baseline 1157 + 본 PR 영역의 8 신규 = **1165** 영역 정합
- PR 본문 영역의 1142 영역과 다른 영역 — 본 환경 영역에 PR #621 (issue_617) + PR #627 (test_624) + PR #630 (test_630) 영역의 추가 테스트 영역 누적 영역

---

## 6. 옵션 분류

본 환경 cherry-pick simulation 결과 + 거버넌스 영역 본 환경 명명 규약 정합 영역 기반:

### 옵션 A — 전체 cherry-pick (1 commit, content 충돌 해결)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick 870e430
# orders add/add 충돌 → ours
# integration_tests.rs content 충돌 → 충돌 marker 제거 영역으로 두 영역 모두 보존
```

**장점**:
- 회귀 방지 통합 테스트 8건 영역 영구 보존
- 거버넌스 산출물 9 파일 영역의 가설 시행 착오 영역 학습 영역 보존
- author Jaeook Ryu 보존
- 거버넌스 영역 명명 규약 정합 (`task_m100_634*`)

**잠재 위험**:
- `integration_tests.rs` content 충돌 영역의 영역 영역 처리 영역 — 두 영역 모두 영역 보존 영역으로 정합

### 옵션 A-2 — 통합 테스트 8건만 cherry-pick (거버넌스 분리)
**잠재 위험**:
- 가설 시행 착오 영역 학습 영역 손실 — 본 PR 영역의 Stage 0~4 영역의 권위 영역 손실 영역

### 권장 영역 — 옵션 A

**사유**:
1. **본 환경 결정적 검증 모두 통과** — cargo test 1165 / test_634 8/8 / clippy 0
2. **src 변경 0 영역** — 회귀 위험 영역 부재 영역
3. **회귀 방지 통합 테스트 8건 영역의 영구 보존 영역**
4. **가설 시행 착오 영역의 학습 영역 보존** — 본 사이클 영역의 권위 패턴 영역 누적 (PR #636 Stage 4 와 동일 영역)
5. **content 충돌 영역의 두 영역 모두 영역 보존 영역 정합** — test_624 + test_634 = 9 정합

### 옵션 영역 요약 표

| 옵션 | 진행 가능 | Issue #634 정정 | 결정적 검증 | 권장 |
|------|----------|----------------|------------|------|
| **A** (전체 1 commit + content 영역 영역 보존) | ✅ 충돌 2 (orders ours + content 두 영역 보존) | ✅ src 변경 0 + 회귀 가드 8건 | ✅ 1165/8 | ⭐ |
| **A-2** (통합 테스트만) | ✅ | ✅ 가드만 | ✅ 동일 | ❌ 학습 손실 |

---

## 7. 잠정 결정

### 권장 결정
- **옵션 A 진행** — 1 commit cherry-pick + content 충돌 영역 두 영역 모두 영역 보존
- 본 환경 결정적 검증 진행 + WASM 빌드 부재 (src 변경 0)
- 시각 판정 영역 부재 가능 영역 (회귀 방지 가드 영역만 영역 영역, 시각 영향 영역 부재 영역)

### 검증 영역 (옵션 A 진행 시 본 환경 직접 점검)
1. cherry-pick (1 commit) — 2 충돌 (orders ours + integration_tests.rs 두 영역 보존)
2. `cargo test --lib --release` 1165 passed (test_634 8 신규 정합)
3. `cargo test --lib --release test_634` 8/8 passed
4. `cargo test --test svg_snapshot --release` 7/7 (영향 부재 영역)
5. `cargo clippy --lib -- -D warnings` 0
6. `cargo build --release`

---

## 8. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- **`feedback_essential_fix_regression_risk`** — 가설 시행 착오 영역의 권위 사례 영역. Stage 0~3 가설 → Stage 4 측정 영역에서 깨짐 → revert 영역 → 회귀 가드 8건 영역 영구 보존
- **`reference_authoritative_hancom`** — 한컴 PDF 영역의 1-up portrait 영역 권위 정답지 영역 비교 영역
- `feedback_close_issue_verify_merged` — Issue #634 close 시 본 PR 머지 검증 + 수동 close
- `feedback_assign_issue_before_work` — Issue #634 assignee 미지정 영역
- 거버넌스 영역 본 환경 명명 규약 정합 — `task_m100_634*` 영역 (PR #622/#627/#632/#636 패턴)
- 본 사이클 영역의 TDD Stage 4 회귀 발견 → 철회 영역 패턴 누적 — PR #636 + **본 PR**

---

## 9. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_638_review.md` 작성 → 승인 요청
3. (필요 시) `pr_638_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy) + 판단 → `pr_638_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (전체 1 commit + content 충돌 두 영역 보존, 권장) / 옵션 A-2 (통합 테스트만)
2. **시각 판정 영역** — src 변경 0 영역으로 시각 판정 영역 부재 가능 영역 (회귀 방지 가드만 영역). 진행 가/부

결정 후 본 환경 cherry-pick + 결정적 검증 + `pr_638_report.md` 작성.
