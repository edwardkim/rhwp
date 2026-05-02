# Task #521: 그림(BehindText) vertical extent — 최종 결과 보고서 (보류)

## 결과

**보류 / Layout 리팩터링 Phase 3 흡수.** 본질 진단 완료, 5개 fix 후보 식별, 모두 회귀 위험 또는 비효과로 수정 미적용.

## 진단 결과

### 1. 시각 결함

`samples/exam_eng.hwp` p2 18번 답안지 ① ~ ⑤ 가 PDF 대비 ~3.5mm (~13 px) 위로 올라감. 이메일 안내문 박스 하단과 ① 사이 시각 여백 부족.

### 2. 본질 (2개 독립 — Stage 1 정밀 검증)

#### 본질 1: tac=true 표 다음 paragraph 의 host_line_spacing 누락

`src/renderer/typeset.rs:1230-1237` 및 `src/renderer/layout.rs` 다중 위치에서 tac=true 인라인 표의 host paragraph 의 `line_spacing` (예: pi=104 ls=344 HU = 4.6 px) 이 다음 paragraph 시작 y 계산에 누락. IR vpos 정합 검증으로 확정.

#### 본질 2: ParaShape `spacing_before / spacing_after` `/2.0` 의심

`src/renderer/style_resolver.rs:658-659` 의 `/2.0` 가정. pi=103 의 IR `spacing_after=1000 HU` 가 IR vpos 자체 (pi=104 col-vpos=2254) 와 비교하면 full 값 가정에 더 정합 (gap 96 HU vs 404 HU). 광범위 영향으로 별도 검증 필요.

### 3. Stage 2: 5건 수정 시도 (모두 보류)

| 시도 | 위치 | 결과 |
|------|------|------|
| 1 | typeset.rs:1230 host_line_spacing tac=true 포함 | 0 diff |
| 2 | typeset.rs:1497 fmt.total_height 사용 | synam-001 11p, exam_eng 0 |
| 3 | 1+2 + cap의 ls/2 → full ls | synam-001 34p + aift 13p + exam_science 1p, exam_eng 0 |
| 4 | layout.rs:2683 partial_table follow-up | 0 diff (pi=104 미경유) |
| 5 | layout.rs:2515 control_index → tac_idx | 미시도 (위험 평가 후) |

**target paragraph (pi=104) 가 어느 layout 경로를 타는지 명확하지 않음.** 작은 수정도 광범위 회귀 발생.

## 보류 사유

1. **Pagination vs Layout 분리** — Pagination current_height 정합은 가능, Layout y_offset 다중 경로
2. **회귀 위험 광범위** — 수정 시도 중 synam-001 34페이지 회귀 발생 케이스 있음
3. **검증 도구 부재** — 변경 페이지 PDF 비교 수동 검증 비용 매우 큼
4. **본질 2 광범위 영향** — `/2.0` 변경은 전체 paragraph spacing 영향, 한컴 환경 검증 필수

## 권장 통합 처리

**Layout 리팩터링 Phase 3** 의 일부로 통합:
- Phase 3 = 다중행 인라인 표 baseline 정렬 정책 (`mydocs/orders/20260502.md` 참조)
- 본 task 본질 = 인라인 표 + 후속 paragraph 정합 → Phase 3 의 자연스러운 일부
- Phase 3 진행 조건 = 한컴 환경 직접 PDF 비교 검증 가능 시점

## 산출물

- `mydocs/plans/task_m100_521.md` — 수행계획서
- `mydocs/plans/task_m100_521_impl.md` — 구현계획서
- `mydocs/working/task_m100_521_stage{1,2,3}.md` — 단계별 보고서
- `mydocs/report/task_m100_521_report.md` — 본 보고서

## GitHub Issue #521

- **상태**: open 유지
- **label**: 보류 / Phase 3 dependency
- **재진행 시점**: Layout 리팩터링 Phase 3 진행 시

## 분류

| 항목 | 결과 |
|------|------|
| 진단 | ✅ 완료 (본질 2개 식별 + IR vpos 정합 검증) |
| 수정 | 🟡 보류 (5건 시도 후 보류) |
| 회귀 영향 | 0 (수정 미적용) |
| 다음 단계 | Phase 3 흡수 |
