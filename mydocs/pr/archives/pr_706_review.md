---
PR: #706
제목: Task #700 — 셀 paragraph cut 위치 vpos 정합 (compute_cell_line_ranges cum 절대 동기화)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
base / head: devel / task700
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: FAILURE (Build & Test — golden SVG mismatch, **시각 판정 권위 영역으로 정답 정합 — golden 갱신 필요**)
변경 규모: +638 / -3, 7 files (소스 2 + 보고서 5)
검토일: 2026-05-09
---

# PR #706 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #706 |
| 제목 | Task #700 — 셀 paragraph cut 위치 vpos 정합 (compute_cell_line_ranges cum 절대 동기화) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터 |
| base / head | devel / task700 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE |
| CI | **FAILURE** ⚠️ — Build & Test 실패 |
| 변경 규모 | +638 / -3, 7 files (소스 2 + 보고서 5) |
| 커밋 수 | 5 |
| closes | #700 |
| 선행 PR | **PR #701 (Task #697) — close 됨, devel 미반영** ⚠️ |

## 2. 본질 결함 (Issue #700)

`samples/inner-table-01.hwp` cell[11] (사업개요, 26 paras) 의 cell-internal split 시:
- p2 첫 줄에 `- 전사 데이터 수집/유통체계 구축` (`p[17]`) 누락 → rhwp 가 `p[18]` 부터 표시
- 원인: cum 누적 metric (line_height + line_spacing + spacing) 이 한컴 LINE_SEG.vpos 누적과 ~50px 어긋남 → abs_limit (한컴 vpos 단위) 와 비교 시 더 많은 paragraph 가 visible

## 3. PR 의 정정 — 5 commits 분석

### 3.1 본질 commits (Task #700)

```
ee8cc9f1 Task #700 Stage 1: 수행 계획서 + 정밀 진단 보고서
ae8a470e Task #700 Stage 2: 구현 계획서 (옵션 C)
072fc42f Task #700 Stage 3-1: compute_cell_line_ranges cum 절대 동기화
c4d76e0f Task #700 Stage 3-1 보고서 + Stage 4 최종 보고서
```

### 3.2 추가 commit (Task #697 후속, **PR 본문 미명시**)

```
c4676e2b Task #697 후속: split row 미분할 cell 의 valign 보존
```

`src/renderer/layout/table_partial.rs:466` — `is_in_split_row` → `is_in_split_row && cell_was_split` 가드 좁힘. PR 본문에 의도/영향 명시 없음. **PR 범위 외 변경**.

### 3.3 핵심 정정 (`src/renderer/layout/table_layout.rs::compute_cell_line_ranges`, +44 LOC)

```rust
let cell_first_vpos = cell.paragraphs.first()
    .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos))
    .unwrap_or(-1);

if pi > 0 && cell_first_vpos == 0 {
    let prev_end_vpos = prev_para.line_segs.last()
        .map(|s| s.vertical_pos + s.line_height).unwrap_or(-1);
    let cur_first_vpos = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1);
    if cur_first_vpos >= 0 && prev_end_vpos > 0 {
        if cur_first_vpos < prev_end_vpos {
            // [Task #697] vpos 리셋 — page-break 신호
            if has_limit && cum < abs_limit { cum = abs_limit; }
        } else {
            // [Task #700] 정상 누적 — vpos 절대 동기화 (전진만)
            let target_cum = hwpunit_to_px(cur_first_vpos, self.dpi);
            if target_cum > cum { cum = target_cum; }
        }
    }
}
```

→ **Task #697 영역 + Task #700 영역의 통합 정정**. PR #706 의 `compute_cell_line_ranges` 변경이 본 환경 devel 에 부재한 Task #697 영역도 함께 도입.

## 4. CI FAILURE 분석 ⚠️

### 4.1 실패 테스트
`tests/svg_snapshot.rs::form_002_page_0` — 골든 SVG 스냅샷 mismatch.

