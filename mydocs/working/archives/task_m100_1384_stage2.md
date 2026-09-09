# Task M100 #1384 — 2단계 완료 보고서 (등록 축 정정 + xfail 승격)

- 브랜치: `local/task1384`
- 작성일: 2026-06-14
- 수정 파일: `src/serializer/hwpx/context.rs`, `tests/hwpx_roundtrip_baseline.rs`

## 1. 구현 내용

### 2.1 등록 축 정정 (1줄)

`context.rs:117`: `register(idx as u16)` → `register((idx + 1) as u16)`.
doc 주석에 사유 명기 — borderFill 방출 id(`idx+1`)·borderFillIDRef(1-based 참조)·
인라인 등록(IR 값 1-based)과 통일.

### 2.2 baseline xfail 승격

`tests/hwpx_roundtrip_baseline.rs` `XFAIL`을 **빈 배열**로 — 4건(exam_kor/exam_social/
exam_social-p1/issue_1133) 제거. doc 주석에 #1384 승격 사유 기록.
`ORACLE_UNFIT`(exam_kor 등)은 유지 — 시각 oracle 부적합은 별개.

### 2.3 단위 테스트

- `task1384_border_fill_registered_one_based` (context): borderFill 31개 적재 →
  borderFillIDRef=31(마지막) resolved + id 0 미등록(1-based 축 가드) + id 31 등록 확인.

`cargo test --lib serializer::hwpx::context` 6 passed / fmt 통과.

## 2. 전수 검증

### 2.1 baseline 게이트

`cargo test --test hwpx_roundtrip_baseline` — **4 passed** (4샘플이 A등급 전수
대상으로 합류, `xfail_entries_still_fail`은 빈 XFAIL로 통과).

### 2.2 전수 배치 (`output/poc/task1384/`)

`hwpx-roundtrip --batch samples/hwpx`:

| 항목 | 종전 | 수정 후 |
|------|------|--------|
| **SERIALIZE_FAIL** | 4 (#1384 4샘플) | **0** |
| **PASS** | 49 | **53** |
| IR_DIFF / ROUND2_DIFF | 0 | 0 (회귀 없음) |
| PARSE_FAIL | 1 (제외 hwpx-01) | 1 (동일) |

4샘플(exam_kor/exam_social/exam_social-p1/issue_1133) 전부 PASS + k-water-rfp 등
기존 통과 샘플 회귀 0.

## 3. 다음 단계

3단계 — 매뉴얼(등급 현황 A=53/B=0) + CI급(release-test) + 최종 보고서
(numbering 잠재 결함 기록 + #1381/#1384 동시 close 안내).

승인 요청드립니다.
