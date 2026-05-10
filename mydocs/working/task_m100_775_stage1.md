# Task #775 Stage 1 보고서 — RED 테스트 + IR 사전 검사

## 결과 요약

- ✅ RED 통합 테스트 작성 (`tests/issue_775.rs`)
- ✅ FAIL 확인 — 회귀 측정값 = +446.61 px
- ✅ calendar_year.hwp / exam_eng.hwp IR 사전 검사 완료
- ⚠️ **옵션 B 폐기**, **옵션 A 채택**

## 1. RED 테스트

### 파일
`tests/issue_775.rs` (+~70 lines)

### 검증 내용
- `samples/exam_eng.hwp` page 4 (0-idx 3) pi=181 ci=1 = 1×1 InFrontOfText 표 (27번 보기 그림 위 데코레이션)
- `build_page_render_tree(3)` 의 RenderTree 에서 Table 노드 bbox 추출
- 정상 y_top ≈ 277.08 px ± 5.0 px 가드

### FAIL 결과

```
[issue_775] page=4 pi=181 ci=1 bbox=[723.69..1146.72] (height=423.03)
pi=181 ci=1 (27번 보기 InFrontOfText 표) 가 단 1 상단(≈y=277.08)에 위치해야 함.
실제 y=723.69 px (PDF 정상값과 차이=446.61 px).
```

회귀 측정값 =bisect 단계의 SVG cell-clip y 분석 결과와 정확히 일치 (+446.61 px).

## 2. IR 사전 검사

### calendar_year.hwp (Task #703 본 케이스)

```
샘플: samples/basic/calendar_year.hwp
용지: 210.0×297.0 mm
구역 0:
  단정의: 1단 (column_count == 1)
  표 IR: 1×1 wrapper, treat_as_char=false, wrap=글뒤로(BehindText)
  vert=문단(643=2.3mm) ← vert_rel_to=Para
  size=51974×2782 HU (183.4×9.8 mm) ← 매우 얇음 (carrier wrapper)
```

### exam_eng.hwp (회귀 케이스)

```
샘플: samples/exam_eng.hwp
용지: 297.0×420.0 mm (A4 가로형 변형 → A3)
구역 0:
  단정의: 2단 간격=11.0mm (column_count == 2) ← 다단
  pi=181 ci=1: 1×1 표, treat_as_char=false, wrap=글앞으로(InFrontOfText)
  vert=문단(672=2.4mm) ← vert_rel_to=Para
  size=30047×31727 HU (106.0×111.9 mm) ← 큼 (그림 위 데코레이션)
```

## 3. 옵션 비교 결론

### 옵션 B (`vert_rel_to == Page` 한정 push-only) — ❌ **폐기**

calendar_year.hwp 의 BehindText 표 `vert=문단(Para)` 이므로 옵션 B 적용 시 push-only 제외 → Task #703 본 케이스 **회귀 발생** (단일 페이지 → 2 페이지로 복귀).

### 옵션 A (`column_count == 1` 한정 push-only) — ✅ **채택**

| 케이스 | column_count | 옵션 A 동작 | 결과 |
|--------|--------------|-------------|------|
| calendar_year.hwp | 1 | push-only (현행 fix 유지) | ✅ Task #703 본 케이스 보존 |
| exam_eng.hwp p4 | 2 | cur_h 누적 (종전 동작 복귀) | ✅ 회귀 해소 |

두 케이스 모두 정합. column_count 가 결정적 차별화 요소 확인.

### typeset.rs 의 column_count 접근

`st.col_count: u16` (typeset.rs:101) — 이미 State 에 존재. 추가 인프라 불필요.

## 4. Stage 2 진행 조건

- Stage 1 RED + IR 검사 결과 본 보고서 승인
- 옵션 A 적용 위치 = `src/renderer/typeset.rs:1553-1563` 의 `matches!(...)` 조건에 `&& st.col_count == 1` 추가
- 변경 라인 = 1 (단순 가드 추가)

## 파일 변경

| 파일 | 변경 종류 | 라인 |
|------|----------|------|
| `tests/issue_775.rs` | 신규 RED 테스트 | +73 |
| `mydocs/plans/task_m100_775.md` | 수행 계획서 (Stage 0) | +94 |
| `mydocs/plans/task_m100_775_impl.md` | 구현 계획서 (Stage 0) | +120 |
| `mydocs/working/task_m100_775_stage1.md` | 본 단계 보고서 | (본 파일) |
