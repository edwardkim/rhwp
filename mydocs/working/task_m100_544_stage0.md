# Task #544 Stage 0 완료 보고서

**제목**: 사전 분석 + 한컴 환경 검증 입력 대기
**브랜치**: `local/task544`
**이슈**: https://github.com/edwardkim/rhwp/issues/544

---

## 1. ParaShape 정확한 값 확인

페이지 4 col 0 [7~9] 박스 영역 paragraph 들:

| pi | text | ps_id | margin_left | margin_right | indent | bf_id | line_spacing |
|----|------|-------|-------------|--------------|--------|-------|--------------|
| 80 | "[7~9] 다음 글을..." | 10 | **0** | **0** | -2072 | 1 | **716** |
| 81 | (빈) | 25 | 1700 | 1700 | 1980 | 7 | -56 |
| 82 | "평등은..." passage | 11 | **1704** | **1704** | 1984 | **4** | 716 |

pi=82 (passage) 의 `border_fill_id=4` 가 박스 outline 의 stroke 정의. 모든 9개
passage paragraph 에 동일 ps_id=11 적용.

## 2. 박스 좌표 차이 본질 분석

### 2.1 박스 left x / width 차이 — 원인 확정

**ParaShape margin 은 HWP 에서 2배 저장값**으로 디스크에 기록됨.
`style_resolver.rs:655` 에서 `/ 2.0` 로 나눔:

```rust
margin_left: hwpunit_to_px(ps.margin_left, dpi) / 2.0,
margin_right: hwpunit_to_px(ps.margin_right, dpi) / 2.0,
```

pi=82 dump 의 `margin_left=1704 HU` → resolved `margin_left = 852 HU = 11.36 px`.

**현재 산식** (`paragraph_layout.rs:2683-2684`):
```rust
(box_x, box_w) = (col_area.x + box_margin_left, col_area.width - box_margin_left - box_margin_right)
```

→ box_x = 117.17 + 11.36 = **128.53 px** (SVG 와 일치 ✓)
→ box_w = col_width (423.32) - 22.72 = **400.6 px** (SVG 402.5 와 거의 일치)

**PDF 산식 (관찰 기반 역산)**:
```
box_x = col_area.x        (margin 미적용)
box_w = col_area.width    (margin 미적용)
```

→ box_x = 117.0, box_w ≈ 425 px (PDF 측정값과 일치).

### 2.2 박스 top y 차이 — 원인 확정

`paragraph_layout.rs:786`:
```rust
let bg_y_start = if para_border_fill_id > 0 { y_start } else { y };
```

pi=82 의 `y_start` = paragraph_layout 진입 시 y_offset = **sequential 누적값**.

