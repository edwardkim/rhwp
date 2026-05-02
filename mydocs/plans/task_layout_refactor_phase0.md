# Layout 리팩터링 Phase 0 수행계획서 — 분석 + 로드맵

**브랜치**: `local/task_layout_refactor`
**작성일**: 2026-05-02
**Phase**: 0 (분석 + 로드맵 문서화)

## 1. 동기

#467 / #491 / #496 보류 사유 "layout 리팩터링 시 종합 해결" 의 구체화. 작업지시자 지시 "layout 리팩터링 진행".

## 2. 분석 결과 요약

상세: `mydocs/tech/layout_refactor_roadmap.md`

### 2-1. 모듈 규모

`src/renderer/` 의 layout 관련 5개 파일 = **12,509 LOC**. 단일 세션 전면 리팩터링 비현실적.

### 2-2. 보류 결함 분류 — 3가지 본질 분리

| 결함 | 위치 | 본질 |
|------|------|------|
| #467 | `document_core/queries/rendering.rs` | master 매핑 (layout 모듈과 별개) |
| #491 | (인프라 부재) | PDF 좌표 정규화 (검증 도구 영역) |
| #496 | `paragraph_layout.rs:81` `layout_inline_table_paragraph` | 다중행 인라인 표 + 다중 줄 본문 처리 한계 |

### 2-3. #496 결함 본질 — 단일 본질이 아닌 복합 (A/B/C)

`samples/exam_science.hwp` p2 pi=61 재현 분석:
- HWP IR: ls[0]=표(2행), ls[1]=본문 첫줄, ls[2]=본문 둘째줄
- 현재 SVG: 본문 baseline 1195.85 가 표 row 0 (1191.68) 과 row 1 (1210.77) 사이에 끼임

복합 본질:
- (A) 본문 baseline 수직 정렬 정책 부재 (다중행 표 시 row 0 위치 고정)
- (B) ls[2..] break 미사용 (`!wrapped_below_table` 제약으로 dynamic reflow)
- (C) 인라인 vs 블록 정책 부재 (분수형 vs 데이터 다중행 표 구분 룰 부재)

## 3. Phase 정의

| Phase | 내용 | 리스크 | 세션 |
|-------|------|--------|------|
| 0 | 분석 + 로드맵 문서화 | 없음 | **본 세션** |
| 1 | 디버그 인프라 (env logging) + PDF 좌표 정규화 도구 | 없음 (도구만 추가) | 1 |
| 2 | `line_break_char_idx` 다중화 (ls[2..] break 사용) | 중 | 1 |
| 3 | 다중행 인라인 표 baseline 정렬 정책 변경 | 큼 | 1~2 |
| 4 | 인라인 vs 블록 정책 도입 | 매우 큼 | 다 |

## 4. 본 세션 결정 (Phase 0)

코드 변경 없이 분석 + 로드맵만 commit.

근거:
1. 결함 본질 복합. 단일 패치 해결 어려움
2. 광범위 회귀 검증 자동화 (Phase 1) 선결 필요
3. 메모리 가이드 `feedback_essential_fix_regression_risk.md`, `feedback_pdf_not_authoritative.md` 적용
4. 자동승인 모드라도 본질 정정 작업은 단계 분리가 안전

## 5. 산출물

- `mydocs/tech/layout_refactor_roadmap.md` (분석 + 로드맵)
- 본 수행계획서
- 최종 보고서 (`mydocs/report/task_layout_refactor_phase0_report.md`)
- 코드 변경: **없음**

## 6. 후속

후속 세션에서 Phase 1 (인프라) 부터 진행 권고. 작업지시자 결정 시 단계적 진행.
