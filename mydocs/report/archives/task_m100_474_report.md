# Task #474 최종 결과보고서 — RowBreak 표가 rowspan 보호 블록 정책으로 일찍 분할

## 결과 요약

`samples/k-water-rfp.hwp` 5페이지 표가 잔여 공간 있음에도 일찍 분할되는 회귀 정정. **표 정책 (RowBreak) 과 rowspan 보호 블록 정책 (Task #398) 의 모순** 을 표 정책 우선으로 정리.

## 정정 본질

| 정합 요소 | 의미 |
|----------|------|
| HWP `TablePageBreak::RowBreak` (bit 0-1 = 0x02) | **행 경계에서 분할** — 행 단위 분할 허용 |
| Task #398 의 `BLOCK_UNIT_MAX_ROWS=3` 보호 블록 정책 | rowspan>1 셀 묶음 보호 → 보호 블록 단위 분할 차단 |
| 모순 케이스 (k-water-rfp pi=52) | RowBreak 표 + rs=2 보호 블록 → 표 정책 의도와 충돌 |

**정리 방향**: RowBreak 표는 보호 블록 정책 비적용 (HWP 의 명시 표 정책 우선).

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/height_measurer.rs` | `allows_row_break_split()` 메서드 추가 + `snap_to_block_boundary` RowBreak 조기반환 + 단위 테스트 2 |
| `src/renderer/typeset.rs` | `paginate_table` 의 `first_block_protected` / `cur_block_protected` / `next_block_protected` 가드 (3개소) |
| `src/renderer/pagination/engine.rs` | `split_table_rows` 의 동일 3개소 가드 |

## 정정 영향 영역

| 표 정책 | block_size | 정정 후 동작 |
|---------|-----------|------------|
| RowBreak | 2~3 | **보호 비적용** (본 정정 — 행 단위 분할 허용) |
| RowBreak | >3 | 영향 없음 (이미 분할 허용) |
| RowBreak | 1 | 영향 없음 (단일 행) |
| None / CellBreak | 2~3 | **보호 유지** (PR #401 v2 정정 그대로) |
| None / CellBreak | 그 외 | 영향 없음 |

## Stage 별 결과

| Stage | 내용 | 결과 |
|-------|------|------|
| 1 | 베이스라인 측정 | k-water-rfp 페이지 5: rows=0..2 (회귀 정황 확정), synam-001 정정 보존 정황 분석 |
| 2 | 정정 적용 | 4파일 변경, 단위 테스트 2 추가, dump-pages 회귀 해소 확인 |
| 3 | 광범위 회귀 검증 | 10 샘플 / 232 페이지 — 정정 영향 영역은 k-water-rfp 페이지 5/6 만 |
| 4 | 작업지시자 시각 검증 | 승인 |
| 5 | 최종 결과보고서 + 오늘할일 갱신 | 본 문서 |

## 회귀 정정 확인 (dump-pages)

### k-water-rfp.hwp

| 페이지 | 정정 전 | 정정 후 |
|--------|---------|---------|
| 5 | `PartialTable rows=0..2`, used=196.3px (21%) | `PartialTable rows=0..4`, 4행 모두 배치 |
| 6 | `PartialTable rows=2..4` + 잔여 컨텐츠 | `PartialTable rows=3..4` + 다음 컨텐츠 정상 배치 |

### synam-001.hwp (PR #401 v2 정정 보존)

| 페이지 | 결과 |
|--------|------|
| 5 | `PartialTable rows=0..5` (변화 없음) |
| 6 | `PartialTable rows=4..8` (변화 없음) |

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1077 passed** (1075 기존 + 단위 테스트 2 추가) |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ (Task #418 정정 보존) |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |
| 작업지시자 시각 검증 | 승인 |

## 회귀 origin

PR #401 v2 (Task #398 Stage 2) commit `81db203` — `MeasuredTable::snap_to_block_boundary` + 호출부 보호 블록 가드 도입. 일반화된 알고리즘이 RowBreak 표에서 의도와 모순 (메모리 `feedback_v076_regression_origin` 정합).

## 메모리 원칙 정합

- **`feedback_hancom_compat_specific_over_general`**: 일반화 (모든 rowspan 보호) 보다 케이스별 명시 가드 (RowBreak 표 비적용)
- **`feedback_v076_regression_origin`**: 외부 PR (Task #398) 의 일반화 알고리즘이 다른 케이스 회귀 → 명시 가드로 정정
- **`feedback_small_batch_release_strategy`**: 작은 단위 정정 (1 가드 추가, 4 파일 변경)

## 다음 단계

- 이슈 #474 close (작업지시자 승인 후)
- 오늘할일 갱신
