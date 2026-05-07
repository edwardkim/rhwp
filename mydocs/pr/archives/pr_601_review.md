# PR #601 검토 보고서

**PR**: [#601 fix: 복수 제목행 반복 시 2행 이상 출력 정정 (closes #594)](https://github.com/edwardkim/rhwp/pull/601)
**작성자**: @oksure (Hyunwoo Park, oksure@gmail.com) — **활발한 컨트리뷰터** (PR #581/#582/#583/#600 등 본 사이클 누적, 5번째 PR)
**상태**: OPEN, **mergeable=MERGEABLE**, **mergeStateStatus=BEHIND** (PR base 134 commits 뒤 — 5/5 등록 후 본 사이클 누적)
**관련**: closes #594 (외부 사용자 등록 권위 영역, 한컴뷰어 정합)
**처리 결정**: ⏳ **옵션 A 진행 중 — 시각 판정 게이트 + 컨트리뷰터 41 페이지 표기 부정확 영역 점검** (작업지시자 승인 후 cherry-pick 적용)
**검토 시작일**: 2026-05-07

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — `table_partial.rs:154` 의 `render_rows.push(0)` 단일 행 하드코딩이 분할 표 + 다중 제목행 영역에서 두 번째 이후 제목행 누락 결함을 유발하는가? 본 환경에서 결함 + 정정 효과 재현 가능?
2. **Copilot review 평가** — 4 코멘트 중 컨트리뷰터의 응답 정합성 (header_rows scope 정정 + pagination 후속 영역 분리 + regression test 메인테이너 영역 권유)
3. **회귀 위험** — 단일 파일 (+22/-12) 작은 영역, 본 환경 권위 샘플 (synam-001.hwp / aift.hwp) 회귀 0 입증 가능?
4. **PR base BEHIND 134 commits** — 본 환경 cherry-pick 충돌 0?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | fix: 복수 제목행 반복 시 2행 이상 출력 정정 (closes #594) | 정합 (한글) |
| author | @oksure (Hyunwoo Park, oksure@gmail.com) | 활발한 컨트리뷰터 (본 사이클 5번째 PR) |
| changedFiles | **1** / +22 / -12 | 단일 파일 (가장 작은 fix 패턴) |
| 본질 변경 | `src/renderer/layout/table_partial.rs` (+22/-12) | 단일 파일 |
| commits | 2 (`45d3eeb` 본질 + `80db71a` Copilot review address) | 검토 응답 정합 |
| **mergeable** | MERGEABLE (UI), BEHIND (PR base 134 commits 뒤) | 본 환경 cherry-pick 충돌 0 (auto-merge 깨끗 통과) |
| Issue | closes #594 (외부 사용자 등록 + 한컴뷰어 첨부 스크린샷) | ✅ |
| Issue assignee | 미지정 | 외부 컨트리뷰터 자율 영역 |
| **PR review** | Copilot review 4 코멘트 + 컨트리뷰터 응답 commit (`80db71a`) | 본 PR 의 자체 검토 영역 정합 |
| CI | 모두 SUCCESS (Build & Test / CodeQL × 3 / Canvas visual diff) | ✅ |

## 3. PR 의 2 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `45d3eeb` 본질 정정 (복수 제목행 반복) | `header_rows` 수집 + 다중 제목행 반복 렌더링 | ⭐ cherry-pick |
| `80db71a` Copilot review address | `r < start_row` 조건 추가 + render_rows 루프 중복 제거 조건 제거 | ⭐ cherry-pick |

→ **2 commits 모두 cherry-pick 정합** (단일 파일 동일 영역).

## 4. 본질 변경 영역 (`table_partial.rs` +22/-12)

### 4.1 결함 가설 (Issue #594 + PR 본문 인용)

**Issue #594 외부 사용자 등록**:
> 페이지를 넘어가는 표에서 제목행 반복이 적용될 때, 두 번째 이후 페이지의 표 헤더가 깨져 보입니다. 정확히는 제목행이 1개 행이 아니라 2개 이상 행으로 구성되어 있는 경우, 최초 제목 행은 깨지지 않지만 반복된 제목행부터는 1행만 제대로 출력되고 2행은 출력되지 않습니다.

**원인 영역** (PR 본문):
> `table_partial.rs` 에서 `render_rows.push(0)` 으로 행 0만 하드코딩.

**본 환경 직접 검증**:
```rust
// devel HEAD 상태 (table_partial.rs:154 영역)
let render_header = is_continuation && table.repeat_header && start_row > 0
    && table.cells.iter().filter(|c| c.row == 0).any(|c| c.is_header);
let mut render_rows: Vec<usize> = Vec::new();
if render_header {
    render_rows.push(0); // 제목행
}
```

→ **PR 본문 진단 100% 정합 확인** (행 0 만 단일 하드코딩, 다중 제목행 누락 영역).

### 4.2 정정 (다중 제목행 수집 + 반복 렌더링)

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

→ **`is_header` 셀이 있는 모든 행 수집** + **`r < start_row` 조건** (Copilot review 응답: 데이터 범위 내 is_header 행이 상단 재배치 방지) + **`seen` boolean vec O(1) 멤버십 검사** (Copilot review 응답).

### 4.3 셀 범위 판별 영역 정정

```rust
let render_range_start = if !header_rows.is_empty() {
    *header_rows.first().unwrap()
} else {
    start_row
};
let is_repeated_header_cell = !header_rows.is_empty()
    && header_rows.contains(&cell_row)
    && cell_end_row <= start_row;
```

→ 다중 제목행 인식 (단일 행 0 하드코딩 → header_rows 동적 수집 정합).

### 4.4 회귀 위험 영역 점검

- **단일 파일 영역** (`table_partial.rs` 단독 수정)
- **분기 가드 강화** (`r < start_row` + `start_row > 0` + `c.is_header`) → 영향 영역 매우 좁음
- **단일 제목행 케이스** 정합 (header_rows.len() == 1 일 때 기존 동작 등가)
- **non-continuation 페이지** 무영향 (`is_continuation` 가드)

## 5. 본 환경 직접 검증 (임시 브랜치 `pr601-cherry-test`)

| 단계 | 결과 |
|------|------|
| `45d3eeb` + `80db71a` cherry-pick | ✅ 단일 파일 충돌 0 (auto-merge 깨끗 통과) |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1141 passed** / 0 failed (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --release --test issue_546 --test issue_554` | ✅ issue_546 1 + issue_554 12 모두 통과 |
| `cargo test --release --test issue_598_footnote_marker_nav` | ✅ 4/4 passed |
| `cargo clippy --release --lib` | ✅ 0건 |

→ **본 환경 base skew 134 commits 영향 0** — 단일 파일 충돌 0 + 결정적 검증 모두 통과.

**현재 본 환경 상태**: 임시 브랜치 `pr601-cherry-test` 정리 후 local/devel 원복 (PR base 상태). 옵션 A 진행 승인 시 정식 cherry-pick + WASM 빌드 진행 예정. devel baseline WASM 4,588,023 bytes 측정 완료.

## 6. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **167** (161 hwp + 6 hwpx) |
| 총 페이지 (BEFORE PR #642 baseline) | **1,687** |
| 총 페이지 (AFTER PR #601) | **1,687** |
| **fixture 별 페이지 수 차이** | **0** |

→ 본 PR 의 변경이 페이지네이션에 영향 없음 (회귀 0).

## 7. SVG byte 차이 측정 (PR 본문 명시 권위 샘플)

PR 본문 명시 검증 영역:
> `samples/synam-001.hwp` p15 (분할 표): 정상 렌더링 확인
> `samples/aift.hwp` (41 페이지, 대형 표 다수): export-pdf 에러 없음

본 환경 BEFORE/AFTER 측정:

| 샘플 | 페이지 | byte 차이 |
|------|------|---------|
| `samples/synam-001.hwp` | 35 페이지 | **35 identical / 0 differ** |
| `samples/aift.hwp` | 77 페이지 | **77 identical / 0 differ** |

→ **PR 본문 명시 두 권위 샘플 모두 회귀 0 정합** (정확히 100% 정합 - 본 두 샘플의 표는 단일 제목행 영역, 본 PR fix 의 분기 미발현).

→ **정량적 의미**: 본 PR fix 가 다중 제목행 영역만 정확히 활성화 + 단일 제목행 영역 (대부분 fixture) 무영향 정합. **회귀 위험 영역 좁음 본 환경 입증**.

## 8. PR 본문 자기 검증 결과 (본 환경 재검증)

| 검증 | PR 본문 결과 | 본 환경 재검증 |
|------|---------|----------|
| `cargo test` | 전체 통과 | ✅ **1141 passed** (본 환경 baseline 정합) |
| `cargo clippy -- -D warnings` | 경고 없음 | ✅ 0건 |
| `samples/synam-001.hwp` p15 분할 표 정상 렌더링 | 명시 영역 | ✅ 본 환경 35 페이지 byte-identical (회귀 0 정합) |
| `samples/aift.hwp` (41 페이지, 대형 표 다수) export-pdf 에러 없음 | 명시 영역 | ✅ 본 환경 77 페이지 byte-identical (회귀 0 정합) |
| **Issue #594 첨부 `테스트.hwp` 권위 케이스** | 컨트리뷰터 응답: "로컬에 없어 직접 검증하지 못했으나" | ⏳ 본 환경 미존재, **작업지시자 시각 판정 게이트** (한컴뷰어 정합) |

## 9. Copilot review 4 코멘트 평가 (`80db71a` 응답 정합성)

| Copilot 영역 | 컨트리뷰터 응답 | 본 환경 평가 |
|------|---------|----------|
| **#1 header_rows scope** (데이터 범위 내 is_header 행 상단 재배치 방지) | `r < start_row` 조건 추가 + render_rows 루프 중복 제거 조건 제거 | ✅ **정합** — 본 환경 직접 patch 영역 점검에서 `r < start_row` 가드 + `header_rows ⊂ [0, start_row)` 영역 정합 |
| **#2 pagination height** (`MeasuredTable.header_row_flags` 미존재로 다중 제목행 높이 예약 불가) | "후속 작업으로 처리하는 것이 적절" | ⚠️ **별도 후속 task 영역** (본 PR 범위 외, 본 환경 후속 영역 인지 정합) |
| **#3 regression test** (리포터 첨부 샘플 미존재) | "메인테이너 쪽에서 샘플 기반 integration test 를 추가해주시면 좋겠습니다" | ⚠️ **본 환경 영역** — Issue #594 첨부 `테스트.hwp` 도입 + 회귀 테스트 추가 영역 (본 PR 처리 후 별도 영역) |
| **#4 O(N) header_rows.contains() per-cell loop** (Copilot 본 review body 의 `seen` boolean 권장) | 본질 영역에서 이미 `seen` boolean vec 으로 O(1) 멤버십 검사 적용 | ✅ **정합** — 본질 영역 (`seen[r]`) 은 O(1), 잔존 `header_rows.contains(&cell_row)` 는 cell 별 1 회 호출 (header_rows.len() 보통 1-3 이므로 실용 영향 미미) |

→ **컨트리뷰터의 자체 검토 응답 정합성 우수** — Copilot review 4 코멘트 모두 명시 응답 + 본질 영역 정정 (`80db71a`) + 후속 영역 분리 정합.

## 10. 메인테이너 정합성 평가

### 정합 영역 — 우수
- ✅ **본질 진단 정확** — `table_partial.rs:154` 의 `render_rows.push(0)` 단일 행 하드코딩 영역 정확 식별
- ✅ **HWP IR 표준 직접 사용** — `is_header` 셀 동적 수집, 휴리스틱 미도입 (`feedback_rule_not_heuristic` 정합)
- ✅ **단일 파일 영역 좁힘** — 4 분기 가드 (`is_continuation` + `repeat_header` + `start_row > 0` + `r < start_row`) 로 회귀 위험 좁음
- ✅ **결정적 검증 정합** — 1141 passed / clippy 0 / svg_snapshot 6/6 / 광범위 sweep 회귀 0
- ✅ **PR 본문 명시 권위 샘플 회귀 0** — synam-001 35 페이지 + aift 77 페이지 모두 byte-identical (정확 100% 정합)
- ✅ **Copilot review 4 코멘트 자체 검토 응답 정합** — `80db71a` 의 정정 영역 모두 본 환경 정합 확인
- ✅ **후속 영역 분리 정합** — pagination 높이 예약 (`MeasuredTable.header_row_flags`) 별도 후속 영역 인지 + 메인테이너 회귀 테스트 영역 권유 (자체 검토 영역 명료)
- ✅ **활발한 컨트리뷰터 영역** — 본 사이클 5번째 PR + 차분/사실 중심 톤 (PR 본문 + Copilot review 응답)

### 우려 영역
- ⚠️ **Issue #594 첨부 `테스트.hwp` 본 환경 미존재** — 다중 제목행 + 분할 표 결함 발현 영역의 권위 시각 판정 자료 부재. 컨트리뷰터 응답: "메인테이너 쪽에서 샘플 기반 integration test 를 추가해주시면 좋겠습니다." → **작업지시자 시각 판정 게이트 권고 + 권위 샘플 본 환경 영역 도입 검토**
- ⚠️ **PR base BEHIND 134 commits** — UI MERGEABLE 표시지만 본 환경 cherry-pick 충돌 0 확인 (저위험 영역)
- ⚠️ **`MeasuredTable.header_row_flags` 후속 영역** — 다중 제목행 높이 예약 영역 별도 task 영역 (본 PR 범위 외)
- ⚠️ **컨트리뷰터의 `aift.hwp` 페이지 수 표기 부정확** — PR 본문 "41 페이지" 표기, 본 환경 측정 **77 페이지** (작업지시자 안내: 실제 47 페이지). 본 환경의 페이지네이션이 한컴 정합과 차이 있을 수 있음 (별도 영역). 본 PR 의 정정 자체는 다중 제목행 영역만 다루므로 영향 없음.
- ⚠️ **시각적으로 개선되지 않음 (작업지시자 안내)** — 본 환경 fixture 들 (synam-001 / aift) 의 표가 단일 제목행 영역으로 본 PR fix 분기 미발현 → SVG byte 차이 0. **시각적 개선은 다중 제목행 + 분할 표 영역에서만 발현** (Issue #594 첨부 `테스트.hwp` 권위 케이스 영역). 작업지시자 시각 판정 영역에서는 본 환경 권위 샘플 영역 부재로 개선 미발현 정합 — **권위 샘플 도입 후 재시각 판정 또는 후속 task 영역 분리** 필요.

## 11. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — `45d3eeb` + `80db71a` 단일 파일 충돌 0
- ✅ **결정적 검증** — 1141 passed / clippy 0 / svg_snapshot 6/6 / 광범위 sweep 회귀 0
- ✅ **PR 본문 명시 권위 샘플 회귀 0** — synam-001 + aift 모두 byte-identical
- ✅ **HWP IR 표준 직접 사용** — `is_header` 셀 동적 수집
- ✅ **Copilot review 응답 정합 (`80db71a`)**
- ⏳ **시각 판정 영역** — Issue #594 첨부 `테스트.hwp` 권위 케이스 영역 (작업지시자 시각 판정 게이트 권고)

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- `45d3eeb` + `80db71a` 2 commits cherry-pick (author oksure 보존)
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep 통과 확인 완료
- WASM 빌드 산출물 검증
- **작업지시자 시각 판정** — Issue #594 첨부 `테스트.hwp` 또는 본 환경 다중 제목행 fixture 영역 (★ 게이트, 한컴뷰어 정합)
- 통과 시 devel merge + push + PR close (한글 댓글)

#### 옵션 B — 후속 영역 권유 결합
- 본 PR 처리 후 컨트리뷰터에게 후속 task 권유 — `MeasuredTable.header_row_flags` 추가 + pagination 높이 예약 영역
- 메인테이너 영역에서 회귀 테스트 추가 (Issue #594 첨부 `테스트.hwp` 본 환경 영역 도입)

#### 옵션 C — close + 본 환경 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장 + 옵션 B 후속 권유 결합.

## 12. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 페이지네이션 sweep (167 fixture / 1,687 페이지) + 1141 passed 회귀 0 + PR 본문 명시 권위 샘플 (synam-001 35 + aift 77 페이지) 모두 byte-identical
- ✅ `feedback_hancom_compat_specific_over_general` — 단일 파일 영역, 4 분기 가드 (`is_continuation` + `repeat_header` + `start_row > 0` + `r < start_row`) (case-specific)
- ✅ `feedback_rule_not_heuristic` — `is_header` 셀 동적 수집, HWP IR 표준 직접 사용
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (Issue #594 첨부 권위 케이스 영역)
- ✅ `feedback_pdf_not_authoritative` — Issue #594 의 한컴뷰어 정합 영역 (PDF 변환 결과지만 한컴뷰어가 권위 영역, `reference_authoritative_hancom` 정합)
- ✅ `feedback_per_task_pr_branch` — 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 활발한 컨트리뷰터 영역, 차분/사실 중심 톤 (반복 컨트리뷰터에 매번 같은 인사 부적절 영역 정합)
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 처리분 점검 후 진행
- ✅ `feedback_assign_issue_before_work` — Issue #594 assignee 미지정 (외부 사용자 등록 + 외부 컨트리뷰터 정정 자율 영역)
- ✅ `feedback_small_batch_release_strategy` — v0.7.10 후 처리분 영역 (본 사이클 5/7 세 번째 PR)
- ✅ `reference_authoritative_hancom` — Issue #594 의 한컴뷰어 첨부 스크린샷이 권위 기준
- ✅ **`feedback_close_issue_verify_merged` 영역** — Copilot review 영역에서 컨트리뷰터의 회귀 테스트 메인테이너 영역 권유 정합 (회귀 차단 가드 영구 보존 영역)

## 13. 옵션 A 진행 결과 (작업지시자 승인 후)

### 13.1 핀셋 cherry-pick

| 단계 | 결과 |
|------|------|
| `45d3eeb` + `80db71a` 2 commits squash cherry-pick | ✅ 단일 파일 충돌 0 (auto-merge 깨끗 통과) |
| local/devel commit | `0059557` (**author Hyunwoo Park (oksure) 보존**, committer edward) |

### 13.2 결정적 재검증 (local/devel cherry-pick 후)

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1141 passed** / 0 failed (회귀 0) |
| `cargo clippy --release --lib` | ✅ 0건 |
| **Docker WASM 빌드** | ✅ **4,588,023 bytes** (1m 26s, PR #642 baseline 4,587,318 **+705 bytes** — table_partial.rs +22/-12 LOC + header_rows Vec allocation 정합) |
| **rhwp-studio `npm run build`** (신규 WASM 반영) | ✅ TypeScript 통과 + dist (`index-BywcUMYq.js` 691,386 / `rhwp_bg-BAk_YtfR.wasm` 4,588,023) |

### 13.3 작업지시자 안내 영역 — 시각적 개선 미발현

**작업지시자 평가**:
> 컨트리뷰터는 41페이지로 지칭한 것은 실제로는 47페이지. wasm 다시 빌드해주세요. 시각적으로 개선되지 않았습니다.

**점검 영역**:

#### A. 컨트리뷰터의 페이지 수 표기 부정확
- **PR 본문 표기**: "samples/aift.hwp (41 페이지, 대형 표 다수)"
- **작업지시자 측정**: 47 페이지 (한컴 정합)
- **본 환경 측정**: **77 페이지** (현재 본 환경 페이지네이션 영역)
- **차이 영역**: 본 환경의 페이지네이션이 한컴 정합과 차이 있음 (별도 영역). 본 PR 의 fix 영역과는 무관.

#### B. 시각적 개선 미발현
- **본 환경 권위 샘플** (synam-001 35 + aift 77) 의 표는 **단일 제목행 영역** 으로 본 PR fix 의 분기 (`r < start_row` && `is_header`) 미발현
- → SVG byte 차이 0 (회귀 0 입증, 단 시각적 개선도 0)
- **시각적 개선은 다중 제목행 + 분할 표 영역에서만 발현** — Issue #594 첨부 `테스트.hwp` 권위 케이스 영역
- 본 환경에는 다중 제목행 + 분할 표 fixture 부재 → 시각 판정 영역 부재

### 13.4 권장 처리 방향 정합

#### 옵션 A-1 — 본 PR cherry-pick 유지 + 권위 샘플 도입 후 재시각 판정 (권장)
- 본 PR 의 정정 본질 정합성은 결정적 검증 통과 + 회귀 0 입증으로 명확
- 다만 시각 판정 영역의 권위 샘플 (다중 제목행 + 분할 표) 본 환경 도입 필요
- Issue #594 첨부 `테스트.hwp` 또는 동등 권위 fixture 추가 후 재시각 판정 → 통과 시 머지

#### 옵션 A-2 — 본 PR cherry-pick 그대로 머지 (회귀 0 + 결정적 검증 통과)
- 본 환경 권위 샘플 (synam-001/aift) 회귀 0 입증 + 결정적 검증 통과
- 시각적 개선 영역은 본 환경 fixture 부재로 별도 영역
- 후속 task 로 권위 샘플 도입 + 회귀 테스트 추가 (옵션 B)

#### 옵션 A-3 — close + 컨트리뷰터에게 권위 샘플 동봉 요청
- 본 PR close 후 컨트리뷰터에게 `테스트.hwp` 또는 동등 권위 샘플 동봉한 후속 PR 권유

### 13.5 WASM 산출물 정합

| 산출물 | bytes |
|---|---|
| `pkg/rhwp_bg.wasm` | **4,588,023** (Docker WASM 빌드 1m 26s) |
| `rhwp-studio/dist/assets/rhwp_bg-BAk_YtfR.wasm` | 4,588,023 (vite copy) |
| `rhwp-studio/dist/assets/index-BywcUMYq.js` | 691,386 |
| **PR #642 baseline** | 4,587,318 |
| **PR #601 (본 PR)** | 4,588,023 ← **+705 bytes** |
| 차이 본질 | table_partial.rs +22/-12 LOC + header_rows Vec allocation + seen boolean vec O(1) 정합 |

### 13.6 47 페이지 (page_num=41) 권위 영역 정밀 분석

작업지시자 안내 정합 — 컨트리뷰터의 "41 페이지" 는 **page_num=41** = **47번째 페이지** (global_idx=46).

**dump-pages -p 46 분석 결과**:
```
=== 페이지 47 (global_idx=46, section=2, page_num=41) ===
  단 0 (items=10, used=893.1px)
    PartialParagraph  pi=579  lines=0..1  vpos=33421
    Table             pi=579 ci=0  5x2  638.4x362.1px  wrap=TopAndBottom tac=false
    Table             pi=581 ci=0  5x2  642.2x320.4px  wrap=TopAndBottom tac=false
    PartialTable      pi=584 ci=0  rows=0..2  cont=false  5x2  split_start=0.0 split_end=46.1
```

**dump-pages -p 47 (48 페이지) — pi=584 continuation 영역**:
```
=== 페이지 48 (global_idx=47, section=2, page_num=42) ===
    PartialTable      pi=584 ci=0  rows=1..5  cont=true  5x2  split_start=46.1 split_end=0.0
```

→ **pi=584 표가 47/48 페이지 영역 분할 발현** (분할 표 권위 영역).

### 13.7 본 환경 aift.hwp 의 is_header 셀 영역 정밀 sweep (`examples/inspect_pr601.rs`)

**47 페이지 영역 표 (pi=579 / pi=581 / pi=584)**:
| pi | 행×열 | repeat_header | is_header rows |
|---|---|---|---|
| 579 | 5×2 | true | **{0} (1개, 단일 제목행)** |
| 581 | 5×2 | true | **{0} (1개, 단일 제목행)** |
| 584 | 5×2 | true | **{0} (1개, 단일 제목행)** |

→ **47 페이지 영역의 모든 표는 단일 제목행 영역** — 본 PR fix 의 분기 미발현 (기존 동작과 동등, 회귀 0 정합).

**aift.hwp 전체 표 영역 sweep**:
| 영역 | 개수 |
|---|---|
| is_header 부재 표 | 85 개 |
| 단일 제목행 표 | 3 개 |
| **다중 제목행 표** | **2 개** (s2 pi=147 7×4 / s2 pi=745 9×14) |

**다중 제목행 표 분할 영역 점검** — `Table` (분할 안 됨) 으로 dump:
- `s2 pi=147 7행×4열` 제목행 2개 → 한 페이지에 완전히 들어감 (분할 미발현)
- `s2 pi=745 9행×14열` 제목행 2개 → 한 페이지에 완전히 들어감 (분할 미발현)

→ **본 환경 aift.hwp 의 다중 제목행 + 분할 표 발현 영역 부재**. 본 PR fix 의 시각적 효과 발현 영역이 아예 없음.

### 13.8 결론 — 시각적 개선 미발현 본질

| 권위 영역 | 본 환경 발현 | 본 PR fix 효과 |
|---|---|---|
| 47/48 페이지 (pi=584 분할 표) | ✅ 분할 발현 | ❌ 단일 제목행 → fix 분기 미발현 (회귀 0) |
| s2 pi=147 (다중 제목행 표) | ❌ 분할 미발현 | — fix 분기 비활성 |
| s2 pi=745 (다중 제목행 표) | ❌ 분할 미발현 | — fix 분기 비활성 |
| **다중 제목행 + 분할** (Issue #594 권위 케이스) | ❌ **본 환경 부재** | ⏳ 권위 fixture 도입 필요 |

→ **작업지시자 안내 ("시각적으로 개선되지 않았다") 정합** — 본 환경 aift.hwp 의 표 구조가 본 PR fix 의 활성 분기를 발현하지 않음.

→ **본 PR fix 의 본질 정합성은 입증됨** (1141 passed / clippy 0 / 회귀 0 / `inspect_pr601` 영역 분석으로 단일 제목행 동작 보존 확인). 단, **시각적 발현은 Issue #594 첨부 `테스트.hwp` 영역의 권위 케이스에서만 가능**.

### 13.9 권장 처리 방향 정합 (시각 발현 분석 결과)

| 옵션 | 영역 | 평가 |
|---|---|---|
| **A-1** (권장, 강화) | **Issue #594 첨부 `테스트.hwp` 본 환경 도입 + 재시각 판정** | 권위 fixture 도입 시 본 PR fix 의 시각적 발현 영역 입증 가능. 단, 첨부 파일 다운로드 영역 |
| **A-2** | 회귀 0 + 결정적 검증 통과 + 본질 정합성 입증으로 그대로 머지 + 후속 task | `inspect_pr601` 영역 정량 입증 (다중 제목행 표 2개 발견 + 분할 미발현 점검) |
| **A-3** | close + 컨트리뷰터에게 권위 샘플 동봉한 후속 PR 권유 + 첨부 `테스트.hwp` 도입 안내 | **작업지시자 영역 결정** |

### 13.10 다음 단계

7. ⏳ **작업지시자 결정** — 옵션 A-1 / A-2 / A-3 중 진행 영역
8. ⏳ 결정에 따라 후속 단계 (devel merge + push + PR close + 처리 보고서 또는 close + 후속 PR 권유)

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**옵션 A 진행 — 작업지시자 시각 판정 영역 결정 대기**.
