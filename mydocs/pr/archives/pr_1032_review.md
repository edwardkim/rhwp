# PR #1032 검토 — Task #1027: 세로 측정 정합 (페이지네이터↔렌더러, 노트 8쪽)

- PR: [#1032](https://github.com/edwardkim/rhwp/pull/1032)
- 작성자: @planet6897 (Jaeook Ryu, 22번째 PR — 분할 표·측정 시리즈 마무리)
- closes #1027 (M100, v1.0.0, 메인테이너 작성 이슈, assignee 미지정)
- base: devel (PR base 시점 `7ec2e25f`, 현재 origin/devel = `d359c302`)
- head: pr-task1027 (단일 squash `c670f929`)
- mergeable: MERGEABLE, CI 전체 통과 (Build & Test / CodeQL / Canvas visual diff)
- 변경 규모: +1562/-235, 23 파일 (4 코드 + 19 문서)
- 일시: 2026-05-20

## 1. 컨트리뷰터 사이클 + 시리즈 위치 (`feedback_contributor_cycle_check`)

@planet6897 22 PR 누적 (`gh pr list --author planet6897 --state all` 기반):

- PR #1003 (closes #990) — 빈 문단 위 TAC 글상자 advance 이중 가산 정정 (5/20 머지)
- PR #1004 (Refs Task #991) — 분할 표 렌더링 부분 적용 (5/20 머지)
- PR #1024 (closes #1022) — 측정 정합: RowCut 모델 + LAYOUT_OVERFLOW 42→12 (5/20 머지)
- **PR #1032 (closes #1027)** — 세로 측정 정합: 페이지네이터↔렌더러 노트 8쪽 (본 PR)
- PR #1033 (closes #1025) — 페이지보다 큰 단일 표 셀 내부 분할 (OPEN, 본 PR 후속)

**분할 표·측정 시리즈** (#990→#991→#1022→#1027→#1025) 의 4번째 마디. PR 본문 명시 "비범위: 분할표 잔여행 advance" + "다단 측정 정합" 등 후속 명확.

## 2. 이슈 #1027 배경 (메인테이너 작성)

세로 측정/페이지네이션 정합 결함 2종:
- **증상 A** (확인된 버그): 페이지네이터 과측정 → 콘텐츠가 다음 쪽으로 밀림. 노트 "추진일정은 …" (21.3px) 이 한컴 2022 PDF 8쪽 하단인데 rhwp 9쪽 맨 위로 밀림. 페이지네이터 used≈925.8px (15.3px 여유) → 21.3px 거부, 렌더러는 더 낮게 그림 → ~6px 과측정.
- **증상 B**: 박스(TAC Shape) 아래 세로 여백 부족. 미세 (수 px) — Stage 1 재판정.

PR 본문 명시 검증 권위: AI 184p (비공개 RFP) 한컴 2022 PDF 대조 — 본 환경 미접근 fixture.

## 3. 본질 — 공유 측정 엔진 (Stage A~F)

PR 의 핵심 아이디어 = **VPOS_CORR 상태머신을 `HeightCursor` 로 추출 후 페이지네이터·렌더러 양쪽 호출** → 두 측정 공간 일치 → 페이지 분할이 렌더 결과와 정합.

### 3.1 신규 모듈 `src/renderer/height_cursor.rs` (+333, 신규)

```rust
pub(crate) struct HeightCursor {
    pub dpi: f64,
    pub col_area_y: f64,
    pub col_area_height: f64,
    pub col_anchor_y: f64,
    pub vpos_page_base: Option<i32>,
    pub vpos_lazy_base: Option<i32>,
    pub prev_layout_para: Option<usize>,
    pub prev_item_was_partial_table: bool,
}
impl HeightCursor {
    pub(crate) fn vpos_adjust(&mut self, y_offset, item_para, paragraphs, styles) -> f64;
}
```

`vpos_adjust` 함수가 `layout.rs build_single_column` 의 inter-item VPOS_CORR 블록과 동작 1:1:
- page_path/lazy_path base 분리 (#412)
- trailing_ls 조건부 보정 (#1022 v2)
- 백워드 클램프 ≤8px (#643)
- overlay-shape/PartialTable bypass (#409/#991)
- stale TopAndBottom forward jump 가드 (#874 #8)

**parity 단위 테스트 7개**: no_prev / same_para / partial_bypass / vpos_reset_bypass / page_path_applied / page_path_sb_prededuct / lazy_path_applied_and_base_set / backward_clamp_rejected. 클램프 경계·기준점 산출·base 갱신을 손계산 정합으로 검증.

### 3.2 `src/renderer/layout.rs` (+96/-233)

230줄의 inline `if let Some(prev_pi) = prev_layout_para {…}` 블록 제거 → `HeightCursor::vpos_adjust` 위임. 클램프 로직은 신규 순수 함수 `vpos_corrected_end_y()` + `para_has_overlay_shape()` 로 추출 (Stage A/B).

**Stage C 무동작**: PR 본문 명시 `svg_snapshot 8/8 유지`. parity 단위 테스트로 추출 정합 보장.

### 3.3 `src/renderer/typeset.rs` (+119/-2)

**Stage D 동작 변경 (핵심)**: `format_paragraph` 직전 `vpos_snap_current_height` 호출 — `current_height` 를 vpos 정합 위치로 스냅 → 누적 drift 제거.

```rust
fn vpos_snap_current_height(&self, st: &mut TypesetState, para_idx, ...) {
    if st.col_count != 1 { return; }  // 다단은 Stage E (#412 per-column base 선행 필요)
    if st.current_items.is_empty() {
        st.vpos_col_anchor = st.current_height;
        st.vpos_page_base = ... ;  // 첫 PageItem 의 vpos
    }
    let mut hc = HeightCursor { ... col_area_y=0.0, ... };  // current_height 상대공간
    let y = hc.vpos_adjust(st.current_height, ...);
    st.current_height = y;
}
```

**Stage E1 동작 변경**: treat_as_char 인라인 표 advance 정합 (host LINE_SEG 기반, fmt.total_height) — 기존 `effective_height` 만 더해 ~16.9px 과소측정 → 표 이후 overflow 해소.

**Stage E2 동작 변경**: atomic top-fit 60px 스필에서 TopAndBottom Shape 제외 — 한컴이 본문 항목처럼 다음 페이지로 넘김 (글상자 pi=142 → 10쪽 정합).

### 3.4 `src/renderer/mod.rs` (+1)

`pub mod height_cursor;` 모듈 등록.

## 4. 결정적 우려 — PR #1031 (Task #1029) 회귀 ⚠️

### 4.1 base 차이로 인한 자연 revert

PR #1032 base = `7ec2e25f` (5/20 14:48 commit, 본 PR 작성 시점). 그 이후 origin/devel 에 **PR #1031 머지** (`a52859de`, 5/20 늦은 시점) — HWP3 외곽선 paper-edge 정합 회귀 정정.

본 PR 의 `src/renderer/layout.rs` diff 가 base 가 PR #1031 이전이라 PR #1031 변경 영역을 자연히 revert 함:

```diff
- use crate::model::page::PageBorderBasis;
- let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
+ let paper_based = (pbf.attr & 0x01) != 0;
```

4 hunk 모두 revert (× 2 sites + footer_inside 제거 + debug log 축소). 정확히 PR #1031 이 복원한 `PR #1011` baseline 을 다시 `PR #987` 시절로 되돌림.

### 4.2 영향 영역

PR #1031 보고서 (`mydocs/pr/archives/pr_1031_report.md`) 명시:
- HWP3 native (attr=0) → body-edge 회귀
- HWP5/HWPX 는 attr bit0=1 로 우연히 가려졌음
- HWP3 sample16 cover paper-edge 정합 — 작업지시자 시각 판정 완료

본 PR 머지 시 PR #1031 이 정정한 회귀가 **재발** — HWP3 외곽선 paper-edge 정합 깨짐.

### 4.3 PR #1032 commit 자체 의도

본 PR 의 commit 메시지 / 본문에는 PageBorderBasis 변경 언급 없음. 본 PR 의 작업 목적 = 세로 측정 정합 (typeset.rs + height_cursor + layout.rs build_single_column). PageBorderBasis 변경은 **명시되지 않은 의도치 않은 부수효과**.

본 PR 의 자체 diff 의도 = Stage A/B/C 추출 영역 (layout.rs 2200줄대 build_single_column) 만. 그러나 base 차이로 PR #1031 영역 (layout.rs 1020줄대 build_page_borders) 도 함께 변경되어 자동 revert.

## 5. 결정적 검증 미흡 영역

### 5.1 검증 fixture 비공개

PR 본문 검증표:
- 노트 "추진일정은" 9쪽→8쪽: AI 184p (비공개 RFP)
- 글상자(pi=142) 10쪽 정합: 동일 fixture
- LAYOUT_OVERFLOW 27→18: 동일 fixture

본 환경 (메인테이너) 에서 AI 184p fixture **접근 불가** → 본 PR 의 핵심 검증 (노트 8쪽, overflow 27→18) 을 본 환경에서 정량 재현 불가.

PR 보고서 `mydocs/report/task_m100_1027_report.md` 도 동일 fixture 의존 — 한컴 PDF 대조의 본질이 비공개 fixture.

### 5.2 공개 fixture 광범위 sweep 미명시

PR 본문에 `svg_snapshot 8/8` 만 명시. 공개 fixture (sample16-hwp5/hwp3, hy-001, exam_kor/math, aift, biz_plan, KTX, mel-001, table-vpos-01, tbox-v-flow-01, form-01/02) 광범위 sweep 미수행 — 본 환경에서 회귀 검증 필요.

### 5.3 다단 (col_count > 1) 비범위 명시

PR 본문 + 보고서 명시: 다단은 #412 per-column page/lazy base 선행 필요로 Stage E3 보류. exam_kor (다단 fixture) 회귀 예상 (보고서: "exam_eng 8→10 회귀, overflow 동일·분산 악화").

본 PR 의 `vpos_snap_current_height` 가드:
```rust
if st.col_count != 1 { return; }
```
다단 가드로 안전 처리. 그러나 단단에서도 측정 변경 = 전 문서 페이지네이션 영향 → 공개 fixture sweep 필수.

### 5.4 골든 부채 (svg_snapshot 267/617/677/issue_598)

PR 보고서 명시: "병합 시 골든=theirs 로 둔 사전 부채" — 본 PR 변경과 무관함을 확인했다고 주장하나 본 환경에서 재검증 필요. 

`issue_598` (각주 마커 nav) 는 Stage C HEAD 에서도 실패 (stash 검증) — 본 PR 와 무관한 사전 부채로 보임.

## 6. 코드 품질 평가

### 6.1 강점

- **순수 함수 추출**: `vpos_corrected_end_y` / `para_has_overlay_shape` 시그니처 명확, 부작용 0, 단위 테스트 가능
- **상태 캡슐화**: `HeightCursor` 가 inter-item 상태 (page/lazy base + prev + partial table) 명시
- **parity 테스트 7개**: 핵심 경로 (page/lazy/clamp/bypass) 손계산 정합 검증
- **점진적 단계**: Stage A→B→C (무동작) → D (페이지네이터 통합) → E1/E2 (표/Shape advance) 순서로 위험 격리
- **다단 가드**: `st.col_count != 1` early return — 다단 회귀 차단
- **debug log 보존**: `RHWP_VPOS_DEBUG` 환경 변수로 진단 가능
- **세부 주석**: 각 코드 영역에 Task 번호 / 회귀 history 명시

### 6.2 우려

- **단일 fixture 의존**: AI 184p (비공개) 가 1차 검증 — 공개 fixture 광범위 sweep 없음
- **base 차이 미인지**: PR 작성 시점 base 와 현재 origin/devel 간 PR #1031 머지 인지 미반영 → PR #1031 회귀 자동 발생
- **commit 메시지 정밀도**: 본 PR commit 메시지에 layout.rs 의 build_page_borders 영역 변경 (PR #1031 revert) 미언급 — 의도 vs 실제 차이

## 7. 검증 계획 (옵션 A 진행 시)

1. **PR #1031 회귀 우선 정정**:
   - cherry-pick 시 layout.rs 의 build_page_borders 4 hunk 를 origin/devel (PR #1031 머지본) 정합 영역으로 유지 (`--theirs` vs `--ours` 선택 필요)
   - 또는 cherry-pick 후 PR #1031 변경 4 hunk 수동 복원
2. **공개 fixture 광범위 sweep**:
   - hy-001 HWPX/HWP5/HWP, sample16-hwp5/hwp3, exam_kor/math, aift, biz_plan, KTX, mel-001, table-vpos-01, tbox-v-flow-01, form-01/02 — 페이지 수 + LAYOUT_OVERFLOW + svg_snapshot
   - 단단 fixture (대부분) 의 단순 정합 + 다단 fixture (exam_kor) 의 다단 가드 발동 확인
3. **HWP3 외곽선 회귀 부재 확인**:
   - sample16-hwp3 의 cover paper-edge 정합 — 작업지시자 시각 판정 (PR #1031 회귀 가드)
4. **issue_852 회귀 가드 통과**: 5/5 (방금 머지된 본 환경 회귀 가드)
5. **CI 패턴** (`feedback_push_full_test_required`):
   - cargo test --release --lib + --tests
   - clippy + fmt --all --check
6. **WASM 빌드**: 4.83~4.84MB 정합
7. **작업지시자 시각 판정**:
   - PR #1031 회귀 부재 (sample16-hwp3 cover)
   - 본 PR 본질 (노트 8쪽) — 비공개 fixture 의존이므로 작업지시자 판정 필수

## 8. 옵션 권고

| 옵션 | 설명 | 위험 | 권고 |
|------|------|------|------|
| **A. cherry-pick + PR #1031 영역 유지** | 본 PR `c670f929` cherry-pick 시 layout.rs build_page_borders 4 hunk 만 origin/devel 정합 영역 유지 (수동 복원 또는 `-X theirs` 선택) | 중간 — Stage A/B/C 추출 검증 + 광범위 sweep 필수 + 다단 미적용 | **권고 (조건부)**: PR #1031 회귀 회피 후 sweep 통과 시 |
| B. supersede 요청 (수정 요청) | 컨트리뷰터에게 PR rebase + base 갱신 (origin/devel) + PR #1031 영역 보존 명시적 처리 요청 | 낮음 — 책임 컨트리뷰터에 위임 | 사이클 마무리 단계 (#1025 후속 OPEN 있으므로 rebase 가능) |
| C. PR scope-out (Stage A/B/C 추출만 머지, D/E 보류) | 무동작 추출만 머지 → 위험 0 + 후속 Stage D/E 별도 PR | 낮음 — 측정 본질 미해결, 노트 8쪽 결함 잔존 | 비권고 — PR 본질 (페이지네이터 정합) 미머지 |

## 9. 메모리 룰 정합

- ✅ `feedback_self_verification_not_hancom` — 본 PR 검증의 본질이 한컴 PDF 대조. 본 환경에서 비공개 fixture 미접근 → 작업지시자 시각 판정 필요
- ✅ `feedback_visual_judgment_authority` — 작업지시자 시각 판정 최종 게이트 (PR #1031 회귀 + 본 PR 본질)
- ✅ `feedback_pr_supersede_chain` — @planet6897 #1003+#1004+#1024+#1032+#1033 시리즈 누적 → 본 PR 가 측정 시리즈 마지막 마디. PR #1031 영역 자연 revert 는 typical supersede chain side-effect
- ✅ `feedback_close_issue_verify_merged` — PR #1031 머지 검증 + 본 PR 머지 시 회귀 부재 검증 필수
- ✅ `feedback_push_full_test_required` — cargo test --tests + clippy + fmt 전체 CI 패턴
- ✅ `feedback_v076_regression_origin` — 컨트리뷰터 자기 환경 fixture (AI 184p 비공개) 정합 ≠ 메인테이너 환경 회귀 부재. 광범위 sweep 필수
- ✅ `feedback_contributor_cycle_check` — @planet6897 22 PR 누적 + 시리즈 위치 명시
- ✅ `feedback_hancom_compat_specific_over_general` — Stage E1 (TAC 표 host LINE_SEG advance) / E2 (TopAndBottom Shape 제외) 모두 case-specific
- ⚠️ `feedback_release_sync_check` — base 차이 인지 + PR #1031 머지 후 작업 시 사전 점검 필요

## 10. 작업지시자 결정 요청

| 결정 | 옵션 |
|------|------|
| 진행 여부 | A (cherry-pick + PR #1031 보존) / B (수정 요청 supersede) / C (보류) |
| PR #1031 회귀 처리 | layout.rs build_page_borders 4 hunk 수동 보존 / 컨트리뷰터에 rebase 요청 |
| sweep 검증 범위 | 공개 fixture 10+ / 광범위 / 비공개 fixture (작업지시자 환경) |
| 시각 판정 | 본 환경 정량 입증 + 작업지시자 시각 판정 / 정량만 |
