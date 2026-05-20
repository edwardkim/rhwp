# PR #1033 검토 — Task #1025: 페이지보다 큰 단일 표 셀 내부 분할 (intra-cell line split)

- PR: [#1033](https://github.com/edwardkim/rhwp/pull/1033)
- 작성자: @planet6897 (Jaeook Ryu, 23번째 PR — 분할 표·측정 시리즈 5번째 마지막 마디)
- closes #1025 (M100, 메인테이너 작성 이슈, milestone 미설정, assignee 미지정)
- base: devel (PR base 시점 `a52859de` = PR #1031 머지 직후, 현재 origin/devel = `5263f53d` = PR #1032 머지 후)
- head: pr-task1025 (단일 squash `7f43226c`)
- mergeable: MERGEABLE, CI 전체 통과 (Build & Test / CodeQL / Canvas visual diff)
- 변경 규모: +846/-35, 15 파일 (8 코드 + 7 문서)
- 일시: 2026-05-20

## 1. 컨트리뷰터 사이클 + 시리즈 위치 (`feedback_contributor_cycle_check`)

@planet6897 23 PR 누적. 분할 표·측정 시리즈 마지막 마디:

- PR #1003 (closes #990) — 빈 문단 위 TAC 글상자 advance 정정 (머지)
- PR #1004 (Refs #991) — 분할 표 렌더링 부분 적용 (머지)
- PR #1024 (closes #1022) — 측정 정합: RowCut 모델 + LAYOUT_OVERFLOW 42→12 (머지)
- PR #1032 (closes #1027) — HeightCursor 공유 측정 엔진: 노트 8쪽 (방금 머지, `12cc1fb3`)
- **PR #1033 (closes #1025)** — page-larger 단일 셀 내부 분할 (본 PR)

PR 본문 검증표 명시: AI 184p (비공개 RFP fixture) 한컴 2022 PDF 정합. 본 환경 fixture 미접근.

## 2. 이슈 #1025 배경

페이지보다 큰 **단일 표 셀**이 분할될 때 본문 영역을 넘어 렌더링되거나 용지 밖으로 잘리는 현상.

- Task #993 (분할 표 cut 모델) / #1022 (측정 정합) 가 행 단위 cut 으로 대부분 해소했으나, 셀 콘텐츠 자체가 페이지 본문 높이를 초과하는 경우는 명시적으로 scope-out (`task993 §4`)
- 재현: 요구사항 명세 표 PMR-007 의 세부내용 셀 (25문단 ≈ 1024px) — rs=2 라벨 셀 (상세설명) 이 걸친 행 + 거대 셀 = 본문 143px 초과 + 용지 밖 잘림

## 3. 본질 — 행블록(row-block) cut

기존 `advance_row_cut` (Task #993) 의 일반화 — rowspan(rs>1) 셀로 묶인 연속 행 블록 `[b_start, b_end)` 를 셀 (row, col) 안정 순서로 순회.

### 3.1 신규 함수 `advance_row_block_cut` (`src/renderer/layout/table_layout.rs:3712`)

```rust
pub(crate) fn advance_row_block_cut(
    &self,
    table: &Table,
    b_start: usize,
    b_end: usize,
    start_cut: &[usize],
    avail_height: f64,
    styles: &ResolvedStyleSet,
) -> RowCutResult
```

핵심 알고리즘:
1. `row_block_cells` 로 블록과 교차하는 셀 (rs>1 라벨 셀 + 블록 내 rs==1 셀) 수집
2. `(row, col)` 안정 정렬
3. 각 셀의 `cell_units` (줄/중첩 atom) 를 `avail_height` 까지 채움
4. rs>1 라벨 셀은 첫 조각 (start_cut 비었을 때) 전량 소비 → 연속 조각에선 0 유닛 (공란)
5. 거대 `row_span==1` 셀은 줄 단위로 페이지 경계까지 채우고 잔여를 다음 조각으로

**parity 보장**: 단일 비-rowspan 행 (`b_end==b_start+1`, rs>1 셀 없음) 에서는 `advance_row_cut` 과 동등 — 회귀 0. 단위 테스트 `test_block_cut_single_row_parity` 4 케이스 (avail 50/96/500/5px) + `test_block_cut_rowspan_giant_split` (라벨 rs=2 + 거대 셀 10줄 → 첫 조각 [2,2,5] / 잔여 [0,0,5]).

### 3.2 `row_block_content_height` 신규 헬퍼

블록 셀별 `content_in_cut + pad` 의 max 산출. `advance_row_block_cut` 과 동일한 `(row, col)` 셀 순서를 사용 — 페이지네이터 / 렌더러 단일 권위.

### 3.3 `PageItem::PartialTable.is_block_split` 필드 (pagination.rs)

```rust
PartialTable {
    para_index, control_index, start_row, end_row, is_continuation,
    start_cut: Vec<usize>,
    end_cut: Vec<usize>,
    /// [Task #1025] true 이면 컷이 rowspan 블록-셀 (row,col) 인덱스
    /// (advance_row_block_cut). false 이면 단일 행 row_span==1 col 인덱스
    /// (advance_row_cut, 기존). page-larger 셀 내부 분할에서만 true.
    is_block_split: bool,
}
```

**Backward-compatible 확장**: 기존 분할 표 (form-002 등) 는 `is_block_split=false` 로 per-row 유지. page-larger 셀 분할만 새 모드 사용.

### 3.4 렌더러 `src/renderer/layout/table_partial.rs` (+170/-29)

`is_block_split=true` 일 때 rowspan 행 포함 + `rowspan_block_range` / `block_cut_index` 로 블록-셀 인덱싱 (높이·줄범위). rs>1 라벨 셀 연속 페이지 공란.

### 3.5 페이지네이터 `src/renderer/typeset.rs` (+83/-6) — mid-page 분할 게이트

mid-page 분할은 **진짜 page-larger** (`block_h > base_available`) 일 때만 활성. 그 외엔 deferred (다음 페이지로). 연속분 행-스킵 가드를 블록 컷에서 스킵 (컷 보존).

## 4. PR #1032 영역 점검 (검토 단계 핵심 확인)

PR base = `a52859de` (PR #1031 머지 직후, PR #1032 머지 이전). 본 환경 origin/devel = `5263f53d` (PR #1032 머지 후).

PR #1033 의 diff 가 base 시점 (PR #1032 이전) 기준이라 typeset.rs 의 PR #1032 변경 (Task #1027 Stage D: vpos_snap_current_height, HeightCursor, reset_vpos_cursor) 을 revert 하는 듯 보임. PR #1032 처리 시 우려한 PR #1031 회귀 패턴과 동일 구조.

**cherry-pick dry-run 검증 결과**: `git cherry-pick --no-commit 7f43226c`
- `Auto-merging src/renderer/layout.rs`
- `Auto-merging src/renderer/typeset.rs`
- 충돌 없음, 빌드 성공
- **PR #1032 변경 13건 모두 보존** (vpos_snap_current_height, vpos_page_base, reset_vpos_cursor 등)
- PR #1033 변경 (advance_row_block_cut, is_block_split) 추가
- cargo test --lib **1319 passed** (1317 + 2 = block_cut parity + rowspan_giant_split)

cherry-pick auto-merge 가 두 PR 의 같은 파일 다른 영역 변경을 양립 통합 — PR #1032 cherry-pick 시 PR #1031 영역 자동 보존된 것과 동일 패턴.

## 5. 코드 품질 평가

### 5.1 강점

- **일반화 전략 명확**: 단일 비-rowspan 행 = `advance_row_cut` 동등 (parity 단위 테스트 4 케이스 입증) — 기존 분할 표 회귀 0
- **새 모드 격리**: `is_block_split` 플래그로 page-larger 분할만 새 알고리즘 사용, 기존 form-002 등 일반 분할 무변경
- **mid-page 분할 게이트**: `block_h > base_available` 조건 — 진짜 page-larger 만 활성, 정상 분할은 deferred 유지
- **셀 순서 단일 권위**: `(row, col)` 정렬을 `advance_row_block_cut` / `row_block_content_height` / 렌더러 모두 공유 → 측정 정합
- **rs>1 라벨 셀 처리 명확**: 첫 조각 전량 소비 → 연속 조각 0 유닛 = 한컴 정답지 (PMR-007 라벨 셀 다음 페이지 공란)
- **rust 단위 테스트**: 회귀 가드 영구화 (parity + 분할 케이스)
- **점진적 통합**: PR #1032 의 측정 정합 위에 page-larger 처리만 추가 — 격리된 책임 추가

### 5.2 우려

- **단일 fixture 의존**: AI 184p (비공개) 가 1차 검증 — 공개 fixture 광범위 sweep 미명시
- **PR base 시점 차이**: PR #1032 이전 base 였으나 cherry-pick auto-merge 로 자동 해소 (PR #1032 와 동일 패턴)
- **rowspan 중첩표 보유 셀**: PR 본문 "비범위 — atom 원자 유지" 명시. 일반화는 후속

## 6. 검증 계획 (옵션 A 진행 시)

1. **cherry-pick 자연 통합 확인** (dry-run 완료):
   - PR #1032 영역 보존 ✅ (vpos_snap_current_height 등 13건)
   - PR #1031 영역 보존 ✅ (paper_based / footer_inside)
   - PR #1033 변경 (advance_row_block_cut, is_block_split) 추가 ✅

2. **CI 패턴 검증**:
   - cargo test --release --lib (1319 expected, +2 block_cut tests)
   - cargo test --release --tests (issue_852 5/5 포함)
   - clippy + fmt --all --check

3. **공개 fixture 광범위 sweep**:
   - hwp3-sample16-hwp5/hwp3, hy-001, exam_kor/math, aift, biz_plan, KTX, mel-001, table-vpos-01, tbox-v-flow-01, form-01/02
   - 페이지 수 + LAYOUT_OVERFLOW + svg_snapshot
   - **분할 표 보유 fixture** (table-vpos-01, form-01/02) 가 새 `is_block_split=false` 경로로 무변경 입증

4. **PR #1031 + #1032 회귀 부재 확인**:
   - HWP3 외곽선 paper-edge 정합 (sample16-hwp3 cover, PR #1031 가드)
   - Task #1027 노트 정합 (sample16-hwp5 측정, PR #1032 가드)

5. **issue_852 회귀 가드 5/5 통과**

6. **WASM 빌드**: 4.86~4.88MB 정합

7. **작업지시자 시각 판정**:
   - PR 본질 (PMR-007 page-larger 셀 분할) — AI 184p fixture 미접근이므로 공개 fixture sweep 결과 + 시각 비교
   - PR #1031 / #1032 회귀 부재

## 7. 옵션 권고

| 옵션 | 설명 | 위험 | 권고 |
|------|------|------|------|
| **A. cherry-pick + sweep + 시각 판정** | dry-run 으로 cherry-pick 자연 통합 확인 → 공개 fixture sweep + WASM + 작업지시자 시각 판정 → 머지 | 낮음 — auto-merge 검증 완료, parity 단위 테스트 보장, mid-page 분할 게이트 격리 | **권고** |
| B. supersede 요청 (rebase 후 재제출) | 컨트리뷰터에게 PR base 갱신 (origin/devel `5263f53d`) 요청 후 재제출 | 매우 낮음 — 명시적 base 정합 | 시리즈 마무리 단계 (4개 머지 사이클 통과) — 불필요 절차 |
| C. PR scope-out (단위 테스트만 머지) | parity 테스트만 분리 머지, 본질 보류 | 낮음 — page-larger 결함 잔존 | 비권고 — PR 본질 미머지 |

## 8. 메모리 룰 정합

- ✅ `feedback_self_verification_not_hancom` — 본 PR 검증의 본질이 한컴 PDF 대조. 비공개 fixture 미접근 → 작업지시자 시각 판정 필요
- ✅ `feedback_visual_judgment_authority` — 작업지시자 시각 판정 최종 게이트
- ✅ `feedback_pr_supersede_chain` — @planet6897 #1003+#1004+#1024+#1032+#1033 분할 표·측정 시리즈 마지막 마디. base 차이 cherry-pick auto-merge 양립 패턴 일관
- ✅ `feedback_push_full_test_required` — cargo test --tests + clippy + fmt 전체 CI 패턴
- ✅ `feedback_contributor_cycle_check` — @planet6897 23 PR 누적 + 시리즈 위치 명시
- ✅ `feedback_hancom_compat_specific_over_general` — page-larger 셀 분할만 새 모드 (`is_block_split=true`), 일반 분할 (form-002) 은 기존 `advance_row_cut` 유지 — case-specific 가드
- ✅ `feedback_close_issue_verify_merged` — PR #1031/#1032 머지 검증 + 본 PR 머지 시 회귀 부재 검증 필수

## 9. 작업지시자 결정 요청

| 결정 | 옵션 |
|------|------|
| 진행 여부 | A (cherry-pick + sweep + 시각 판정) / B (supersede) / C (보류) |
| sweep 검증 범위 | 공개 fixture 10+ / 광범위 / 비공개 fixture (작업지시자 환경) |
| 시각 판정 | 본 환경 정량 입증 + 작업지시자 시각 판정 / 정량만 |
