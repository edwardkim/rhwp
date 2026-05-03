# PR #538 검토 문서

**PR**: [#538 fix: 21_언어_기출_편집가능본.hwp 줄간격 716 HU drift 정정 (Task #534 v2 + #537 + #539)](https://github.com/edwardkim/rhwp/pull/538)
**작성자**: @planet6897 (Jaeuk Ryu)
**Base / Head**: `devel` ← `devel` (컨트리뷰터 fork 의 devel 브랜치)
**Linked Issue**: [#537](https://github.com/edwardkim/rhwp/issues/537), [#539](https://github.com/edwardkim/rhwp/issues/539) (closes), Task #534 v2
**상태**: OPEN, **CONFLICTING** (mergeStateStatus = DIRTY) ⚠️
**CI**: ALL SUCCESS
**작성일**: 2026-05-03
**검토일**: 2026-05-03

---

## 1. 처리 결정

**작업지시자 의도**: ✅ **본질만 cherry-pick** (Task #534 v1+v2 + #537 + #539 의 12 commit 만). fork 의 devel 누적분 (182 commits / 42 task) 전체 머지 아님.

**컨트리뷰터 안내**: 다음 PR 전에 devel 동기화 (`git pull --ff-only origin devel` + 충돌 해소) 부탁.

---

## 2. 개요 + 위험 분석

### 2.1 PR 의 비정상 상태

| 항목 | 값 |
|------|-----|
| changedFiles | 129 ⚠️ |
| additions / deletions | +19,350 / -2,387 ⚠️ |
| commits | 182 (devel 분기 후) ⚠️ |
| 명시 task (PR 본문) | Task #534 v2 + #537 + #539 (3 task) |
| 실제 누적 task | 42 task (#332 / #435 / #439 / #445 / ... / #539) |
| mergeable | CONFLICTING |
| mergeStateStatus | DIRTY |

**근본 원인**: 컨트리뷰터의 fork 가 본 repo 의 devel (`e3285b27`, Task #516 머지 시점) 에서 분기된 후 fork 의 devel 에서 매우 많은 작업을 누적 (40 여 task). PR 의 base/head 가 모두 `devel` 이라 fork 의 devel 전체가 PR 에 포함됨.

### 2.2 본 PR 의 본질 (PR 본문 기준)

| Task | 결함 | 정정 위치 |
|------|------|----------|
| **#537** | 21_언어 기출 hwp TAC 표 직후 첫 답안 줄간격 716 HU drift (11곳) | `src/renderer/layout.rs` lazy_base trailing-ls 보정 (+14 / -1) |
| **#539** | 글박스 (InFrontOfText tac=true Shape) 호스트 직후 paragraph 줄간격 716 HU drift (2곳) | `src/renderer/layout.rs` prev_has_overlay_shape 가드 완화 (treat_as_char 제외, +9 / -0) |
| **#534 v2** | layout_shape_item TAC Picture LINE_SEG.column_start 정합 | `src/renderer/layout.rs` (+12 / -1) |

**핵심 변경 영역**: 모두 `src/renderer/layout.rs` 의 **매우 작은 영역**. 본질 정정은 `+44 / -3` 수준.

### 2.3 의존성 — Task #534 v1 + v2

- PR 본문은 Task #534 v2 만 명시
- 그러나 **v2 는 v1 의 후속 정정** (`9dfc56a` 가 v1 `4abee04` 의 동일 영역을 보강)
- 본 환경 devel 에 v1 미반영 → cherry-pick 시 **v1 도 함께 cherry-pick 필수**

---

## 3. cherry-pick 대상 commits (12 commits)

| 순서 | commit | task | 변경 영역 |
|------|--------|------|---------|
| 1 | `fbcb5c5` | #534 계획서 | mydocs/plans/task_m100_534.md (66) |
| 2 | `b669ab5` | #534 Stage 1 진단 | mydocs/working/task_m100_534_stage1.md (132) |
| 3 | `4abee04` | #534 Stage 3 정정 | **`src/renderer/layout.rs` (+25 / -2)** |
| 4 | `5357223` | #534 Stage 3-5 보고서 | mydocs (278) |
| 5 | `9dfc56a` | #534 v2 정정 | **`src/renderer/layout.rs` (+12 / -1)** |
| 6 | `47d1aac` | #534 v2 보고서 | mydocs (135) |
| 7 | `226b644` | #537 Stage 1 + TDD | mydocs + `tests/integration_tests.rs` (496) |
| 8 | `1803bc6` | #537 Stage 2 정정 | **`src/renderer/layout.rs` (+14 / -1)** |
| 9 | `a39085a` | #537 Stage 3 보고서 | mydocs (271) |
| 10 | `e8a0a8c` | #539 Stage 1 + TDD | mydocs + `tests/integration_tests.rs` (557) |
| 11 | `0db709b` | #539 Stage 2 정정 | **`src/renderer/layout.rs` (+9 / -0)** |
| 12 | `eb0ddc2` | #539 Stage 3 보고서 | mydocs (260) |

**소스 변경 합계**: `src/renderer/layout.rs` +60 / -4, `tests/integration_tests.rs` +228, mydocs 다수.

---

## 4. 충돌 위험 점검

### 4.1 `mydocs/orders/20260502.md`

- 본 환경 devel: PR #506/#509/#510 의 일지 누적
- PR #538 의 commits: 컨트리뷰터의 #534 v1+v2 일지 추가
- 충돌 가능성: 🟧 중간 (PR #507/#531 사례에서 동일 파일 auto-merge 정합 확인)

### 4.2 `mydocs/orders/20260503.md`

- 본 환경 devel: 오늘 작업 (PR #507 + PR #531 + 이슈 #543 등록) 기록
- PR #538 의 commits (#537/#539): 컨트리뷰터의 #537/#539 일지 추가
- 충돌 가능성: 🟥 **거의 확실** — 동일 파일을 양쪽에서 신규/수정. 수동 해소 필요

### 4.3 `src/renderer/layout.rs`

- 본 환경 devel: PR #531 의 `src/renderer/layout/table_layout.rs` 정정 적용 (다른 파일)
- PR #538 의 commits: `src/renderer/layout.rs` (다른 파일) 정정
- 충돌 가능성: 🟢 **매우 작음** — 다른 파일 + Task #534/#537/#539 의 정정 위치가 layout.rs 내 다른 영역

### 4.4 `tests/integration_tests.rs`

- 본 환경 devel: 사전 영역 변경 가능 (Task #480 / #490 / #489 등)
- PR #538: TDD 테스트 신규 추가
- 충돌 가능성: 🟧 중간 — Task #537/#539 의 신규 test 가 같은 파일 끝에 추가되는 경우 정합

---

## 5. 검토 항목

### 5.1 코드 품질 (PR 본문 + commit 메시지 기준)

#### Task #537 — TAC 표 직후 답안 줄간격 716 HU drift

**근본 원인** (PR 본문):
- `prev_tac_seg_applied` 가드 + Task #479 trailing-ls 제외 + lazy_base sequential drift 동결

**정정**:
```rust
let trailing_ls_hu = paragraphs.get(prev_pi)
    .and_then(|p| p.line_segs.last())
    .map(|s| s.line_spacing.max(0))
    .unwrap_or(0);
let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32
    + trailing_ls_hu;
```

✅ **본질 정합** — TAC 표 직후의 첫 paragraph 가 prev 의 trailing line_spacing 을 누락하던 결함을 정정. 좌표계 (HU/px) 변환 정합.

✅ **검증 완료** (PR 본문):
- 작업지시자 명시 11곳 (P2/P3/P5/P6/P8/P9/P12/P13/P14) 모두 IR vpos delta 와 정확 일치
- 광범위 회귀 (8 핵심 샘플): synam-001 / 복학원서 / exam_math/kor/science / 2010-01-06 = 변경 없음
- exam_eng_002: 151 paragraph +7.68 px shift (양의 shift only = 정합성 개선)

#### Task #539 — 글박스 호스트 직후 paragraph 줄간격 (#537 후속)

**근본 원인**:
- pi=145/pi=181 의 controls 에 `Shape wrap=InFrontOfText tac=true` 보유
- `prev_has_overlay_shape` 가드가 treat_as_char 무관하게 true → 직후 paragraph 의 vpos correction skipped

**정정**:
```rust
Control::Shape(s) => {
    let cm = s.common();
    if cm.treat_as_char {
        return false;  // tac=true 는 LINE_SEG vpos 에 통합 → overlay 영향 없음
    }
    matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
        || ...
}
```

✅ **본질 정합** — `treat_as_char=true` Shape 은 LINE_SEG 의 자연 위치에 통합되므로 overlay 가드 미적용.

✅ **검증** (PR 본문): 7p pi=145→146 + 9p pi=181→182, 14.67 px → 24.21 px (IR 일치).

#### Task #534 v2 — layout_shape_item TAC Picture LINE_SEG.column_start

PR 본문의 상세 설명은 부족 (요약만). v1 의 후속 정정으로 추정. cherry-pick 후 commit 메시지 + Stage 1 보고서로 본질 점검.

### 5.2 회귀 테스트

- PR 본문 보고: TDD 통합 테스트 3건 신규 (`test_537_first_answer_after_tac_table_line_spacing` / `test_539_paragraph_after_overlay_shape_host` / `test_539_partial_paragraph_after_overlay_shape`)
- `cargo test --release --lib` **1119 passed** (PR 본문 보고 일치, 본 환경 PR 브랜치에서도 1119 확인)

### 5.3 외부 영역 정합 점검

본 사이클 누적 정정과의 충돌 가능성:
- **PR #506 (HWP 3.0 파서)**: HWP3 별도 파서, 무관 ✅
- **PR #507 (수식 parser)**: equation 영역, 무관 ✅
- **PR #531 (table_layout.rs)**: TAC 표 + Top caption, 본 PR 의 #534 v1+v2 와 동일 본질이지만 **다른 파일** (table_layout.rs vs layout.rs). 잠재 영향 가능성 점검 필요 🟧

### 5.4 외부 컨트리뷰터 점검

- ✅ @planet6897 — 메인 컨트리뷰터 중 한 명 (HWP 3.0 파서 등 다수 작업)
- ✅ 디버깅 도구 적극 활용 (#534 Stage 1 의 IR vpos delta 비교 + 광범위 샘플 회귀)
- ✅ Stage 별 단계 분리 정합 (수행 → Stage 1 → Stage 2/3 → 보고서)
- ⚠️ **fork devel 에 매우 많은 task 누적** — PR 본문에 명시 안 한 task 들이 같이 들어옴
- 메모리 `feedback_pr_comment_tone` 적용 — 차분/사실 중심

---

## 6. 위험 정리

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| **PR 본문 외 task 의 의도하지 않은 머지** | 🟥 **차단** | 본질 12 commits 만 cherry-pick — 다른 task 들은 컨트리뷰터에게 별도 PR 분리 요청 |
| `mydocs/orders/20260503.md` 충돌 | 🟥 거의 확실 | 양쪽 보존 (수동 해소) — 본 환경 일지에 컨트리뷰터의 #537/#539 진행 기록 통합 |
| `mydocs/orders/20260502.md` 충돌 | 🟧 중간 | PR #507/#531 사례 처럼 auto-merge 정합 가능 |
| `tests/integration_tests.rs` 충돌 | 🟧 중간 | 같은 파일 끝에 새 테스트 추가하는 경우 정합 |
| Task #534 v1+v2 의 누적 정정 위험 | 🟧 중간 | v1 + v2 모두 cherry-pick 후 결정적 검증 |
| Task #534 v2 본문 부족 | 🟧 중간 | cherry-pick 후 commit 메시지 + 보고서 점검 |
| #537/#539 가 이미 close 상태 | 🟢 정합 | PR 머지 후 close 유지 (정정 적용으로 본질 해결) |
| WASM 빌드 영향 | 🟧 중간 | 본 PR 은 native lib 변경. WASM 의 동일 영역 영향 받음 — 머지 전 WASM 빌드 + 시각 판정 2차 필요 |

---

## 7. 결정

**권장**: 🟧 **본질 cherry-pick + 컨트리뷰터 안내** (작업지시자 의도 정합)

**근거:**
1. 본질 정정 (Task #534 v1+v2 + #537 + #539) 은 작고 정합 (`layout.rs` +60 / -4).
2. 회귀 테스트 (TDD 3건 + 통합 테스트) 정합.
3. 광범위 회귀 검증 (8 핵심 샘플) 통과.
4. PR 본문에 명시되지 않은 40 여 task 는 별도 검토 필요 → 본질만 cherry-pick.
5. 컨트리뷰터의 fork devel 동기화 미실행으로 PR 가 비정상 상태 — 다음 PR 전에 동기화 부탁.

**남은 게이트 (작업지시자):**
1. **시각 판정 1차** (SVG, CLI) — Task #534/#537/#539 의 정정 fixture (21_언어 기출 등) 의 SVG 비교
2. **시각 판정 2차** (rhwp-studio web Canvas + 한컴 2010/2020) — PR #507/#531 절차 정합

**머지 시 추가 정합 사항:**
- 이슈 #537/#539 milestone 미지정 → v1.0.0 추가 권장
- 이슈 #534 의 v1+v2 정정 통합 close 또는 별도 처리 결정
- README 기여자 목록 (@planet6897 — PR 카운트 누적, 본 사이클 일괄)
- 컨트리뷰터에게 다음 PR 전 devel 동기화 안내 (PR close 댓글)

---

## 8. PR 본문 산출물 점검

PR 본문 보고 산출물 (cherry-pick 시 함께 들어옴):
- 수행 / 구현 계획서: `mydocs/plans/task_m100_534{,_impl}.md` + `task_m100_537{,_impl}.md` + `task_m100_539{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_534_stage{1,3}.md` + `task_m100_534_v2_stage1.md` + `task_m100_537_stage{1,2,3}.md` + `task_m100_539_stage{1,2,3}.md`
- 최종 보고서: `mydocs/report/task_m100_534_report.md` + `task_m100_537_report.md` + `task_m100_539_report.md`
- 일지: `mydocs/orders/20260502.md` + `20260503.md` (충돌 해소 필요)

---

## 9. 메모리 정합

- `feedback_check_open_prs_first` — 본 PR 처리 정합
- `feedback_pr_comment_tone` — close 댓글 차분/사실 중심
- `feedback_release_sync_check` — cherry-pick 시점 main 동기화 점검 필수
- `feedback_v076_regression_origin` — 컨트리뷰터의 광범위 회귀 검증 정합. 작업지시자 시각 판정도 추가 게이트
- `feedback_visual_regression_grows` — 시각 판정 게이트 (1차 + 2차)
- `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 작업지시자 한컴 2010/2020 직접 판정으로 정답지 확정
- `feedback_assign_issue_before_work` — 이슈 #537/#539/#534 모두 assignee 없음 → 외부 컨트리뷰터에게 "오픈 타스크" 인식 사례. 사전 assign 권장 (사후 처리)
- `feedback_no_close_without_approval` — 이슈 #537/#539/#534 가 작업지시자 승인 없이 컨트리뷰터에 의해 close (PR closes 자동 동작) — 정합한 정정 적용 후 close 유지로 사후 정합 가능
- `feedback_hancom_compat_specific_over_general` — 본 PR 의 정정은 case-specific (TAC 표/글박스/Picture 한정)

---

## 10. 다음 단계

작업지시자 본 검토 문서 승인 후:

1. **시각 판정 1차** (SVG export-svg 출력 자료 준비) — fixture 들의 page 별 SVG
2. 통과 시 `pr_538_review_impl.md` 작성 (cherry-pick 절차 12 commits + 충돌 해소 안내)
3. 작업지시자 승인 후 cherry-pick + 결정적 검증 + WASM 빌드 + studio 동기화
4. **시각 판정 2차** (rhwp-studio web Canvas) — 머지 전 최종 게이트
5. 시각 판정 2차 통과 후 devel 머지 + push
6. `pr_538_report.md` 작성 + PR close (본질 cherry-pick 안내) + 이슈 처리 + **컨트리뷰터에게 다음 PR 전 devel 동기화 부탁 안내**
7. README 기여자 목록 갱신
