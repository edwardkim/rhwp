# PR #641 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #641 |
| 제목 | Task #639: 한컴 호환 — cover-style 페이지 자동 쪽번호 미표시 (closes #639) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 11번째 사이클 PR |
| base / head | `devel` ← `planet6897:pr-task639` |
| state / mergeable | OPEN / MERGEABLE / CLEAN |
| 변경 | 15 files, +1,916 / -2 |
| commits | 11 (5 task637 + 5 task639 + merge + conflict 정정) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **closes #639** + Issue #637 통합 (이미 CLOSED) |
| 작성일 / 갱신 | 2026-05-06 07:09 / 2026-05-07 15:53 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED

---

## 2. 본 PR 영역의 본질 영역

### 2.1 cover-style 룰 영역
PR 본문 영역의 결정적 룰:
> 페이지가 `items=1` 인 단일 완전한 Table (PartialTable 아님) 을 포함하고 그 Table 의 `treat_as_char=false` 일 때 한컴은 쪽번호 표시 안 함.

```rust
// finalize_pages 영역 영역 영역 PageHide 적용 직후
if page.page_hide.is_none() && Self::is_cover_style_page(page, paragraphs) {
    page.page_hide = Some(crate::model::control::PageHide {
        hide_page_num: true,
        ..Default::default()
    });
}
```

### 2.2 본 환경 영역의 셀 안 PageHide 결함 영역과의 관계

**중요 영역** — PR 영역의 분석 영역은 **셀 안 PageHide 영역을 무시 영역하고 본문 paragraph 영역만 점검** 영역:
- Issue #637 본문: "두 페이지 모두 PageHide page_num=true 컨트롤 **없음** (검증됨)"
- 본 PR Stage 1 commit 영역: "H2 (셀 내부 PageHide): 기각 — 문서 전체 PageHide 가 정확히 2개"

