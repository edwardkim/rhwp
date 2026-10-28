# Task M100 #1317 Stage 1 완료보고서 — 적분 글리프 metric 모델 + layout attach point

## 목표

적분기호 path 렌더와 상·하한 attach point가 공유할 **단일 기준(SSOT)**을 layout에 도입하고, 기존 매직 offset을 metric 기반 계산으로 치환한다.

## 변경 내용 (`src/renderer/equation/layout.rs`)

### 1. `IntegralGeom` 모델 + `integral_geom(fs)` 신설

적분 글리프 박스(좌상단 원점, 높이 `fs*INTEGRAL_SCALE`) 기준 상대 좌표로 글리프 형상과 attach 기준점을 정의:

| 필드 | 값(fs 비율) | 의미 |
|------|------------|------|
| `width` | 0.52 | 글리프 가로 폭(advance) |
| `stroke_w` | 0.06 | 줄기 stroke 두께 |
| `top_y` | h*0.04 | path 상단 |
| `bottom_y` | h*0.96 | path 하단 |
| `top_hook_x` | 0.50 | 상단 갈고리 우측 끝 (상한 attach) |
| `bottom_hook_x` | 0.04 | 하단 갈고리 좌측 끝 (하한 attach) |

(h = fs*INTEGRAL_SCALE)

### 2. `layout_subsup()` 적분 분기 재작성

- 기존 매직 offset(`sup fs*0.21`, `sub fs*0.55`, `top_pad fs*0.13`, `sub_x -fs*0.42`) 제거.
- `integral_geom` 기반으로 attach point 산출:
  - 상한(sup): 상단 갈고리 우측(`top_hook_x`), 글리프 최상단 근처 (`top_y - sp.height*0.30`)
  - 하한(sub): 하단 갈고리 우측(`bottom_hook_x`), 글리프 최하단 근처 (`bottom_y - sb.height*0.72`)
  - 상한이 박스 위로 넘치면 글리프를 그만큼 아래로 내림(`head`)
- `total_w`는 첨자 우측 끝 + `BIG_OP_TRAIL_PAD`로 산출(첨자가 가장 오른쪽).

### 3. `layout_math_symbol()` bare 적분 advance

- 글리프를 path로 그리므로 advance를 `geom.width + BIG_OP_TRAIL_PAD`로 변경 (기존 `estimate_text_width` 제거).

## 검증

- `cargo build --release` 성공.
- `cargo test` 전체 통과 (8 + 1 + doc, 0 failed).
- `test_big_op_trailing_pad`(∑ 대상) 무영향 확인.

## 비고

렌더는 아직 `<text>`라 이 단계 단독 SVG는 중간 상태(글리프-attach 불일치). 실제 시각 정합은 Stage 2(path 렌더)에서 검증.