페이지 4 sequential 누적:
- pi=80 advance = lh = 1100 HU = 14.67 px (trailing-ls 716 HU 제외, Task #479)
- pi=81 advance = lh = 1100 HU (음수 ls floor)
- pi=82 시작 = body_area.y + (14.67 + 14.67) ≈ 239 px

그러나 page 4 SVG 박스 top = 224.4 px = body_area.y + 14.67. 이것은 **pi=81 의
border (border_fill_id=7) 가 push 되어 group top 으로 잡힌 결과**. pi=81 시작 =
pi=80 끝 = 224.4 (sequential).

**PDF 박스 top = 233.8 px** = body_area.y + 24.04 px = **pi=80 IR vpos end** (= lh + ls = 1816 HU = 24.21 px). 즉 PDF 는 pi=80 의 line_spacing (716 HU) 을 paragraph border 시작 산출에 포함.

**즉 현재 동작에서는 두 가지 sequential 시프트 누적**:
1. pi=80 trailing-ls 716 HU 제외 (Task #479) → SVG 시작이 9.55 px 위
2. pi=81 sequential advance 가 box top 으로 잡힘 → 추가 shift

PDF 는 pi=82 시작 (= 박스 안 첫 텍스트의 직전 paragraph end) 을 박스 top 으로
사용. SVG 는 pi=81 시작을 사용.

### 2.3 광범위 PDF 패턴 검증

PDF 의 9개 passage 박스 좌표 (한컴 2010 출력):

| 페이지 | col | top y | x range | width |
|--------|-----|-------|---------|-------|
| 2 | 1 | 233.8 | 579.4~1004.5 | 425.1 |
| 3 | 0 | 694.4 | 116.8~542.0 | 425.1 |
| 3 | 1 | 526.5 | 593.1~1004.0 | 410.9 |
| 4 | 0 | 233.8 | 116.8~542.0 | 425.1 |
| 4 | 1 | 490.5 | 579.4~1004.5 | 425.1 |
| 5 | 1 | 233.8 | 579.4~1004.5 | 425.1 |
| 6 | 0 | 700.8 | 116.8~542.0 | 425.1 |
| 7 | 0 | 233.8 | 116.8~542.0 | 425.1 |
| 7 | 1 | 645.3 | 579.4~1004.5 | 425.1 |
| 8 | 1 | 233.8 | 579.4~1004.5 | 425.1 |
| 9 | 0 | 591.7 | 116.8~542.0 | 425.1 |
| 10 | 0 | 233.8 | 116.8~542.0 | 425.1 |
| 10 | 1 | 750.1 | 579.4~1004.5 | 425.1 |
| 11 | 1 | 233.8 | 579.4~1004.5 | 425.1 |
| 12 | 0 | 824.3 | 116.8~542.0 | 425.1 |
| 13 | 1 | 233.8 | 579.4~1004.5 | 425.1 |
| 14 | 0 | 387.8 | 116.8~542.0 | 425.1 |
| 15 | 0 | 233.8 | 116.8~542.0 | 425.1 |

**일관된 패턴**:
- col 0 박스: x=116.8 (≈ body_area.x = 117.17), width=425.1
- col 1 박스: x=579.4 (≈ col 1 시작), width=425.1
- 모든 박스 width 가 ~425 px = col_width 전체

→ PDF 는 paragraph border 를 col 전체 폭으로 그림. paragraph margin_left/right 미적용.

## 3. fix 위치 진단

### 3.1 박스 left/width 정정

`paragraph_layout.rs:2695-2697`:
```rust
// 현재
(col_area.x + box_margin_left, col_area.width - box_margin_left - box_margin_right)
// 정정 후
(col_area.x, col_area.width)
```

또는 wrap host (`border_box_override`) 케이스 보존 + 일반 케이스만 변경.

### 3.2 박스 top y 정정

`paragraph_layout.rs:786`:
```rust
// 현재
let bg_y_start = if para_border_fill_id > 0 { y_start } else { y };
// 정정 후 (옵션 1): pi 의 IR vpos 기반 산출
let bg_y_start = if para_border_fill_id > 0 {
    // prev paragraph 의 IR vpos end 위치 = 현재 paragraph 의 IR vpos 시작 위치 - (sequential drift)
    // y_start 가 sequential 인 경우 trailing-ls 만큼 보정 필요
    y_start_with_trailing_ls_correction
} else { y };
```

또는 빈 paragraph (pi=81) 의 border push skip + group top 산출 시 다음 paragraph
의 IR vpos 사용 (Task #540 Stage 4 와 유사 접근).

## 4. 광범위 영향 / 회귀 위험 평가

### 4.1 박스 left/width 변경 위험

**영향 받는 paragraph 범위**: paragraph border 를 가진 모든 paragraph
(border_fill_id > 0). 즉 21_언어_기출 의 9개 passage 외에도 모든 샘플의
paragraph border 박스 영향.

**위험**:
- synam-001 / 복학원서 / 기타 샘플의 paragraph border 박스 좌표가 달라짐.
- 현재 산식이 일부 케이스에서는 PDF 와 일치할 가능성 (즉 21_언어_기출 만 차이일 가능성).
- wrap=Square 호스트 case (`border_box_override`) 영향 가능.

### 4.2 박스 top y 변경 위험

**영향 받는 paragraph 범위**: 박스 직전 paragraph 가 양수 line_spacing 을 가진
모든 case (Task #479 trailing-ls 제외 영향 받는 모든 박스).

**위험**:
- Task #537 fix (lazy_base + trailing-ls 보정) 와 충돌 가능.
- Task #540 fix (빈 paragraph floor) 와 충돌 가능.
- 셀 내부 paragraph border 회귀 위험.

### 4.3 메모리 룰 적용

- **[feedback_essential_fix_regression_risk]**: 본 정정은 paragraph border 의
  본질적 산출 변경. 광범위 회귀 위험 매우 큼. **한컴 2010/2020 다중 환경 검증 필수**.
- **[feedback_pdf_not_authoritative]**: 한컴 2010 PDF 만으로 정정 결정 위험.
  한컴 2020 / 한컴독스 출력도 비교 필요.
- **[feedback_rule_not_heuristic]**: HWP 표준 명세에 paragraph border 좌표 산출
  룰이 명시되어 있는지 확인 필요. 명세 없으면 hancom 기본 동작 (margin 미적용)
  을 룰로 채택.

## 5. 작업지시자 검증 입력 요청

다음 사항에 대한 입력 부탁드립니다:

1. **다른 샘플 PDF 비교**: synam-001, 복학원서, exam_math 등에서도 paragraph
   border 박스가 PDF 와 다른지 확인. 모든 샘플에서 일관되게 다르면 광범위 정정.
   일부만 다르면 분기 필요.

2. **한컴 2020 / 한컴독스 출력 비교**: 한컴 2010 PDF 외 다른 환경에서 박스
   위치/크기 동일한지 확인. 환경 차이가 있다면 어느 기준 채택할지 결정.

3. **HWP 표준 명세 확인**: HWP 명세에 paragraph border 좌표 산출 룰 명시 여부.
   명세 있으면 룰 채택, 없으면 한컴 동작 (margin 미적용 + IR vpos 기반) 을
   룰로 정의.

4. **fix 범위 결정**:
   - A안: 모든 paragraph border 정정 (광범위 영향)
   - B안: 빈 paragraph (text=∅, controls=∅) 직후의 paragraph border 만 정정 (Task #540 가드 활용)
   - C안: 21_언어_기출 같은 ParaShape (margin_left=1704 등) 만 정정 (heuristic, 권장 안 함)

## 6. 산출물

| 파일 | 변경 |
|------|------|
| `mydocs/plans/task_m100_544.md` | 수행 계획서 (Stage 0 시작) |
| `mydocs/working/task_m100_544_stage0.md` | 본 보고서 |

## 7. 다음 단계

작업지시자 검증 입력 수신 후:
- Stage 1: TDD 통합 테스트 (3+ 박스 좌표 검증) + fix 위치 정밀 진단
- Stage 2: fix 적용 (가드 / 산식 변경)
- Stage 3: 광범위 회귀 검증 (메모리 룰 따라 다중 환경 확인)
