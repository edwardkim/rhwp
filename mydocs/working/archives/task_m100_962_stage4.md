# Task #962 Stage 4 — 다중 sample 회귀 검증

## 1. cargo test --release --lib

**결과**: **1288 passed, 0 failed, 2 ignored**

## 2. 시험지 page 2 (Fix B 검증)

### Before fix
```
보기 textbox equations: 12 (= 6 expected × 2 duplicates)
Set 1 (gap 위치, 정상): (444,449) (474,467) (469,485) (560,485) (443,503) (568,503)
Set 2 (duplicate at x=406): (406,448) (406,466) (406,484) (431,484) (475,484) (540,484)
```

### After fix
```
보기 textbox equations: 6 ✓ (Set 1 만)
시각: ㄱ. h(1)=3 / ㄴ. 함수 h(x)는... / ㄷ. 함수 g(x)가 닫힌구간 [-1, 1]에서...  ← 한컴 정합
```

문12, 문14, 문15 의 모든 textbox content 한컴 PDF 정합.

## 3. LAYOUT_OVERFLOW 회귀 검증

| Sample | Pre-Fix B | Post-Fix B | 차이 |
|--------|-----------|------------|------|
| exam_kor | 28 | 28 | 0 |
| exam_math | 0 | 0 | 0 |
| exam_eng | 13 | 13 | 0 |
| hwp3-sample14 | 0 | 0 | 0 |
| 3-11월_실전_통합_2022 | 284 | 284 | 0 |
| **Total** | **325** | **325** | **0** |

→ Fix B 가 추가 회귀 도입 안 함. 모든 overflow 가 pre-existing.

## 4. 다른 inline 컨트롤 검토 (Picture/Shape/Table)

shape_layout 두번째 loop 의 다른 분기:
- Shape (Control::Shape): `layout_shape_object` 호출 — 자체 등록 path 있음 (별도 path)
- Picture (Control::Picture): `layout_picture` 호출 — 동일 분석 필요시 별도 task
- Table (Control::Table): nested table 처리

본 fix 는 Equation 만 정정 (시험지 page 2 의 결함 영역). 동일 패턴이 다른 control type 에 있으면 별도 issue 등록 필요.

→ 다른 control duplicate 미발견 (시험지 page 2 시각 검증 + LAYOUT_OVERFLOW 회귀 0).

## 5. 평가

- 단위 검증 (시험지 page 2 보기 textbox): ✓
- cargo test 전체: ✓ (1288/0/2)
- LAYOUT_OVERFLOW 회귀: ✓ 0
- 한컴 PDF 정합: ✓

→ Stage 5 진행 (commit + PR).
