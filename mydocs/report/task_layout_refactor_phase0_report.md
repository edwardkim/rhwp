# Layout 리팩터링 Phase 0 최종 보고서

**브랜치**: `local/task_layout_refactor`
**Phase**: 0 (분석 + 로드맵)
**상태**: 완료 (코드 변경 없음, 분석 + 로드맵만 commit)

## 1. 작업 내용

`#467 / #491 / #496` 보류 사유 "layout 리팩터링 시 종합 해결" 의 구체화 분석.

상세: `mydocs/tech/layout_refactor_roadmap.md`

## 2. 핵심 결과

### 2-1. 결함 본질 분리

3개 보류 이슈는 본질이 다른 별개 영역:

- **#467**: master 매핑 (layout 모듈과 별개, `document_core/queries/rendering.rs`)
- **#491**: PDF 좌표 정규화 (검증 도구 인프라 부재)
- **#496**: `layout_inline_table_paragraph` 다중행 표 + 다중 줄 본문 한계

### 2-2. #496 결함 — 복합 본질 (A/B/C)

`exam_science.hwp` p2 pi=61 재현 분석:
- 본문 baseline 1195.85 가 표 row 0 (1191.68) / row 1 (1210.77) 사이 끼임
- 시각적으로 본문이 표와 겹쳐 보임

세 가지 본질 식별:
- (A) 본문 baseline 수직 정렬 정책 부재
- (B) ls[2..] break 미사용 (dynamic right_margin reflow 사용)
- (C) 인라인 vs 블록 정책 부재

## 3. 로드맵

| Phase | 내용 | 리스크 |
|-------|------|--------|
| 0 (본) | 분석 + 로드맵 | 없음 |
| 1 | 디버그 인프라 + PDF 좌표 정규화 도구 | 없음 |
| 2 | line_break_char_idx 다중화 | 중 |
| 3 | 다중행 인라인 표 baseline 정렬 | 큼 |
| 4 | 인라인 vs 블록 정책 도입 | 매우 큼 |

## 4. 변경 파일

| 파일 | 변경 |
|------|------|
| `mydocs/tech/layout_refactor_roadmap.md` | 신규 (분석 + 로드맵) |
| `mydocs/plans/task_layout_refactor_phase0.md` | 신규 (수행계획서) |
| `mydocs/report/task_layout_refactor_phase0_report.md` | 신규 (본 보고서) |

코드 변경 없음.

## 5. 후속

후속 세션에서 Phase 1 (디버그 인프라) → Phase 2 (line_break 다중화) 순으로 진행 권고.

각 Phase 별로 별도 task / 별도 issue 등록 후 단계적 처리.