### 4.2 본 환경 직접 재현
```
$ cargo test --release --test svg_snapshot form_002_page_0
test form_002_page_0 ... FAILED
SVG snapshot mismatch for form-002/page-0.
```

### 4.3 회귀 본질
diff 분석:
```
< [text 26개] "ㅇPFC 나노산소운반체의 최적제조공정개발 및 GMP실증"
  (y=1022.25, expected golden, PR 분기에서 누락)
```

→ `form-002/page-0` 의 분할 표 마지막 visible 줄 26 글자 누락.

### 4.4 PR #662 (Task #656) 본질 정정 회귀

PR #662 의 본질 정정 영역:
- "synam-001 p15 PartialTable OVERFLOW 해소 + form-002 page 0 분할 표 **마지막 visible 줄 26 글자** ('ㅇPFC 나노산소운반체의 최적제조공정개발 및 GMP실증') **클립 해소**"
- 골든 SVG 가 이 정정 영역의 회귀 차단 가드 영역으로 영구 보존됨

PR #706 의 `compute_cell_line_ranges` 영역 정정이 동일 함수 영역의 PR #662 영역을 회귀시킴 — **PR #662 본질 정정 회귀**.

### 4.5 컨트리뷰터 자체 검증 부정합

PR 본문:
> `tests/svg_snapshot.rs` (form-002 포함) ✅ pass

→ **사실과 부정합**. 컨트리뷰터 환경에서는 통과 / CI + 본 환경 모두 실패.

