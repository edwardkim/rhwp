# PR #401 검토 — 표 페이지 분할 시 rowspan>1 셀 분할 단위 산정 누락 (#398)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#401](https://github.com/edwardkim/rhwp/pull/401) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터 (다수 머지 이력) |
| base / head | `devel` ← `planet6897:local/task398` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | BEHIND |
| 변경 통계 | +786 / -22, 11 files |
| CI | 모두 SUCCESS |
| 이슈 | [#398](https://github.com/edwardkim/rhwp/issues/398) |
| 정황 | CLAUDE.md 절차 준수 (수행/구현 계획서 + Stage 보고서 + 최종 보고서 모두 포함) |

## 작성자 정황

@planet6897 — 머지 이력: PR #371, #373, #392 등. 신뢰 컨트리뷰터. CLAUDE.md 의 하이퍼-워터폴 절차 준수.

## 결함 (이슈 #398)

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 1쪽 하단 표 (pi=22, 3×3) 의 첫 행 잘림 + rowspan=2 제목 셀 텍스트가 본문 영역 끝에 겹쳐 그려짐.

### 근본 원인

분할 단위 산정에서 **rowspan>1 셀의 시각적 점유 영역 누락**:

- `mt.row_heights[r]` 산출 시 `cell.row_span == 1` 셀만 집계 (`height_measurer.rs:466`)
- `split_table_rows::first_row_h = mt.row_heights[0]` = 17.27px (rs=1 셀만)
- 실제 첫 분할 단위는 행 0+1 묶음 (rowspan=2 셀 점유) = 35.4px
- 페이지 1 잔여 19px ≥ 17.27px → 행 0 배치 결정 → 35.4px 셀이 시각적으로 잘림

## 변경 내용 (11 files / +786 / -22)

### 1. `src/renderer/height_measurer.rs`

`MeasuredTable` 에 rowspan 묶음 블록 캐시 추가:
- `row_block_start[r]` / `row_block_end[r]` — r 행을 포함하는 모든 rowspan 셀의 최소 시작 / 최대 종료 행
- `compute_row_blocks(table, row_count)` 헬퍼 — 모델의 `table.cells` 직접 사용 (모든 page_break 모드 일관)

신규 메서드:
- `MeasuredTable::row_block_for(row) -> (start, end_exclusive, height)`
- `MeasuredTable::snap_to_block_boundary(end_row) -> usize`

### 2. `src/renderer/pagination/engine.rs::split_table_rows`

- `first_row_h = mt.row_heights[0]` → `first_block_h = mt.row_block_for(0).2`
- `find_break_row` 결과에 `snap_to_block_boundary` 적용
- 단일 행 블록일 때만 인트라-로우 분할 시도
- 다중 행 블록이 안 들어가면 `end_row = cur_b_end` (블록 전체 한 단위)

### 3. `src/renderer/typeset.rs::paginate_table`

같은 패턴 적용 (분할 로직 중복 — 향후 통합 고려, 별도 task)

### 4. 단위 테스트 7건 (`height_measurer.rs::tests`)

- `test_compute_row_blocks_all_single` — rowspan 모두 1
- `test_compute_row_blocks_rs2_at_row0` — 행 0에 rs=2 셀
- `test_compute_row_blocks_overlapping` — 겹치는 rowspan 통합
- `test_compute_row_blocks_disjoint` — 비인접 rowspan 별개 블록
- `test_row_block_for_basic` — 같은 블록 내 모든 행 동일 (start, end, height)
- `test_row_block_for_empty_metadata` — 빈 vec → 단일 행 폴백
- `test_snap_to_block_boundary` — 블록 중간 → 시작으로 후퇴

### 5. 회귀 검증 자료

| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| 페이지 1 마지막 항목 | `PartialTable pi=22 rows=0..1 cont=false 3x3` | `pi=21 (빈)` (표 사라짐) |
| 페이지 2 첫 항목 | `PartialTable pi=22 rows=1..3 cont=true` | `Table pi=22 ci=0 3x3 635.0x924.5px` (표 전체) |
| 페이지 1 SVG 제목 글자 (분/석/답/례) | 본문 영역 끝에 겹쳐 그려짐 | 페이지 1 부재, 페이지 2 정상 |

### 6. 문서 + 샘플

- `mydocs/plans/task_m100_398{,_impl}.md`
- `mydocs/working/task_m100_398_stage{1,2}.md`
- `mydocs/report/task_m100_398_report.md`
- `mydocs/orders/20260428.md`
- `samples/2025년 기부·답례품 실적 지자체 보고서_양식.{hwpx,pdf}` (신규)

## 검증

### CI

모든 CI SUCCESS (Build & Test, CodeQL javascript / python / rust).

### 본 검토에서 cherry-pick 후 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1038 passed** (1031 → +7 신규) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 21s, 4,112,090 bytes |
| 작업지시자 시각 검토 | ✅ 통과 (`samples/...hwpx,pdf` 적합 판정) |

## 평가

### 강점

1. **결함 분석 명확** — `row_heights` 의 `row_span == 1` 필터가 분할 단위 산정에서 누락하는 정황 정확히 짚음
2. **신뢰 컨트리뷰터** — @planet6897, 다수 머지 이력
3. **CLAUDE.md 절차 준수** — 수행/구현 계획서 + Stage 보고서 + 최종 보고서 모두 포함
4. **데이터 구조 캐시** — `row_block_start/end` 사전 계산으로 호출마다 재계산 회피
5. **모든 page_break 모드 일관** — `mt.cells` 가 아닌 `table.cells` 사용
6. **단위 테스트 7건** — rowspan 묶음 시나리오 광범위
7. **회귀 검증 양호** — 본 샘플 + 다른 표 샘플 (table-vpos-01, 표-텍스트) 무회귀 명시
8. **CI 통과** + 본 검토 cherry-pick 후 1038 passed
9. **샘플 + PDF 첨부** — 작성자가 직접 만들어 시각 검증 자료 제공

### 약점 / 점검 필요

#### 1. PDF 정답지 정황

이슈 본문: "PDF 원본은 표 전체를 2쪽으로 미룸". 메모리 (`feedback_pdf_not_authoritative.md`) 의 "PDF 환경 의존성" 점검 필요. 그러나:

- **작업지시자가 직접 시각 검토 통과** 판정 — 정합성 확인됨
- 작성자의 PDF 가 작업지시자 환경 PDF 와 일치 (시각 검토 결과)

→ 본 PR 의 정정 방향 정당.

#### 2. typeset.rs / engine.rs 분할 로직 중복

작성자가 본문에 명시:
> 동일 패턴을 `pagination/engine.rs` 와 `typeset.rs` 양쪽에 적용 (분할 로직 중복 — 향후 통합 고려, 별도 타스크).

→ **별도 task 후보** — 향후 typeset / pagination 통합 리팩토링.

## 메인테이너 작업과의 관계

### 충돌 가능성

본 PR base 가 PR #395, #396 머지 전 (`4828937`) 이라 BEHIND 였지만 작성자가 자체 devel merge 진행 (commit `0cf38d4`). cherry-pick 시 자동 머지 성공.

검증 — Task #418 회귀 테스트 통과 ✅ (페이지 분할 로직 다른 영역).

## 처리 결과 — 옵션 A (작성자 재정정 요청)

### 회귀 발견 — `samples/synam-001.hwp` 5 페이지

작업지시자 환경에서 PR #401 cherry-pick 후 시각 비교 결과 **다른 표 샘플에서 회귀**:

| 항목 | devel (PR #401 미적용) | PR #401 적용 후 |
|------|----------------------|----------------|
| 전체 페이지 수 | 35 | **37** (+2 증가) |
| 페이지 5 의 PartialTable pi=69 | rows=0..5 (5 행 배치) | rows=0..2 (2 행만) |
| 페이지 5 SVG 크기 | 333,125 bytes (정상) | 101,136 bytes (대폭 축소) |

### 회귀 원인 (분석)

`pi=69` 표의 셀[4]: `r=2, c=0, rs=5, cs=1` — 행 2~6 을 점유하는 큰 rowspan 셀.

PR #401 의 `compute_row_blocks` 가 이 셀을 보고 **행 2~6 을 한 블록** 으로 묶음. 결과:

- devel: 행 0,1 (제목 + 소득) + 일부 행 분할 → 5행까지 페이지 5 에 들어감
- PR #401: 행 2~6 한 블록 = 너무 큰 블록 → 페이지 5 잔여 공간에 안 들어감 → 행 0,1 까지만 → 페이지 6, 7 으로 미룸

본 PR 의 정책 (rowspan 묶음 단위 분할) 이 **너무 보수적**:
- 큰 rowspan 셀 (rs=5) 에 대한 별도 가드 없음
- 한컴은 큰 rowspan 셀이 페이지에 안 들어가면 셀 내부 분할을 허용하는데, 본 PR 은 블록 전체를 다음 페이지로 미룸

### 회귀 정황

- 메인테이너 환경의 `samples/synam-001.hwp` (메인테이너만 보유, git 미추가) 에서 회귀 발견
- 작성자는 본 샘플을 가지지 못해 회귀 검증 못함 — 외부 컨트리뷰터의 환경 한계

### cherry-pick 폐기

- `local/pr401` 브랜치 삭제
- `pr401-test` 임시 ref 삭제
- WASM 도 PR #401 적용 전 상태 (4,106,811 bytes) 로 재빌드
- devel 무회귀 상태 유지

## 다음 단계 — 옵션 A

1. **`samples/synam-001.hwp` 를 메인테이너 commit 으로 추가** — 작성자가 회귀 검증할 수 있게
2. **작성자 댓글** — 회귀 정황 + synam-001 샘플 안내 + 재정정 요청
3. PR #401 OPEN 유지 (close 안 함)
4. 작성자 재정정 후 다시 검토

## 참고

- PR: [#401](https://github.com/edwardkim/rhwp/pull/401)
- 이슈: [#398](https://github.com/edwardkim/rhwp/issues/398)
- 작성자 머지 이력: PR #371 (Task #370), #373 (Task #372), #392 (Task #391)