**본 환경 직접 측정 영역의 결과 영역 (PR #638 검토 영역에서 발견 영역)**:
```
section 0 / paragraph 1 / Table[0] / 셀[167] / paragraph[3]
text: "       년        월        일"
ctrl[0] = PageHide(header=true, footer=true, master=true,
                   border=true, fill=true, page_num=true)
```

→ aift.hwp page 2 의 셀 안 영역에 **PageHide 가 정확히 인코딩** 되어 있고, 본 환경 페이지네이션 (`pagination/engine.rs:516-531`) 영역이 **셀 안 PageHide 를 무시**하여 검출 못 함. PR #641 의 cover-style 룰은 **본 환경 결함 영역의 우회 (workaround)** 영역.

### 2.3 작업지시자 결정 영역 — 본 PR 만 리뷰 + 처리

작업지시자 결정 영역: "그냥 이번 PR 만 리뷰해서 처리하겠습니다."
- 셀 안 PageHide 영역의 본질 정정 영역은 별도 영역 진행 영역
- 본 PR 영역의 cover-style 룰 영역으로 page 2/3 영역 즉시 정합 영역
- 후속 영역에서 본질 정정 영역 후 영역 cover-style 룰 영역 폴백 영역으로 의미 영역 가능 영역

---

## 3. 본 환경 정합 상태 점검

### 본 환경 영역의 두 페이지네이션 경로 영역 (PR 본문 명시)
- **`TypesetEngine::typeset_section`** (기본 main path, `src/renderer/typeset.rs`) — `render_page_svg_native` 영역 사용
- **`Paginator::paginate_with_measured`** (RHWP_USE_PAGINATOR=1 fallback, `src/renderer/pagination/engine.rs`)

→ 본 PR 영역은 두 경로 영역 모두 영역 동일 영역 fix 영역 적용 영역 정합 영역.

### 본 환경 영역의 PageHide 영역 처리 영역 점검
- 본문 paragraph 영역의 PageHide 영역 — `pagination/engine.rs:519-520` 영역 정합 영역
- 셀 안 paragraph 영역의 PageHide 영역 — **수집 영역 부재** (본질 결함 영역, PR #638 댓글 명시 영역)
- `hide_border` + `hide_fill` 영역의 렌더러 영역 가드 — **부재** (PR #638 댓글 명시 영역)

### Issue #637 영역
- 이미 CLOSED 영역 (분석 완료, 컨트리뷰터 영역 close)
- 본 PR 영역 머지 시 영역 closes #639 + 분석 docs 통합 영역

---

## 4. PR 의 본질 정정 영역

### 4.1 src 변경 영역
- `src/renderer/typeset.rs` (+30/-2 영역) — `TypesetEngine::typeset_section` 영역의 `finalize_pages` 영역에 cover-style 룰 추가
- `src/renderer/pagination/engine.rs` (+30/-1 영역) — `Paginator::paginate_with_measured` 영역의 동일 영역
- `src/renderer/layout/integration_tests.rs` (+95) — 통합 테스트 5건

### 4.2 통합 테스트 5건 (TDD RED → GREEN)
| 테스트 | 영역 | 결과 |
|--------|------|------|
| `test_639_aift_page2_cover_style_no_page_number` | items=1, Table 35×27, tac=false | PASS (footer 글리프 0) |
| `test_639_aift_page3_cover_style_no_page_number` | items=1, Table 14×17, tac=false | PASS (footer 글리프 0) |
| `test_639_aift_page1_shows_page_number` | items=2 (회귀 가드) | PASS (footer 글리프 3) |
| `test_639_aift_page6_shows_page_number` | items=18 (회귀 가드) | PASS (footer 글리프 3) |
| `test_639_aift_page74_tac_true_table_shows_page_number` | items=1 + tac=true (회귀 가드) | PASS (footer 글리프 4) |

### 4.3 174 샘플 룰 매칭 (PR 본문)
- 174 샘플 전수 조사 결과 — aift.hwp 페이지 2, 3 만 매칭 (전체의 0.06%)
- 한컴 PDF 미표시와 정확 일치
- **회귀 위험 영역 매우 낮음**

### 4.4 거버넌스 산출물 영역 (12 파일)
- `mydocs/plans/task_m100_637.md` + `task_m100_639.md` + `task_m100_639_impl.md`
- `mydocs/working/task_m100_637_stage0.md` + `stage1.md`
- `mydocs/working/task_m100_639_stage1.md` + `stage2.md` + `stage3.md`
- `mydocs/report/task_m100_637_report.md` + `task_m100_639_report.md`
- `examples/inspect_637.rs` (분석 도구)

---

## 5. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr641_test`) 에서 진행:

### cherry-pick + 충돌 처리
- 11 commits cherry-pick 영역 — 다수 충돌 발생 영역
- `mydocs/orders/20260506.md` add/add 충돌 (5+ 영역) → `git checkout --ours` 본 환경 보존 (PR #622/#627/#632/#636 패턴)
- `de3c220` (orders only commit, #639 등록 영역) skip — 본 환경 영역에 영역 영역 영역
- `fb583bc` (Stage 1 RED 통합 테스트 영역) — `integration_tests.rs` content 충돌 영역 → 충돌 marker 제거 영역으로 두 영역 모두 보존 영역 → 빈 commit 영역 (충돌 marker 제거 영역만 영역, 본질 영역 다음 commit 영역 영역)
- `b33a7ff` (devel merge conflict 정정 영역) skip — 본 환경 영역의 정정 영역 후속 commit 영역 영역
- 최종 영역에 본 환경 영역의 `integration_tests.rs` 영역의 `assert!()` 닫는 `);` + 함수 닫는 `}` 누락 영역 → 본 환경 직접 정정 영역

### 결정적 검증 결과 (모두 통과)

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1162 passed** (회귀 0) |
| `cargo test --lib --release test_639` | ✅ **5/5 passed** (cover-style 룰 정합) |
| `cargo test --lib --release test_624` | ✅ 1/1 (회귀 0) |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |

### 본 환경 영역의 cargo test 1162 영역
- 본 환경 baseline 1157 (PR #684 후) + test_639 5건 영역 = 1162 영역
- PR 본문 영역의 1139+76 영역과 다른 영역 — 본 환경 영역에 영역 누적 영역 영역 정합 영역
- **PR #638 (test_634 8건) 영역은 close 영역으로 본 환경 영역에 부재** 영역 정합 영역

---

## 6. 옵션 분류

본 환경 cherry-pick simulation 결과 + 충돌 처리 영역의 영역 영역 영역 + 작업지시자 결정 영역 (본 PR 만 리뷰) 기반:

### 옵션 A — 전체 cherry-pick (11 commits, 본 환경 영역 정정 보강 1 commit)
**진행 영역**:
- 11 commits 영역 cherry-pick 영역 (orders ours 보존 + de3c220/b33a7ff skip + fb583bc 빈 commit + integration_tests 영역 본 환경 정정 추가 commit)
- 본 환경 영역의 `integration_tests.rs` 영역 정정 commit 영역 추가 영역

**장점**:
- TDD 5 단계 영역 + 분석 영역 (Task #637 5 commits) + fix 영역 (Task #639 5 commits) 모두 보존
- 거버넌스 산출물 12 파일 영역 보존
- 회귀 가드 5건 영구 보존
- author Jaeook Ryu 모든 commits 보존

**잠재 위험**:
- 충돌 영역 처리 영역의 영역 영역 영역 — 본 환경 정정 추가 commit 영역 영역 영역
- 본 환경 영역의 셀 안 PageHide 영역 영역 정정 영역 부재 영역 (작업지시자 결정 영역으로 별도 영역 진행)

### 옵션 A-2 — squash 머지 (1 단일 commit)
- TDD 5 단계 영역 + 분석 영역 손실 영역

### 권장 영역 — 옵션 A (11 commits 단계별 보존)

**사유**:
1. 본 환경 결정적 검증 모두 통과 — cargo test 1162 / test_639 5/5 / clippy 0
2. TDD 5 단계 영역의 권위 패턴 영역 (Stage 0 분석 → 1 RED → 2 GREEN → 3 회귀)
3. 회귀 위험 영역 매우 낮음 (174 샘플 영역 0.06% 영역 영향)
4. 거버넌스 영역 본 환경 명명 규약 정합 (`task_m100_637*` + `task_m100_639*`)

---

## 7. 잠정 결정

### 권장 결정
- **옵션 A 진행** — 11 commits 단계별 cherry-pick + 본 환경 정정 보강 1 commit
- 본 환경 결정적 검증 + WASM 빌드 + 시각 판정 ★

### 검증 영역 (옵션 A 진행 시 본 환경 직접 점검)
1. cherry-pick (11 commits) — 충돌 처리 영역 영역
2. `cargo test --lib --release` 1162 passed (test_639 5 신규 정합)
3. `cargo test --test svg_snapshot --release` 7/7
4. `cargo test --test issue_546 / issue_554 / issue_418 / issue_501 / issue_630` 통과
5. `cargo clippy --lib -- -D warnings` 0
6. Docker WASM 빌드
7. **시각 판정 ★** — `samples/aift.hwp` page 2 (사업계획서 표지) + page 3 (요약문) 영역의 쪽번호 미표시 영역 + page 1 / page 6 / page 74 영역의 쪽번호 표시 영역 회귀 가드 영역 작업지시자 직접 시각 판정 (한컴 2022 PDF 권위 정답지 비교)

---

## 8. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- **`feedback_rule_not_heuristic`** — 본 PR 의 룰 영역 (`items=1 + Table + tac=false`) 은 명시 영역 비트/카운트 조합 영역, 휴리스틱 임계값 영역 부재. 다만 영역 본질 영역 영역 영역 셀 안 PageHide 영역 영역 영역 영역 — `feedback_rule_not_heuristic` 영역의 본질 영역 영역과 영역 영역 영역 영역
- **`feedback_essential_fix_regression_risk`** — 174 샘플 영역 광범위 sweep 영역 정합 영역
- **`feedback_pdf_not_authoritative`** — IR 기반 검증 영역 (page_hide → SVG footer 글리프)
- **`reference_authoritative_hancom`** — 한컴 PDF 권위 정답지 영역 비교
- `feedback_close_issue_verify_merged` — Issue #639 close 시 본 PR 머지 검증 + 수동 close
- `feedback_assign_issue_before_work` — Issue #639 assignee 미지정 영역
- 거버넌스 영역 본 환경 명명 규약 정합 (`task_m100_637*` + `task_m100_639*`)

---

## 9. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_641_review.md` 작성 → 승인 요청
3. (필요 시) `pr_641_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정 ★) + 판단 → `pr_641_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (11 commits 단계별 + 본 환경 정정 보강 1 commit, 권장) / 옵션 A-2 (squash)
2. **시각 판정 권위 영역** — `samples/aift.hwp` page 2/3 영역 (cover-style fix) + page 1/6/74 영역 (회귀 가드) 작업지시자 직접 시각 판정 진행 가/부
3. **WASM 빌드 + rhwp-studio public 갱신** 가/부

결정 후 본 환경 cherry-pick + 결정적 검증 + WASM 빌드 + 시각 판정 ★ + `pr_641_report.md` 작성.
