---
PR: #707
제목: Task #703 — BehindText/InFrontOfText 표 본문 흐름 누락 정정 (closes #703)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
base / head: devel / pr-task703
mergeStateStatus: BEHIND
mergeable: MERGEABLE — 충돌 0건
CI: ALL SUCCESS
변경 규모: +1285 / -0, 10 files (소스 1 + 통합 테스트 1 + 보고서 5 + sweep TSV 2 + plans 2)
검토일: 2026-05-09
---

# PR #707 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #707 |
| 제목 | Task #703 — BehindText/InFrontOfText 표 본문 흐름 누락 정정 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터 |
| base / head | devel / pr-task703 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — `git merge-tree` 충돌 0건 |
| CI | ALL SUCCESS (Build & Test, CodeQL ×3, Render Diff, Canvas visual diff) |
| 변경 규모 | +1285 / -0, 10 files (소스 1 + 신규 통합 테스트 1 + 보고서 5 + sweep TSV 2 + plans 2) |
| 커밋 수 | 3 (Stage 1 RED + Stage 2 GREEN + Stage 3 sweep + 보고서) |
| closes | #703 |
| 분리 후속 | Issue #704 (별개 결함, `#[ignore]` 처리) |

## 2. 결함 본질 (Issue #703)

### 2.1 결함 메커니즘

`pagination/engine.rs:976-981` 에는 BehindText/InFrontOfText (글뒤로/글앞으로) 표를 Shape 처럼 push 후 `continue` 하는 가드가 있으나, 메인 pagination 인 **`typeset.rs::typeset_table_paragraph` 분기에 동일 가드 미반영**.

→ `place_table_with_text` 영역의 `cur_h += pre_height + table_total_height` 가 BehindText/InFrontOfText 표에도 적용되어 **본문 흐름 cur_h 누적 발생** → trailing 항목이 다음 페이지로 밀림.

### 2.2 영향 샘플
- `samples/basic/calendar_year.hwp` — pi=12 PushButton 이 다음 페이지로 밀림 (HWP/PDF 1p ↔ rhwp 2p)
- 부수 효과: `samples/table-ipc.hwp` 11 InFrontOfText 표 영역 자동 정합 (13 → 10 페이지)

## 3. PR 의 정정

### 3.1 본질 정정 (`src/renderer/typeset.rs:1403`, +13 LOC)

```rust
match ctrl {
    Control::Table(table) => {
        // [Issue #703] 글앞으로 / 글뒤로 표는 Shape처럼 취급 — 본문 흐름 공간 차지 없음.
        // pagination/engine.rs:976-981 와 동일 시멘틱
        if matches!(
            table.common.text_wrap,
            crate::model::shape::TextWrap::InFrontOfText
                | crate::model::shape::TextWrap::BehindText
        ) {
            st.current_items.push(PageItem::Shape {
                para_index: para_idx,
                control_index: ctrl_idx,
            });
            continue;
        }
        // ... 기존 분기
    }
}
```

→ `pagination/engine.rs:976-981` 와 의미 정합. 영향 좁힘 (`text_wrap` 두 변형만 가드).

### 3.2 단위 테스트 (`typeset.rs::tests::test_typeset_703_behind_text_table_no_flow_advance`)

BehindText 1×1 표 (height ≈800 px) + 5 후속 paragraph 영역 — paginator (engine.rs reference) 와 typeset 결과 둘 다 1 페이지 검증. **paginator 와 typeset 두 경로 동치성** 입증.

### 3.3 통합 테스트 (`tests/issue_703.rs`, +54 LOC, 신규)

