# Task #297 1단계 완료 보고서 — 원인 확정 및 수정 범위 확정

## 1단계 목표

이슈 #297(바탕쪽 "* 확인 사항" 표 배치 오류)의 근본 원인을 확정하고, 수정 범위(어떤 조건에만 적용할지)를 결정한다.

## 조사 내용

### 1. 정확한 좌표 측정 (pdftotext -bbox-layout)

PDF 좌표(포인트 → 96dpi 픽셀 변환):

| 요소 | PDF yMin(pt) | PDF 픽셀 |
|------|-------------|----------|
| "확인" (header 첫 단어) | 923.60 | **1231.5** |
| "답안지의" (bullet 1) | 946.52 | 1262.0 |
| "이어서" (bullet 2) | 985.52 | 1314.0 |
| "확인하시오" | 1002.68 | 1336.9 |
| 페이지번호 "20" | 1039.83 | 1386.4 |

SVG 좌표(translate의 y = baseline, font-size=15.33 기준 text top ≈ y-11.8):

| 요소 | SVG translate y | SVG text top |
|------|----------------|--------------|
| "* 확인 사항" | 1385.47 | **1373.7** |

**결정적 측정치**: SVG text top(1373.7) − PDF text top(1231.5) = **+142.2 px 드리프트**

### 2. HWP 원본 데이터 (바탕쪽 표)

```
vert=Paper/101954 HU  vert_align=Top  wrap=TopAndBottom
size=58408x2196 HU    margin_top=1417  margin_bottom=283  attr=0x082a2400
master_text_area: 66614×90994 HU  (body_area와 일치)
```

### 3. 핵심 관계식 발견

- `101954 HU = 1359.4 px` (현재 SVG 표 top 위치)
- `body_area.bottom = 1360.7 px ≈ 101981 HU` ≈ **v_offset (오차 27 HU = 0.1mm)**
- PDF 표 top 추정 = 1214 px (text top 1231.5 − 셀 padding/베이스라인 17.5)
- **1359.4 − 1214 = 145.4 px ≈ 렌더된 표 높이**

→ **한컴 뷰어는 v_offset=101954를 "표의 BOTTOM 위치(용지 상단 기준)"로 해석**한다. 즉 vert_align 값과 무관하게 Bottom-anchored처럼 동작.

공식: `table_top = v_offset − rendered_table_height`

### 4. 수정 범위 — 다른 샘플 조사 (30건 바탕쪽 표)

모든 145개 `samples/*.hwp`를 스캔한 결과:

| 패턴 | 샘플 | v_offset | valign | wrap |
|------|------|----------|--------|------|
| **exam_math(_8)** | 2개 파일 | 101954 | Top | TopAndBottom |
| exam_kor, exam_eng | 2개 파일 | 9921 | Top | TopAndBottom |
| exam_science, exam_social | 2개 파일 | 5102 | Bottom | BehindText |
| exam_kor, exam_science | 2개 파일 | 0 | Bottom | TopAndBottom |

exam_eng/exam_kor의 `9921 HU = 34.9mm` (머리말 영역). 현재 코드(paper-top 기준)로 132.3 px 위치에 렌더 → 머리말 영역 내부. 자연스러운 위치이며 **수정 불필요**.

exam_math(_8)의 `101954 HU`만 body_area 하단 경계 근처(본문 바닥)를 나타냄. 이 케이스만 현재 동작이 어긋남.

### 5. 구분 조건 도출

"표의 BOTTOM 기준 배치"로 해석해야 하는 케이스의 특징:
- **`v_offset ≥ body_area.bottom` 근처** (즉, paper-top 기준으로 body 영역 밖)
- 바탕쪽(master page) 문맥
- `wrap=TopAndBottom`
- `valign=Top`

구체 기준: `v_offset * dpi/7200 ≥ body_area.y + body_area.height - margin_bottom_hu`로 **본문 하단에 근접**하면 Bottom-anchored 배치로 처리.

## 원인 확정

**한컴 뷰어의 바탕쪽 표 배치 규칙**: `valign=Top` + `VertRelTo::Paper`일 때, `vertical_offset`이 body 영역을 크게 초과하면 이를 "표 바닥(BOTTOM)의 용지 상단 기준 y 좌표"로 해석. 표가 content에 따라 위로 확장되어 v_offset 위치에 바닥이 닿도록 한다.

한컴 파일 포맷에 `valign=Top` + v_offset을 "표 BOTTOM 기준"으로 저장하는 것은 **spec의 문자적 해석과 어긋나는 한컴 내부 관행**으로 보인다. 사용자가 UI에서 표를 body 하단에 배치하면 Hancom이 이렇게 저장하는 것으로 추정.

## 수정 범위 결정

**좁은 범위**에 한정한다:

✅ **적용**:
- 바탕쪽(master page) 문맥
- `VertRelTo::Paper`
- `wrap=TopAndBottom`
- `valign=Top`
- `v_offset` 위치가 body_area 하단 근처이거나 초과 (heuristic)

❌ **미적용** (현 동작 유지):
- 본문(body) 문단의 모든 표 (본 수정의 영향권 밖)
- 바탕쪽이더라도 `wrap=BehindText/Square/InFrontOfText`
- 바탕쪽이더라도 v_offset이 body 상단 근처(exam_eng/exam_kor 머리말 케이스)
- `valign=Bottom/Center/Inside/Outside` (기존 로직 그대로 사용)

## 회귀 영향 범위

- **영향을 받는 파일**: exam_math.hwp, exam_math_8.hwp (v_offset=101954 패턴)
- **영향 없음 파일**: 나머지 143개 샘플 (v_offset이 body 상단 영역이거나 valign≠Top)
- **본문 표**: 완전 무관 (compute_table_y_position에서 바탕쪽 문맥 분기 추가 예정)

## 구현 지점 (예상)

- `src/renderer/layout/table_layout.rs:961-1036` `compute_table_y_position`에 "바탕쪽 Paper+TopAndBottom+Top+body 하단" 케이스 분기 추가
- 분기 내 로직: `table_top = v_offset_px - rendered_table_height`
  - `rendered_table_height`는 `measured_tables[...]`에서 조회 (호출 체인 확인 필요)

## 1단계 결론

- 원인 확정: 바탕쪽 표 `valign=Top` + `VertRelTo::Paper` + body-하단 v_offset의 해석 차이
- 수정 범위: 바탕쪽 + TopAndBottom + valign=Top + v_offset≥body_bottom 조건 한정
- 회귀 위험: 매우 낮음 (exam_math 계열 2개 파일 외에는 변화 없음)

## 승인 요청

위 원인 확정 및 수정 범위로 진행해도 되는지 승인 부탁드립니다. 승인 후 2단계(구현 계획서 `plans/task_m100_297_impl.md` 작성)로 진입합니다.

## 참고 파일

- `samples/exam_math.hwp` (p12), `samples/exam_math.pdf` (p12)
- `/tmp/task297_cmp/exam_math_012.svg`, `/tmp/task297_cmp/pdf_p12-12.png`
