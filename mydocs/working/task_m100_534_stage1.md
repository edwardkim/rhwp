# Task #534 Stage 1 — Root Cause 위치 조사

**작성일**: 2026-05-02
**이슈**: [#534](https://github.com/edwardkim/rhwp/issues/534)
**브랜치**: `local/task534`
**범위**: 조사만 (코드 변경 0)

## 1. 결론 — 1차 조사 완료, **본질 다중 영역 가능성 확인**, 추가 조사 필요

> 페이지 18 우측 단 pi=50/56 (Square wrap 인라인 표 + tac=true 그림) 의 이미지 x 좌표가 단독 그림 케이스 (pi=46/54) 와 비교해 약 60.66 px 좌측에 위치. 본 결함은 (1) Picture 가 wrap host paragraph 안에서 emit 되는 경로의 모호성, (2) HWP IR 측 음수 horz_offset i32 wraparound, (3) ParaShape indent 부호별 effective_margin_left 계산 형식의 3 영역 모두 영향 가능성. **명확한 단일 root cause 미확정**, 추가 디버그 로깅 + 코드 path 추적 권장.

## 2. 측정 정밀화

### 2-1. 현재 SVG 이미지 x 좌표

| pi | ParaShape | indent | image x | image w | image x_end |
|----|----------|--------|---------|---------|-------------|
| 46 | ps_id=52 | **+7400** | **654.05** | 306.80 | 960.85 |
| 50 | ps_id=53 | **-3800** | **593.39** | 368.05 | 961.44 |
| 54 | ps_id=52 | **+7400** | **654.05** | 278.25 | 932.30 |
| 56 | ps_id=40 | **-3800** | **593.39** | 368.27 | 961.66 |

→ indent ≥ 0 (pi=46/54): image x = 654.05
→ indent < 0 (pi=50/56): image x = 593.39
→ Diff = 60.66 px

### 2-2. col_area.x 추정

우측 단 텍스트 위치 측정 (pi=51 첫 글자 "학" x=604.72, margin_left=22.67 px) → col_area.x ≈ 582.05.

### 2-3. effective_margin_left 가설 검증

`src/renderer/layout.rs::layout_shape_item` line 2798-2811:

```rust
let effective_margin_left = if para_indent > 0.0 {
    para_margin_left + para_indent
} else {
    para_margin_left
};
let pic_x = match para_alignment {
    ...
    _ => col_area.x + effective_margin_left,  // Left, Justify
};
```

기대값:
- pi=46/54 (indent +7400): effective_margin = 1700 + 7400 = 9100 HU = 121.33 px → pic_x = 582.05 + 121.33 = **703.38**
- pi=50/56 (indent -3800): effective_margin = 1700 = 22.67 px → pic_x = 582.05 + 22.67 = **604.72**

관측값:
- pi=46/54: 654.05 (기대 703.38, 차이 **-49.33 px**)
- pi=50/56: 593.39 (기대 604.72, 차이 **-11.33 px**)

→ 기대값과 관측값 모두 차이 발생. layout_shape_item 단일 경로로는 설명 불가.

## 3. 결함 영역 가설 (다중 영역)

### 3-1. 가설 A — 다른 emit 경로

`layout_shape_item` 외에도 Picture (tac=true) 가 emit 되는 경로 존재:
- `paragraph_layout.rs::layout_composed_paragraph` line 2153-2222 (빈 문단 + TAC Picture 분기)
- `layout_wrap_around_paras::layout_partial_paragraph` 내부 호출 (has_host_text=False 면 미동작 — pi=50 적용 안 됨)
- `paragraph_layout.rs::layout_inline_table_paragraph` (TAC 표만 처리, pi=50 제외)

**조사 필요**: layout_shape_item 의 `already_registered` 분기가 True 인지 (다른 경로에서 미리 등록됨), 그 등록 좌표가 다른 공식 사용 중인지.

### 3-2. 가설 B — HWP IR i32 wraparound

```
pi=50 그림: 위치: 가로=단 오프셋=15151598.2mm(4294941220) 정렬=Left
```

`4294941220` (u32) = `-26076` (i32) → 음수 horz_offset 이 unsigned 로 표시되어 dump 값 손상. `cur` 측도 동일 (`cur=27604×4294963926`).

→ Parser 가 i32 → u32 캐스팅 시 부호 손실, 또는 layout 가 unsigned 값으로 처리하여 거대한 양수 offset 으로 잘못 더해질 가능성. **단**, pi=46/54 dump 도 동일한 4294941220 값을 가지므로 (wraparound 가 동일하게 발생) 단순 i32 wraparound 만으로는 pi=46/54 vs pi=50/56 차이 설명 불가.

### 3-3. 가설 C — Square wrap 표 영역의 col_area 좁히기

pi=50/56 은 Square wrap 인라인 표 [A]/[B] 가 같은 paragraph 에 존재. layout_table_item 의 wrap_area 또는 layout_shape_item 의 col_area 가 표 폭만큼 좁아진 wrap_area 로 처리될 가능성. 

→ wrap_area.x = wrap_text_x - host_margin_left (line 3057-3058). 이 값이 layout_shape_item 에 영향?

## 4. 추가 조사 권장

| 조사 | 방법 |
|------|------|
| Picture (tac=true) 의 실제 emit code path | `eprintln!("EMIT_TAC_PIC: pi={} ci={} x={} y={} src={}")` 임시 추가, pi=50/56 추적 |
| `already_registered` 발동 여부 | `tree.set_inline_shape_position` 호출 위치 모두 로그 |
| `pic_x` 계산 시 col_area.x 값 | pi=46 vs pi=50 실제 col_area.x 비교 |

## 5. Stage 2 진행 권고 (옵션)

### 옵션 A — 추가 조사 (권장)

본 단계 추가 조사 후 Stage 2 정확한 fix 설계. 회귀 위험 최소화.

### 옵션 B — 보류 / 별도 task

본 결함 패턴이 광범위 영역 (Picture emit 경로 + HWP IR wraparound) 영향 가능성 → 단일 task 로 정정 어려울 수 있음. 별도 조사 task (pi 별 경로 측정) 후 본격 fix.

### 옵션 C — Task #533 패턴 적용 시도

가설 C (Square wrap 표 영역) 가 본질이면 `layout_table_item` 경로에서 Picture x 보정. 단 Stage 1 에서 root cause 명확하지 않아 위험.

## 6. 메모리 정합

- `feedback_essential_fix_regression_risk` 정합 — 본 결함은 Picture emit 경로 다중성 + HWP IR i32 wraparound 영역으로 광범위. 충분한 측정 없이 정정 시도 위험
- `feedback_pdf_not_authoritative` — 한컴 PDF 와 작업지시자 시각 판정으로 fix 효과 검증 필수

## 7. 산출물 (본 단계)

| 산출물 | 내용 |
|--------|------|
| 본 보고서 | Root cause 1차 조사 + 가설 다중 영역 식별 |
| 코드 변경 | **0** (조사만) |
| 측정 데이터 | SVG image x 좌표 (4 pi), HWP IR dump |

## 8. 다음 단계

작업지시자 결정 영역:
- (A) 추가 조사 (eprintln 임시 로깅 + 정확한 path 식별)
- (B) 본 task 보류 / 별도 조사 task 분리
- (C) 가설 C 기반 Task #533 패턴 시도 (위험)

## 9. 승인 게이트

- [x] 측정 데이터 (SVG image x 좌표 4 pi 비교)
- [x] HWP IR dump 분석 (i32 wraparound 발견)
- [x] 가설 다중 영역 식별 (A/B/C)
- [ ] 단일 root cause 확정 (추가 조사 필요)
- [ ] 회귀 안전 fix 위치 확정 (Stage 2 영역)
