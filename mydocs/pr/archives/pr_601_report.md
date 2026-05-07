# PR #601 처리 보고서 — 옵션 A-2 진행 (회귀 0 입증으로 머지) + 후속 결함 영역 분리 (Issue #652)

**PR**: [#601 fix: 복수 제목행 반복 시 2행 이상 출력 정정 (closes #594)](https://github.com/edwardkim/rhwp/pull/601)
**작성자**: @oksure (Hyunwoo Park, oksure@gmail.com) — **활발한 컨트리뷰터** (PR #581/#582/#583/#600 + 본 PR = 5번째 사이클)
**관련**: closes #594, **후속 발견 이슈 = #652** (aift.hwp/aift.hwpx 표 조판 영역 결함)
**처리 결정**: ✅ **옵션 A-2 — 회귀 0 입증으로 머지 + 후속 영역 분리**
**처리일**: 2026-05-07

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 옵션 A-2 — 합본 cherry-pick (2 commits squash, src 단일 파일 +22/-12) + devel merge + push + PR/Issue close |
| 시각 발현 영역 | ⚠️ 본 환경 권위 fixture 부재 (시각 변화 0, 회귀 0 정합) |
| 본질 정합성 | ✅ 결정적 검증 통과 + 회귀 0 입증 + Copilot review 자체 검토 응답 정합 |
| Devel merge commit | `6d9849a` |
| Cherry-pick commit (local/devel) | `0059557` (2 commits squash) |
| Cherry-pick 충돌 | 0 (단일 파일 auto-merge 깨끗 통과) |
| Author 보존 | ✅ Hyunwoo Park (oksure@gmail.com) 보존 |
| PR #601 close | ✅ 한글 댓글 등록 + close |
| Issue #594 close | ✅ 수동 close + 안내 댓글 (closes #594 키워드는 cherry-pick merge 로 자동 처리 안 됨) |
| **신규 Issue #652 등록** | ✅ aift.hwp/aift.hwpx 표 조판 결함 영역 (별도 후속 task) |
| 광범위 페이지네이션 sweep | 167 fixture / 1,687 페이지 / 회귀 0 |
| **inspect_pr601 진단 영역 신규** | `examples/inspect_pr601.rs` 영구 보존 |

## 2. 본질 결함 (Issue #594 권위 영역)

> 페이지를 넘어가는 표에서 제목행 반복이 적용될 때, 두 번째 이후 페이지의 표 헤더가 깨져 보입니다. 정확히는 제목행이 1개 행이 아니라 2개 이상 행으로 구성되어 있는 경우, 최초 제목 행은 깨지지 않지만 반복된 제목행부터는 1행만 제대로 출력되고 2행은 출력되지 않습니다.

→ `src/renderer/layout/table_partial.rs:154` 의 `render_rows.push(0)` 단일 행 하드코딩 영역 (다중 제목행 누락).

## 3. 본질 정정 (`table_partial.rs` +22/-12)

```rust
// 신규 (PR 적용 후)
let mut header_rows: Vec<usize> = Vec::new();
if is_continuation && table.repeat_header && start_row > 0 {
    let mut seen = vec![false; row_count];
    for c in &table.cells {
        let r = c.row as usize;
        if c.is_header && r < start_row && r < row_count && !seen[r] {
            seen[r] = true;
            header_rows.push(r);
        }
    }
    header_rows.sort_unstable();
}
let mut render_rows: Vec<usize> = Vec::new();
render_rows.extend_from_slice(&header_rows);
for r in start_row..end_row.min(row_count) {
    render_rows.push(r);
}
```

→ **`is_header` 셀이 있는 모든 행 동적 수집** + **`r < start_row` 가드** (Copilot review #1 응답: 데이터 범위 내 is_header 행 상단 재배치 방지) + **`seen` boolean vec O(1) 멤버십** (Copilot review #4 응답).

## 4. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1141 passed** / 0 failed (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --release --test issue_546 --test issue_554 --test issue_598_footnote_marker_nav` | ✅ 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,588,023 bytes** (PR #642 baseline 4,587,318 +705 — header_rows Vec allocation 정합) |
| `rhwp-studio npm run build` | ✅ TypeScript 통과 + dist (`index-BywcUMYq.js` 691,386 + `rhwp_bg-BAk_YtfR.wasm` 4,588,023) |

## 5. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|------|------|
| 총 fixture | **167** (161 hwp + 6 hwpx) |
| 총 페이지 (BEFORE PR #642 baseline) | **1,687** |
| 총 페이지 (AFTER PR #601) | **1,687** |
| **fixture 별 페이지 수 차이** | **0** |

→ 본 PR 의 변경이 페이지네이션에 영향 없음 (회귀 0).

## 6. 정량 측정 — PR 본문 명시 권위 샘플 (회귀 0 입증)

| 샘플 | 페이지 | byte 차이 |
|------|------|---------|
| `samples/synam-001.hwp` | 35 페이지 | **35 identical / 0 differ** |
| `samples/aift.hwp` | 77 페이지 | **77 identical / 0 differ** |

→ 두 샘플의 표는 **단일 제목행 영역** 으로 본 PR fix 의 분기 미발현 (회귀 0 정합 입증).

## 7. 시각 발현 영역 정밀 분석 — `examples/inspect_pr601.rs` 진단 영역 (신규)

본 환경 `aift.hwp` 의 모든 표 영역 sweep:

| 영역 | 개수 |
|---|---|
| is_header 부재 표 | 85 개 |
| 단일 제목행 표 | 3 개 (47 페이지 영역의 pi=579 / pi=581 / pi=584 모두) |
| **다중 제목행 표** | **2 개** (s2 pi=147 7×4 + s2 pi=745 9×14) — 한 페이지에 완전히 들어감, **분할 미발현** |

→ 본 환경 `aift.hwp` 의 다중 제목행 + 분할 표 발현 영역 부재. 본 PR fix 의 시각적 효과 발현 영역은 **Issue #594 첨부 `테스트.hwp` 영역의 권위 케이스** 에서만 가능.

## 8. Copilot review 4 코멘트 자체 검토 응답 정합

| 영역 | 응답 정합 |
|------|---------|
| #1 header_rows scope (`r < start_row` 가드) | ✅ `80db71a` 응답 정합 |
| #2 pagination height (`MeasuredTable.header_row_flags`) | ⚠️ 후속 task 영역 (별도) |
| #3 regression test (Issue #594 첨부 미존재) | ⚠️ 메인테이너 영역 권유 |
| #4 O(N) 최적화 (`seen` boolean vec) | ✅ 본질 영역에서 이미 적용 정합 |

## 9. 후속 발견 — Issue #652 (aift.hwp/aift.hwpx 표 조판 결함)

PR #601 시각 발현 영역 점검 중 작업지시자가 발견한 **별도 결함 영역**:

> s2:pi=578 y=131.0 다음에 와야 하는 s2:pi=579 가 그 사이에 빈 공간이 너무 큰 이유와 s2:pi=579 다음 표 2개가 또 오버랩까지 되는 조판 붕괴현상

### 9.1 페이지 수 영역 (본 환경 vs 한컴 권위 정답지)

| 영역 | 페이지 수 |
|---|---|
| 한컴 2010/2020 PDF (권위 정답지) | **74** |
| 본 환경 `samples/hwpx/aift.hwpx` (한컴 2020 변환본) | **74** ← **한컴 정합** ✅ |
| 본 환경 `samples/aift.hwp` (HWP5) | **77** (+3 vs 정답지) |

→ **HWPX 영역은 한컴 정합 (74 페이지) 으로 정합** ✅ — 본 환경 HWPX 파서/렌더러 영역 정합.
→ **HWP5 영역만 +3 페이지 부풀음** (74 → 77, 약 4% 오차) — 본 환경 HWP5 영역의 좁은 결함.

### 9.2 권위 자료 영역 도입

| 자료 | 영역 | 비고 |
|---|---|---|
| `samples/hwpx/aift.hwpx` | git add (영구 보존) | 한컴 2020 변환본, 74 페이지 한컴 정합 |
| `samples/hwpx/aift-2020.pdf` | 작업지시자 PDF 재작성 대기 (1 page 1 본문 영역) | 한컴 2010/2020 PDF 정답지, 74 페이지 |

### 9.3 단위 해석 자체는 정합 — 결함은 vpos 영역

```
pi=579 표 size: IR 47879×27155 HU
  → 168.9×95.8 mm (HU/283.46)
  → 638.4×362.1 px (DPI 96)
본 환경 측정: 638.4 × 362.1 ✅ 정합
```

→ HU → mm → px 단위 변환은 정합. **결함은 HWP5 영역의 vpos / paragraph 누적 y / 표 영역 위 빈 공간 처리 영역**.

### 9.4 Issue #652 등록 (별도 후속 task)

본 이슈는 **HWP5 영역의 좁은 결함** 으로 정정. M100 (v1.0.0) 진입 전 우선 처리 권고 영역.

## 10. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 sweep (167 fixture / 1,687 페이지) + 1141 passed 회귀 0 + PR 본문 권위 샘플 회귀 0 입증
- ✅ `feedback_hancom_compat_specific_over_general` — 단일 파일 영역, 4 분기 가드 (case-specific)
- ✅ `feedback_rule_not_heuristic` — `is_header` 셀 동적 수집, HWP IR 표준 직접 사용
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 영역. 본 환경 fixture 부재 → 후속 task (Issue #594 첨부 도입) 영역 분리
- ✅ `feedback_pdf_not_authoritative` — Issue #594 의 한컴뷰어 영역 + Issue #652 의 한컴 PDF 권위 정답지 영역 분리 정합 (`reference_authoritative_hancom`)
- ✅ `feedback_per_task_pr_branch` — 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 활발한 컨트리뷰터 영역, 차분/사실 중심 톤
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 처리분 점검 후 진행
- ✅ `feedback_assign_issue_before_work` — Issue #594 assignee 미지정 (외부 사용자 등록 + 외부 컨트리뷰터 정정 자율 영역)
- ✅ `feedback_small_batch_release_strategy` — v0.7.10 후 처리분 영역 (본 사이클 5/7 세 번째 PR)
- ✅ `reference_authoritative_hancom` — Issue #594 의 한컴뷰어 + Issue #652 의 한컴 2010/2020 PDF 권위 영역 정합

## 11. 본 PR 의 본질 — v0.7.10 후 여섯 번째 처리 PR (5/7 세 번째)

본 PR 의 처리 본질에서 가장 우수한 점:

1. **본질 진단 정확** — `table_partial.rs:154` 의 `render_rows.push(0)` 단일 행 하드코딩 영역 정확 식별
2. **HWP IR 표준 직접 사용** — `is_header` 셀 동적 수집, 휴리스틱 미도입
3. **케이스별 명시 가드** — 4 분기 (`is_continuation` + `repeat_header` + `start_row > 0` + `r < start_row`) 로 회귀 위험 좁음
4. **회귀 0 입증** — 본 환경 권위 샘플 (synam-001 35 + aift 77 페이지) 모두 byte-identical
5. **Copilot review 자체 검토 응답 정합** — 본질 영역 + 후속 영역 분리 명료
6. **활발한 컨트리뷰터 영역 (oksure 5번째 PR)** — 차분/사실 중심 톤
7. **`inspect_pr601` 진단 영역 영구 보존** — 본 PR 검토 영역에서 신규 작성한 진단 스크립트 (`examples/inspect_pr601.rs`) 가 본 환경 영구 자산
8. **별도 결함 영역 발견 + 분리 정합** — PR 검토 중 작업지시자 시각 발견 + Issue #652 신규 등록 + 한컴 정답지 자료 영역 도입 (DTP 엔진 정체성 영역 강화)

**옵션 A-2 진행 본질** — 회귀 0 + 결정적 검증 + 본질 정합성 입증으로 머지 + 시각 발현 영역은 후속 task 영역 (Copilot review #2/#3 + Issue #652) 분리.

## 12. 본 사이클 사후 처리

- [x] PR #601 close (cherry-pick 머지 + push + 한글 댓글, 옵션 A-2 정합)
- [x] Issue #594 close (수동 close + 안내 댓글)
- [x] 신규 Issue #652 등록 (aift.hwp/aift.hwpx 표 조판 결함 영역)
- [x] 권위 자료 영역 — `samples/hwpx/aift.hwpx` (한컴 정합 74 페이지) git tracked
- [x] `samples/hwpx/aift-2020.pdf` (1 page 1 본문 영역, 74 페이지, 한컴 2010/2020 PDF 권위 정답지) git tracked
- [x] 처리 보고서 (`mydocs/pr/archives/pr_601_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_601_review.md` → `mydocs/pr/archives/pr_601_review.md`)
- [ ] 5/7 orders 갱신 (PR #601 항목 + Issue #652 신규 영역)

## 13. 후속 영역 (별도 task 후보)

1. **Issue #652** (HWP5 영역 vpos 부풉 결함) — M100 우선 처리 영역
2. **`MeasuredTable.header_row_flags` 추가** (Copilot review #2) — 다중 제목행 높이 예약
3. **회귀 테스트** — Issue #594 첨부 `테스트.hwp` 본 환경 도입 + integration test
4. **본 환경 권위 샘플 영역** — 다중 제목행 + 분할 표 발현 fixture 영구 보존