- `issue_703_calendar_year_single_page` — calendar_year.hwp 1 페이지 검증 (본질 정정 효과)
- `issue_703_tonghap_2010_11_single_page` — `#[ignore]` (Issue #704 별개 결함)
- `issue_703_tonghap_2011_10_single_page` — `#[ignore]` (Issue #704 별개 결함)

## 4. 분리된 후속 — Issue #704

PR 본문 명시:
> 통합재정통계 (2010.11/2011.10) 페이지 분할 결함은 다른 본질로 확인 — TopAndBottom TAC 1×1 + 각주 환경 0.84 px borderline. Issue #704 별도 분리.

→ scope 정확 분리 정합 (`feedback_process_must_follow` 정합 — scope 확장 충동 억제). Issue #704 OPEN, assignee 부재.

## 5. 광범위 회귀 sweep

PR 본문: 196 샘플 SVG/PDF 페이지 수 비교 — 회귀 0, 정합 +2 (calendar_year + table-ipc).
- baseline: `mydocs/report/svg_vs_pdf_diff_20260508.tsv`
- after: `mydocs/report/svg_vs_pdf_diff_20260508_after.tsv`

## 6. 본 환경 fixture 직접 측정

| 파일 | rhwp BEFORE | PDF 권위 | 기대 효과 |
|------|-------------|---------|-----------|
| `samples/basic/calendar_year.hwp` | **2 페이지** | 1 페이지 | 1 페이지 정합 회복 ★ |
| `samples/table-ipc.hwp` | **11 페이지** | 10 페이지 | 10 페이지 정합 회복 (부수 효과) |
| `samples/통합재정통계(2010.11월).hwp` | 1 페이지 (PR #679 영향) | 1 페이지 | 무관 (#704 별개) |

→ PR 본문 효과 영역 본 환경에서 직접 입증 가능.

## 7. 충돌 / mergeable

- `mergeStateStatus: BEHIND` (PR base = `215abb52`, devel HEAD = `805fb48d`, 24 commits 뒤처짐)
- `git merge-tree --write-tree` 실측: **CONFLICT 0건**

## 8. 처리 옵션

### 옵션 A — 3 commits 단계별 보존 cherry-pick + no-ff merge (추천)

PR 의 TDD 절차 (Stage 1 RED → Stage 2 GREEN → Stage 3 sweep + 보고서) 정합. PR #694/#693/#695/#699/#706 패턴 일관.

```bash
git branch local/task707 805fb48d
git checkout local/task707
git cherry-pick b0cb586b^..5c64c03f
git checkout local/devel
git merge --no-ff local/task707
```

→ **옵션 A 추천**.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release --test issue_703` — 1 PASS + 2 ignored (Issue #704)
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets` clean
- [ ] **광범위 sweep** 또는 **직접 페이지 수 측정**: calendar_year 2→1 / table-ipc 11→10 정합 회복 확증

### 시각 판정 게이트 (선택)
- 본 PR 은 BehindText/InFrontOfText 표 영역의 **본문 흐름 누락 정정** — 페이지 분할 영역 영향. 시각 판정 가능:
  - `samples/basic/calendar_year.hwp` 1 페이지 정합 (한컴 PDF 정답지 정합)
  - `samples/table-ipc.hwp` 10 페이지 정합 (한컴 PDF 정답지 정합) — 부수 효과
  - PR #706 의 form-002 영역 회귀 없음 점검 (golden SVG 보존 확증)
- `feedback_visual_judgment_authority` 정합 — 결정적 검증 + 광범위 sweep 통과 후 작업지시자 시각 판정 영역의 안전 게이트

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 (누적 23 머지) 정확 표현 |
| `feedback_hancom_compat_specific_over_general` | `text_wrap` 두 변형만 가드 (영향 좁힘) — 일반화 회피 |
| `feedback_process_must_follow` | TDD Stage 1 RED → Stage 2 GREEN → Stage 3 sweep 절차 정합 + 분리된 후속 (Issue #704) 명확 분리 |
| `feedback_assign_issue_before_work` | Issue #703 / #704 assignee 부재 영역 — 컨트리뷰터 self-등록 패턴 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI ALL SUCCESS + sweep) + 작업지시자 시각 판정 (선택) |
| `feedback_image_renderer_paths_separate` | typeset.rs 분기 + pagination/engine.rs 분기 동기 유지 — 두 경로 동치성 확증 (단위 테스트 명시) |

## 11. 처리 순서 (승인 후)

1. `local/devel` 에서 3 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 신규 issue_703 1 PASS + 직접 페이지 수 측정)
3. 광범위 sweep (선택) 또는 작업지시자 시각 판정 (선택)
4. no-ff merge + push + archives 이동 + 5/9 orders 갱신
5. PR #707 close (closes #703 자동 close 정합)

---

작성: 2026-05-09
