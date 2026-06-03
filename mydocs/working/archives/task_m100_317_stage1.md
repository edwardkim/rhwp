# Task #317 1단계 완료 보고서: paragraph current_height 추적

상위: 구현 계획서 `task_m100_317_impl.md`

## 진단 도구

`src/renderer/typeset.rs` 에 `RHWP_TYPESET_TRACE=N` env-gated trace 추가 (4단계에서 제거).
임시 진단 테스트 `tests/task317_diag.rs` 추가.

## 핵심 발견

**차이 origin: 표 paragraph (Table) 처리에서 paragraph당 ~2.7~3.0px 추가 누적**.

### 페이지 3 paragraph-by-paragraph 누적 추적 (sec0 = direct, sec0 reloaded는 두 번째)

| pi | text/type | direct post cur_h | reloaded post cur_h | 차이 |
|----|-----------|-------------------|---------------------|------|
| 33 | Table 1x3 | 41.0 | 43.7 | **+2.7** |
| 34 | (빈) | 44.7 | 47.4 | +2.7 |
| 35 | (빈) | 51.1 | 53.8 | +2.7 |
| 36 | (빈) | 54.9 | 57.5 | +2.6 |
| 37 | (빈) | 61.3 | 63.9 | +2.6 |
| 38 | "(총투자 기준)" | 85.3 | 87.9 | +2.6 |
| 39 | (빈) | 93.3 | 95.9 | +2.6 |
| 40 | "1. 분기별 동향" | 121.0 | 123.7 | +2.7 |
| 41 | Table 4x11 | 213.7 | 219.3 | **+5.6** (+2.9 표) |
| 42 | (빈) | 232.8 | 238.4 | +5.6 |
| 43 | "□ 업종별..." | 254.9 | 260.4 | +5.5 |
| 44 | (빈) | 260.7 | 266.3 | +5.6 |
| 45 | Table 12x11 | 535.3 | 543.7 | **+8.4** (+2.8 표) |
| 46-48 | (빈) | 564.1 | 562.9 | (재정렬) |
| 49 | "□ 국가별..." | 584.1 | (?) | |
| 50 | (빈) | 587.0 | (?) | |
| 51 | Table 12x11 | **1268.5** | (다음 페이지) | |

**패턴**:
- 표 paragraph (pi=33, 41, 45) 마다 ~2.7~2.9px 추가 누적
- 빈 paragraph 는 차이 없음 (표 누적 차이만 전파)
- 페이지 3 끝까지 약 +20px 누적 (예상)
- pi=51 (Table 326.7px) 가용 공간 부족으로 다음 페이지로 → +1쪽

### 차이 origin 위치

`typeset_table_paragraph` (또는 그 호출 chain) 가 같은 paragraph 에 대해 direct vs reloaded 에서 ~2.7px 다른 height contribution.

원인 후보:
- 표의 cell paragraphs IR 차이 (#314 normalize 가 미적용된 셀 내부 필드)
- 표 host_spacing (spacing_before/after) 차이
- 표 outer_margin / cell_spacing 차이
- raw_ctrl_data 의 변환 후 미세 차이

## 산출

- typeset.rs trace 추가 (env-gated, 4단계에서 제거)
- tests/task317_diag.rs (임시, 4단계에서 제거)
- 본 보고서

## 다음 단계

2단계: pi=33 의 표 IR 필드 direct vs reloaded 비교 → +2.7px contribution 의 정확한 origin 식별. typeset_table_paragraph 내부 trace 강화.
