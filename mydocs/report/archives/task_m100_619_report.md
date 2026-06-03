# 최종 결과 보고서 — Task #619

다단 paragraph 내 vpos-reset 미처리 — TypesetEngine partial-split 에서 `line.vertical_pos == 0` 무시

- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task619` → fork `planet6897:pr-task619`
- 이슈: https://github.com/edwardkim/rhwp/issues/619
- 처리 일자: 2026-05-06

## 1. 증상

`samples/21_언어_기출_편집가능본.hwp` 페이지 8 우측 단(단 1) 마지막 줄(pi=181 line 8) 이 본문 영역 하단을 17.1 px 초과한 위치에 그려져 라인 절반 이상이 꼬리말 영역으로 빠져 보임.

```
LAYOUT_OVERFLOW_DRAW: section=0 pi=181 line=8 y=1453.2 col_bottom=1436.2 overflow=17.1px
```

## 2. 원인

`pi=181 line_segs[8].vertical_pos = 0` (line>0 인데 vpos=0) 은 HWP 가 line 8 을 다음 단/페이지 최상단에서 시작하도록 인코딩한 vpos-reset 신호다. 한컴 2020 PDF 도 이 신호에 따라 page 8 단 1 = pi=181 line 0..8 (8줄), page 9 단 0 = line 8..13 분포로 렌더.

활성 페이지네이션 엔진 `TypesetEngine` (`src/renderer/typeset.rs`) 의 partial-split 루프 (line 1077–1093) 가 문단 *내부* `line.vertical_pos == 0` 신호를 인식하지 않음. 문단 *간* vpos-reset 만 `next_will_vpos_reset` (line 444) 으로 처리.

`paragraph_layout.rs:1733` 의 주석에서 한계 명시:
> *"vpos-reset 미지원으로 paragraph 가 col_bottom 너머에 layout 될 수 있는데…"*

## 3. 정정 (A 안 — 가장 보수적)

`TypesetEngine::typeset_paragraph` 의 partial-split 루프 (`src/renderer/typeset.rs:1077-1093`) 의 inner fit 루프에 vpos-reset forced break 검출을 추가.

```rust
for li in cursor_line..line_count {
    // [Task #619] 다단 paragraph 내 vpos-reset 강제 분리.
    // line_segs[li].vertical_pos == 0 (li>0) 은 HWP 가 해당 line 을
    // 다음 단/페이지 최상단에 배치하도록 인코딩한 신호.
    // 다단 한정 적용 — 단일 단은 partial-table split 회귀 (issue #418) 차단 위해 미적용.
    if st.col_count > 1
        && li > cursor_line
        && para.line_segs.get(li).map(|s| s.vertical_pos == 0).unwrap_or(false)
    {
        break;
    }
    let content_h = fmt.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line {
        break;
    }
    cumulative += fmt.line_advance(li);
    end_line = li + 1;
}
```

**변경 LOC**: +9 / -0.

### 3.1 적용 범위

| 조건 | 의도 |
|------|------|
| `st.col_count > 1` | 다단 섹션 한정. 단일 단은 partial-table split (issue #418) 회귀 차단 |
| `li > cursor_line` | 세그먼트 첫 줄 제외 (forced break 후 cursor 가 vpos-reset line 부터 시작 시 무한 루프 방지) |
| `vertical_pos == 0` | HWP vpos-reset 신호 |

## 4. 검증

### 4.1 대상 파일 정합

| 항목 | 변경 전 | 변경 후 |
|------|---------|---------|
| 페이지 8 단 1 pi=181 | `lines=0..9` | **`lines=0..8`** ✓ |
| 페이지 9 단 0 pi=181 | `lines=9..13` | **`lines=8..13`** ✓ |
| LAYOUT_OVERFLOW_DRAW (텍스트) | 17.1 px | **사라짐** ✓ |
| LAYOUT_OVERFLOW (PartialParagraph bbox) | 26.6 px | 2.4 px (잔여, 별도 이슈 후보) |
| LAYOUT_OVERFLOW (Shape bbox) | 26.6 px | 2.4 px |

### 4.2 한컴 PDF 정합

| 환경 | pi=181 페이지 8 줄 수 | 페이지 9 첫 줄 |
|------|----------------------|---------------|
| 한컴 **2010** PDF | 5줄 (line 0..5) | "인도하고…" |
| 한컴 **2020** PDF | **8줄 (line 0..8)** | "토속 신앙의…" |
| **변경 후 rhwp** | **8줄 (line 0..8)** ✓ | "토속 신앙의…" ✓ |

→ 변경 후 결과는 **한컴 2020 PDF 와 정확히 일치**. HWP `vpos-reset` 인코딩 의도 = 한컴 2020 분포.

메모리 노트 `feedback_pdf_not_authoritative` 룰 ("PDF 200dpi 는 보조 ref. 한컴 2010/2020 환경 차이 함께 점검") 정합.

### 4.3 회귀 검증

- `cargo test --release --lib` 통과.
- `cargo clippy --release --all-targets -- -D warnings` 통과 (신규 warning 0).
- 회귀 가드 샘플 10개 (HEAD~1 빌드 vs 변경 후 빌드 자동 비교):

| 샘플 | LAYOUT_OVERFLOW (전/후) | 페이지 분포 |
|------|------------------------|-------------|
| `exam_eng.hwp` (Task #470 가드) | 3 / 3 동일 | 동일 |
| `exam_kor.hwp` (issue #418 가드) | 1 / 1 동일 | 동일 |
| `exam_science.hwp` (Task #568 가드) | 1 / 1 동일 | 동일 |
| `exam_math.hwp` | 0 / 0 동일 | 동일 |
| `exam_social.hwp` | 0 / 0 동일 | 동일 |
| `hwp-multi-001.hwp` (Task #470 가드) | 1 / 1 동일 | 동일 |
| `hwp-multi-002.hwp` | 0 / 0 동일 | 동일 |
| `k-water-rfp.hwp` (Task #361 가드) | 2 / 2 동일 | 동일 |
| `kps-ai.hwp` (Task #362 가드) | 7 / 7 동일 | 동일 |
| `aift.hwp` (77p 다단) | 8 / 8 동일 | 동일 |

→ **모든 회귀 가드 샘플의 overflow + 페이지 분포 변경 전후 완전 동일**.

## 5. 잔여 사항

- **PartialParagraph/Shape bbox 2.4 px overflow**: 텍스트 글리프는 단 안 (`LAYOUT_OVERFLOW_DRAW` 미발생). bbox 가 line_spacing trail 까지 포함하는 기하학적 잔여. 본 Task 핵심 증상과 무관 — 별도 이슈 분리 후보.

## 6. 메모리 정합

- `feedback_pr_target_devel` — PR base 는 `devel`. main 은 릴리즈 전용
- `feedback_per_task_pr_branch` — 각 PR 은 별도 fork branch (`planet6897:pr-task619`)
- `feedback_pdf_not_authoritative` — 한컴 2010 vs 2020 환경 차이 함께 점검 (본 정정은 2020 정합 방향)
- `feedback_essential_fix_regression_risk` — 다단 한정 + 광범위 회귀 가드 검증
- `feedback_rule_not_heuristic` — HWP 인코딩 신호 (vpos-reset) 존중 단일 룰. 분기/허용오차 없음

## 7. 변경 파일 목록

| 파일 | LOC |
|------|-----|
| `src/renderer/typeset.rs` | +9 |
| `mydocs/plans/task_m100_619.md` | 신규 |
| `mydocs/plans/task_m100_619_impl.md` | 신규 |
| `mydocs/working/task_m100_619_stage1.md` | 신규 |
| `mydocs/working/task_m100_619_stage2.md` | 신규 |
| `mydocs/working/task_m100_619_stage3.md` | 신규 |
| `mydocs/report/task_m100_619_report.md` | 신규 |
| `mydocs/orders/20260506.md` | 신규 |
