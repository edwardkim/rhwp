# PR #454 검토 — Task #452 단락 마지막 줄 trailing line_spacing 정합 (exam_kor pi=1↔pi=2)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#454](https://github.com/edwardkim/rhwp/pull/454) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 본 사이클 14번째 PR |
| 이슈 | [#452](https://github.com/edwardkim/rhwp/issues/452) (closes) |
| base / head | `devel` ← `planet6897:local/devel` |
| 변경 규모 | +2,960 / -610, 24 files |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-30 |

## 본질

`paragraph_layout.rs::is_para_last_line` 분기 제거 → **본문 단락 모든 줄에서 `y += lh + ls` 통일** (trailing line_spacing 포함).

### 원인

`is_para_last_line` 분기가 단락 마지막 visible 줄에서 trailing `line_spacing` 을 제외하고 y 를 `lh` 만 전진. 이 y 가 다음 단락 `y_start` 가 되어 다음 단락 첫 줄이 1 ls 만큼 위로 당겨짐.

Task #332 stage 2 의 의도 (typeset 의 `height_for_fit` 와 layout 정합) 가 **layout 만 trailing 제외 → pagination/engine 의 `current_height += para_height` 누적과 1 ls drift** 발생한 절반의 정합. 본 PR 은 layout 도 trailing 포함으로 통일하는 반대 방향 정합.

### 정정

```rust
// Before
let is_para_last_line = cell_ctx.is_none()
    && line_idx + 1 == end
    && end == composed.lines.len();
if (is_cell_last_line && cell_ctx.is_some()) || is_para_last_line {
    y += line_height;
} else {
    y += line_height + line_spacing_px;
}

// After (셀 마지막 줄만 trailing 제외, 본문 단락은 모든 줄 trailing 포함)
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else {
    y += line_height + line_spacing_px;
}
```

### 작성자 명시 효과

- `samples/exam_kor.hwp` 1페이지 pi=1.line9 ↔ pi=2.line0 step **15.34 → 24.50 px** (단락 내 step 과 동일, PDF 정합 1838 HU)

## 처리 방향

**옵션 A — Task #452 본질 4 commits 분리 cherry-pick** (본 사이클 일관 패턴).

작업지시자 결정: 본 PR + 후속 PR (#457 Task #455, #461 Task #459/#462/#463/#468/#469) 가 같은 영역 (paragraph_layout / vpos / col_bottom) 에 누적 정정 진행 정황. **모든 PR 처리 후 통합 시각 검증** 이 효율적 + 위험 분산.

## dry-run cherry-pick 결과

`local/pr454` 브랜치 (`local/devel` 분기) — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `e89a10b` (← `3dd8695`) | @planet6897 | Stage 1: 수행/구현 계획서 + baseline 측정 |
| `8c58c66` (← `aafccb1`) | @planet6897 | Stage 2: paragraph_layout trailing line_spacing 정합 + golden 갱신 |
| `4a30590` (← `12a8b92`) | @planet6897 | Stage 3: 광범위 회귀 검증 — 페이지수/Task #332 회귀 0 |
| `a8f4d5e` (← `5129faa`) | @planet6897 | Stage 4: 최종 결과보고서 + 오늘할일 갱신 |

cherry-pick 결과: 충돌 없이 자동 적용.

## 검증 게이트 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1069 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed (golden 2건 갱신: issue-147, issue-157) |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |

### golden snapshot 갱신 2건

본 PR 의 trailing line_spacing 정합 영향:
- `tests/golden_svg/issue-147/aift-page3.svg` (+207/-207 라인)
- `tests/golden_svg/issue-157/page-1.svg` (+388/-388 라인)

## 광범위 byte 단위 비교

10 샘플 / 305 페이지 SVG 비교 (devel ↔ PR #454):

| 결과 | 카운트 |
|------|------|
| byte 동일 | **39 / 305 (12.8%)** |
| 차이 발생 | **266 / 305 (87.2%)** |

차이 분포 (모든 샘플에 광범위 영향):

| 샘플 | 차이 페이지 |
|------|----------|
| kps-ai | 71 |
| aift | 70 |
| exam_* | 48 |
| 2025년 기부·답례품 | 25 |
| k-water-rfp | 23 |
| synam-001 | 22 |
| biz_plan | 6 |
| equation-lim | 1 |

→ **paragraph_layout 의 단락 마지막 줄 처리** 변경이라 본문 단락이 있는 모든 페이지에 영향. 작성자 PR 본문은 exam_kor 1페이지만 명시했으나 실제는 광범위.

## 시각 판정 정황 (작업지시자 결정)

작업지시자 결정 명시:
> "메인테이너는 이 PR 처리를 끝 낸 후 시각적 검증을 하겠습니다. 이 결정이 합리적이라고 판단합니다."

본 PR 단독 시각 판정 보류 — 후속 PR (#457, #461 5 Tasks) 모두 처리 후 **누적된 모든 정정의 통합 시각 검증** 진행.

이유:
1. 본 PR + 후속 PR 이 같은 영역 (paragraph_layout / vpos / col_bottom) 에 누적 정정
2. 단독 시각 검증 시 후속 정정의 영향 분리 어려움
3. 통합 검증이 작업 효율 + 메모리 `feedback_small_batch_release_strategy` (작은 단위 회전) 부합
4. 메모리 `feedback_v076_regression_origin` 정확 적용 — 광범위 변화 후 작업지시자 직접 시각 검증 게이트

## 본 PR 의 좋은 점

1. **정확한 본질 진단**: `is_para_last_line` 분기가 Task #332 stage 2 의 의도와 어긋나 1 ls drift 발생을 정확히 식별
2. **PDF 정량 측정**: pi=1↔pi=2 step 15.34→24.50 px (24.51 px 단락 내 step 과 정확 일치)
3. **셀 마지막 줄 보존**: `is_cell_last_line && cell_ctx.is_some()` 조건은 그대로 유지 — 셀 높이 모델 영향 없음 (메모리 `feedback_hancom_compat_specific_over_general` 부합)

## 본 PR 의 위험 정황

- **광범위 영향 (87% 페이지)**: 작성자 명시 외 다른 샘플에서도 단락 간 줄간격 변화. 후속 시각 검증에서 회귀 점검 필요
- **PDF 정답지 의존**: 한컴 PDF (200dpi 측정) 가 정답지로 작용 — 메모리 `feedback_pdf_not_authoritative` 균형 고려 필요

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1069 + svg_snapshot 6/6 + clippy 0 |
| 시각 판정 게이트 (push 전 필수) | ⏸️ 후속 PR 처리 후 통합 검증으로 보류 (작업지시자 결정) |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ `is_cell_last_line && cell_ctx.is_some()` 명시 유지 (셀 보존) |
| `feedback_v076_regression_origin` | ⚠️ 광범위 변화 — 통합 시각 검증 필수 |
| 작은 단위 PATCH 회전 | ✅ Task #452 본질 4 commits 만 분리 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr454` 에서 커밋 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr454` → `local/devel` → `devel` 머지 + push
3. PR #454 close + 작성자 댓글 (이슈 #452 자동 close, 후속 PR #457 / #461 처리 후 통합 시각 검증 명시)
4. **PR #457 (Task #455) 처리 진행**

## 참고

- PR: [#454](https://github.com/edwardkim/rhwp/pull/454)
- 이슈: [#452](https://github.com/edwardkim/rhwp/issues/452)
- 후속 PR (같은 영역 누적 정정): [#457](https://github.com/edwardkim/rhwp/pull/457) (Task #455), [#461](https://github.com/edwardkim/rhwp/pull/461) (Task #459/#462/#463/#468/#469)
- 작성자 본 사이클 머지 PR (10건): [#401 v2](https://github.com/edwardkim/rhwp/pull/401)~[#450](https://github.com/edwardkim/rhwp/pull/450)
