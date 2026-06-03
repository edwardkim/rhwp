# Task #604 Stage 1 — IR 표준 + LineSeg helper

## 본 단계 목표

Document IR 의 LineSeg 표준 명문화 + 포맷 무관 wrap zone 판정 helper 도입. 호출처 변경
없이 (Stage 2 에서 일괄 적용) 표준 인프라 구축.

## 변경 영역

### A. `mydocs/tech/document_ir_lineseg_standard.md` 신규 (+150 LOC)

LineSeg 필드별 단위/원점/0 의미 명시. HWP5/HWPX/HWP3 각 파서의 인코딩 책임 명시.
HWP3 의 `vertical_pos=0` 잔존 부채 + `Paragraph.wrap_precomputed` 청산 대상 명시.

### B. `src/model/paragraph.rs` 갱신 (-10 / +25)

- `LineSeg` struct 의 모든 필드 doc 주석 정합화 (단위 HWPUNIT 명시, 원점 명시, 0 의미 명시)
- `LineSeg::is_in_wrap_zone(col_w_hu) -> bool` helper 추가
  - 본질: `column_start > 0 OR (segment_width > 0 AND segment_width < col_w_hu)`
  - 포맷 무관, 상태 무관, per-line 판정

### C. 분석 자료 이동 (3 파일, mydocs/tech/ 로)

- `document_ir_parser_relationship_analysis.md` (16KB)
- `document_ir_wrap_zone_standard_review.md`
- `hwp5_wrap_precomputed_analysis.md`

git 미추적 `/tmp/` 에서 git 추적 영역으로 이동 — 다른 컨트리뷰터/세션 참조 가능 + 본 task
의 본질적 자료가 코드 변경과 함께 보존.

### D. 분석 자료 내 경로 갱신

기존 `/tmp/` 참조를 `mydocs/tech/` 경로로 일괄 갱신 (sed). 5 파일 정정 (분석 자료 3 + 수행/구현
계획서 2).

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` | ✅ 통과 (rhwp v0.7.9) |
| `cargo test --lib` | ✅ **1130 passed** / 0 failed / 2 ignored |
| `cargo clippy --lib -- -D warnings` | ✅ 0건 |
| 호출처 변경 | 0 (Stage 2 에서 일괄 적용 예정) |

## LOC 합계

| 영역 | 추가 | 제거 |
|------|-----|-----|
| `mydocs/tech/document_ir_lineseg_standard.md` 신규 | +151 | 0 |
| `mydocs/tech/document_ir_*` + `hwp5_wrap_*` 이동 | +0 (cp) | 0 |
| `src/model/paragraph.rs` doc + helper | +27 | -10 |
| **소스 합계** | **+27** | **-10** |
| **문서 합계** | **+151** | **0** |

## 다음 단계 (Stage 2) 영역 미리보기

- `src/renderer/typeset.rs:496` — `wrap_precomputed` → `is_in_wrap_zone` 교체
- `src/renderer/layout/paragraph_layout.rs` 3곳 (862, 883, 1208) — 동일 교체
- `src/renderer/layout.rs:2957, 3345` — 주석 갱신 (Task #604 인용)
- `src/model/paragraph.rs` — `wrap_precomputed: bool` 필드 제거
- `src/parser/hwp3/mod.rs:1556~` — `wrap_precomputed=true` 후처리 30 LOC 제거

## 위험 영역 (Stage 1 시점)

| 위험 | 평가 | 완화 |
|------|------|------|
| helper 시그니처 부적합 | 낮음 | `col_w_hu` 만 인자 (호출 컨텍스트에서 자연스러움) |
| 추가 helper 의 회귀 | 0 | 호출처 없음 — Stage 2 에서 일괄 적용 |
| 분석 자료 이동 시 참조 깨짐 | 0 | 5 파일 sed 일괄 정정 + grep 검증 통과 |

## 작업지시자 승인 요청

본 Stage 1 완료 보고. 다음 단계 (Stage 2: 렌더러 표준 적용 + `wrap_precomputed` 필드 제거)
진입 승인 요청.

## 참조

- 수행계획서: `mydocs/plans/task_m100_604.md`
- 구현계획서: `mydocs/plans/task_m100_604_impl.md`
- LineSeg 표준: `mydocs/tech/document_ir_lineseg_standard.md`
- Issue: #604
