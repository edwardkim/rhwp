# PR #401 처리 보고서 — Task #398 표 페이지 분할 시 rowspan>1 셀 분할 단위 산정 누락 정정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#401](https://github.com/edwardkim/rhwp/pull/401) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#398](https://github.com/edwardkim/rhwp/issues/398) (closes #398) |
| 처리 결정 | **cherry-pick 머지** (작성자 v2 정정 후) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 1차 검토 (2026-04-28) — 옵션 A 재정정 요청

본 PR cherry-pick 후 시각 판정에서 **`samples/synam-001.hwp` 5 페이지 회귀** 발견:
- devel: 35 페이지 (정상)
- PR #401 v1: 37 페이지 (+2), pi=69 표 rows=0..5 → rows=0..2 축소

**원인**: `pi=69` 표의 셀[4] (`r=2, c=0, rs=5`) 큰 rowspan 셀이 PR 의 rowspan 블록 단위 분할 정책으로 페이지 5 잔여 공간에 안 들어감 → 블록 전체가 다음 페이지로 미뤄짐.

**처리**: cherry-pick 폐기 + `samples/synam-001.hwp` 를 devel 에 commit (`b1c97fe`) + 작성자에게 재정정 요청 댓글 ([comment-4336798591](https://github.com/edwardkim/rhwp/pull/401#issuecomment-4336798591))

### Stage 1: 작성자 v2 정정

작성자가 메인테이너 피드백 옵션 3 (블록 단위 분할은 작은 블록만) 채택:

| 블록 크기 | 정책 |
|-----------|------|
| 1 행 (rowspan=1) | 인트라-로우 콘텐츠 분할 (기존) |
| **2~3 행 보호 블록** | 블록 단위 분할만 (PR #401 v1 동작) |
| **4 행 이상 큰 rowspan** | 행 경계 분할 + 인트라-로우 분할 (devel 복귀, HanCom 호환) |

신규 commit `e393cc1` — "Task #398 v2: synam-001.hwp 회귀 정정"
- `BLOCK_UNIT_MAX_ROWS = 3` 상수
- `snap_to_block_boundary` 보호 블록만 후퇴
- `cur_block_protected` / `next_block_protected` 도입

### Stage 2: cherry-pick

`local/pr401` 브랜치 (`local/devel` 분기) 에서 5 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `0c7b6dc` (← `4454573`) | @planet6897 | Stage 1: MeasuredTable rowspan 묶음 메타데이터 |
| `81db203` (← `d570ab7`) | @planet6897 | Stage 2: 분할 호출부 rowspan 블록 정책 |
| `2851365` (← `50f4164`) | @planet6897 | Stage 3: 회귀 검증 + 최종 보고서 |
| `c2944a4` (← `ebc3630`) | @planet6897 | 샘플 PDF 갱신 |
| `0d7e776` (← `e393cc1`) | @planet6897 | **v2: synam-001 회귀 정정** |

cherry-pick 결과:
- 첫 4 commit 자동 머지 성공
- `e393cc1` 시 `mydocs/orders/20260428.md` add/add 충돌 (PR #415 의 cherry-pick 시 이미 추가됨) → HEAD (PR #415 의 통합 orders) 그대로 유지 후 `--continue`

### Stage 3: 검증 (회귀 정정 통과)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1044 passed** (1037 → +7 신규 Task #398 테스트) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 23s, 4,127,757 bytes (PR #415 시점 → +10,684 bytes) |

### Stage 4: synam-001 회귀 정정 확인 (핵심)

| 항목 | devel | PR #401 v1 | **PR #401 v2 (정정 후)** |
|------|-------|------------|--------------------------|
| 전체 페이지 수 | 35 | 37 (+2 회귀) | **35** ✅ |
| 페이지 5 의 PartialTable pi=69 | rows=0..5 | rows=0..2 (회귀) | **rows=0..5** ✅ |
| 페이지 5 SVG 크기 | 333 KB | 101 KB (회귀) | 333 KB ✅ |

→ **synam-001 회귀 완전 정정**.

### Stage 5: 작업지시자 시각 판정

| 시나리오 | 결과 |
|---------|------|
| `samples/synam-001.hwp` 5 페이지 (rs=5 큰 rowspan) | ✅ 통과 |
| `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 1, 2 페이지 (rs=2 보호 블록) | (PR #401 원 목표 보존) |

작업지시자 시각 판정: **통과**.

## 변경 요약

### Task #398 핵심 (5 단계)

| Stage | 내용 |
|-------|------|
| 1 | MeasuredTable 에 rowspan 묶음 블록 캐시 (`row_block_start/end`, `compute_row_blocks`) |
| 2 | 분할 호출부에 rowspan 블록 정책 적용 (`split_table_rows`, `paginate_table`) |
| 3 | 회귀 검증 + 최종 보고서 |
| 샘플 갱신 | 본 PR 검증용 hwpx + PDF |
| **v2 정정** | 큰 rowspan 셀 (≥4 행) 행 단위 분할 허용 (HanCom 호환), 작은 블록 (≤3 행) 만 보호 |

### 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/height_measurer.rs` | 블록 캐시 + `compute_row_blocks` + `row_block_for` + `snap_to_block_boundary` + `BLOCK_UNIT_MAX_ROWS` 상수 + 단위 테스트 7건 |
| `src/renderer/pagination/engine.rs::split_table_rows` | pre-loop `first_block_h` (보호 블록만) + snap + `cur_block_protected` / `next_block_protected` 가드 |
| `src/renderer/typeset.rs` | 동일 정책 병행 코드 경로에 적용 |
| `samples/2025년 기부·답례품 실적 지자체 보고서_양식.{hwpx,pdf}` | 신규 (회귀 검증) |

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1044 + svg_snapshot 6/6 + clippy 0 + WASM 빌드 |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 직접 판정 통과 |
| 메인테이너 회귀 검증 샘플 공유 | ✅ samples/synam-001.hwp commit (`b1c97fe`) |
| PR 댓글 톤 | ✅ 차분, 사실 중심 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr401` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr401` → `local/devel` → `devel` 머지 + push
3. PR #401 close + 작성자 댓글 (이슈 #398 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_401_review.md` (1차 검토)
- PR: [#401](https://github.com/edwardkim/rhwp/pull/401)
- 이슈: [#398](https://github.com/edwardkim/rhwp/issues/398)
- 1차 검토 댓글 (재정정 요청): [comment-4336798591](https://github.com/edwardkim/rhwp/pull/401#issuecomment-4336798591)
- 작성자 v2 정정 보고: [comment 2026-04-28T17:00:06Z](https://github.com/edwardkim/rhwp/pull/401)
- 회귀 검증 샘플: `samples/synam-001.hwp` (devel commit `b1c97fe`)