가능성:
- 컨트리뷰터 환경의 stale 골든 SVG (PR #662 머지 이전 버전)
- 컨트리뷰터 의 `cargo test --release` 영역 캐시 문제
- 컨트리뷰터 의 "form-002 포함" 검증 영역 영역 의 다른 골든 영역 (`form-002` 다른 page)

→ `feedback_v076_regression_origin` 권위 사례 강화 영역 — 컨트리뷰터 자체 검증 통과만으로는 회귀 차단 부족, CI + 메인테이너 환경 가드 필수.

## 5. PR #701 (Task #697) 미머지 영향 + 동일 컨트리뷰터 후속 PR 부재

### 5.1 PR #701 close 사유 — 컨트리뷰터 자체 supersede

PR #701 close 시 컨트리뷰터 댓글 (2026-05-08):
> "PR #706 (Task #700) 으로 통합 — 본 PR 의 변경 사항 (Task #697 vpos 리셋 검출 + 결함 2 valign 가드) 모두 PR #706 에 포함. #706 가 동일 함수 (compute_cell_line_ranges) 의 더 정밀한 정합 (cum 절대 동기화) 을 포함하므로 superset 으로 통합. 본 PR 은 close."

→ **PR #701 → PR #706 supersede** (동일 컨트리뷰터 자체 결정). 본 PR 의 5 commits 중 마지막 commit (Task #697 후속 valign 보존) 이 PR #701 영역 의 결함 2 (valign) 통합 영역 정합.

### 5.2 본 환경 영향
- `local/devel` 의 `compute_cell_line_ranges` 영역에 Task #697 영역 **부재**
- PR #706 cherry-pick 시 Task #697 영역 + Task #700 영역 함께 도입 (의도된 통합)

### 5.3 동일 컨트리뷰터 후속 PR 부재 확인

@planet6897 의 모든 PR 영역 (state 무관) 점검:
- Task #700 / Task #697 / `compute_cell_line_ranges` / `inner-table` 영역 **PR #706 가 가장 최신**
- 후속 supersede PR 부재
- Issue #700 도 컨트리뷰터에 의해 자체 close (PR #706 OPEN 시점 영역, PR 미머지)
- Issue #697 도 close (PR #701 close 영역 영역, PR 미머지)

→ **두 Issue 모두 PR 미머지 상태 영역 close 영역** — `feedback_close_issue_verify_merged` 권위 룰 정합 영역. 본 PR close 결정 시 Issue #700 / #697 OPEN 으로 재오픈 권고 영역.

## 6. 메인테이너 시각 판정 ★ 통과 — 정답지 정합 확정

작업지시자 직접 시각 판정 (2026-05-09):
- BEFORE (devel, PR #662 본질 정정 적용 영역, golden SVG `12a6cbcc...`) — 26 글자 ("ㅇPFC 나노산소운반체의 최적제조공정개발 및 GMP실증") 표시
- AFTER (PR #706 적용, `672c78c6...`) — 26 글자 누락
- PDF 권위본 (`pdf/hwpx/form-002-2022.pdf` page 1) 비교 결과 — **AFTER 가 한컴 정답지 정합**

→ **PR #706 이 한컴 권위 정합 영역**. 기존 golden SVG (`tests/golden_svg/form-002/page-0.svg`) 가 PR #662 (Task #656) 의 잘못된 정정 영역 결과 영역 으로 회귀 차단 가드 영역 자체가 부정확 영역.

### 처리 방향 — 옵션 A: PR #706 머지 + golden SVG 갱신

1. PR #706 5 commits cherry-pick (Task #697 + Task #700 통합 정정)
2. golden SVG 갱신 (`tests/golden_svg/form-002/page-0.svg` → `672c78c6...` 반영, `UPDATE_GOLDEN=1` 또는 직접 복사)
3. golden 갱신 commit 추가 (작업지시자 시각 판정 권위 영역 commit message 명시)
4. `cargo test --release` ALL GREEN 확인
5. no-ff merge + push + PR/Issue close + archives + 5/9 orders

### 부수 사실 — PR #662 (Task #656) 의 form-002 정정이 잘못된 영역

PR #662 본문 명시: "form-002 page 0 분할 표 마지막 visible 줄 26 글자 클립 해소" → **사실은 한컴 권위 영역에서는 그 26 글자가 표시되면 안 되는 영역 영역** (PR #706 시각 판정 권위 영역). 후속 별도 영역 분석 필요 가능성 (작업지시자 결정).

## 7. 충돌 / mergeable

- `mergeStateStatus: BEHIND` (PR base = `215abb52`, devel HEAD = `feb11e7a`, 17 commits 뒤처짐)
- 충돌 점검 미수행 (CI FAILURE 영역 영역 점검 우선)

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_close_issue_verify_merged` | PR #701 close → PR #706 supersede 영역의 본질 정정 영역 재검토 영역 |
| `feedback_v076_regression_origin` | 컨트리뷰터 자체 검증 통과 (form-002 포함) 영역 의 CI 영역 영역 본 환경 영역 모두 실패 영역 — 환경 차이 회귀 영역 검출 영역 |
| `feedback_visual_regression_grows` | 골든 SVG (form-002/page-0.svg) 회귀 가드 영역 작동 — CI 차단 영역 |
| `feedback_pr_supersede_chain` | PR #701 (Task #697) close → PR #706 (Task #697 + #700 통합) 영역 신규 패턴 영역 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI) 가 작업지시자 시각 판정 게이트 이전 영역 영역 회귀 차단 영역 |
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 |
| `feedback_pr_comment_tone` | 차분 + 사실 중심 댓글 영역 |

## 9. 처리 결정 — 옵션 A (작업지시자 승인)

작업지시자 시각 판정 ★ 통과 — PR #706 이 한컴 정답지 정합. 옵션 A 진행:

1. local/task706 임시 브랜치 + 5 commits cherry-pick
2. golden SVG (`tests/golden_svg/form-002/page-0.svg`) 갱신 — 작업지시자 시각 판정 권위 영역 commit 추가
3. 자기 검증 (cargo test ALL GREEN, svg_snapshot form-002 통과)
4. no-ff merge + devel push
5. PR #706 close (closes #700 + 수동 close + 한국어 댓글)
6. Issue #700 + Issue #697 close (PR #701 supersede 영역 정합)
7. archives 이동 + 5/9 orders 갱신

---

작성: 2026-05-09
