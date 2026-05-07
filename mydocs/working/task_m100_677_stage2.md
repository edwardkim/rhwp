# Stage 2 — 결함 #1 본질 정정 보고서 (Task #677)

## 본질 정정 영역 — 두 영역 결합

### 1. PartialParagraph y 누적 결함 (Stage 1 진단 영역)

**대상**: `src/renderer/layout.rs:2120-2147` `PageItem::PartialParagraph` 분기

**정정 내용**:
- TAC 표 보유 paragraph 의 PP (start_line > 0) 진입 시 `y_offset` 을 LineSeg.vpos 정합 위치로 리셋
- 조건 가드 3개로 좁게 발동: (1) start_line > 0, (2) para 가 TAC 표 보유, (3) para_start_y 등록
- y_offset 누적: `y_offset = y_offset.max(pp_y_out)` — Table item 누적값과 PP 자연 종료값 중 최대로 갱신

**결과**: pi=16 PP 의 LAYOUT_OVERFLOW 273.1px → 2.5px (96% 감소, tolerance 영역)

### 2. U+F081C HWP PUA 채움 문자 폭 결함 (Stage 2 진단 영역, 신규)

**Stage 2 진단으로 추가 식별** — Stage 1 디버그 추적 후 검증 단계에서 시각 결함이 잔존함을 발견:
- pi=16 의 ComposedLine cl[0] 가 99 chars × U+F081C 채움 문자 보유
- `compute_tac_leading_width` (`layout.rs:3563-3610`) 가 block-TAC 케이스에서 line 0 의 모든 run 텍스트 폭 합산
- `estimate_text_width` 의 default fallback 이 U+F081C 를 `font_size * 0.5` 로 측정 → 99 × 6.65 = 658px 의 leading width 추가
- 결과: 3×3 접수증 표가 col_left + 658 = **x=716px (body 우측 끝)** 로 배치 → 본문 우측 외곽

**대상**: `src/renderer/layout/text_measurement.rs` `char_width` 클로저 5개 사이트 (lines 192, 318, 593, 711, 940)

**정정 내용**:
```rust
// [Issue #677] HWP PUA 채움 문자 (U+F081C) — 시각 폭 0
// 한컴이 인라인 TAC 표/도형 앞에 삽입하는 placeholder 채움 문자.
// 한컴 PDF 정합 — 폭 0 으로 라인 inline x 에 영향 없음. fillers 가
// 표 너비만큼 (≈97 chars × 1 char width = table width) 채워져
// 표가 fillers 영역 위에 시각적으로 겹쳐 column-left 출력 패턴.
if c == '\u{F081C}' {
    return 0.0;
}
```

**결과**: pi=16 의 3×3 접수증 표가 **x=716 → x=63.69 (body left margin)** 로 정합 — 한컴 PDF 정답지 영역 일치

## 정량 측정 (BEFORE → AFTER)

| 측정 항목 | BEFORE | AFTER | Δ |
|----------|--------|-------|---|
| pi=16 LAYOUT_OVERFLOW (PP) | y=1357.8 overflow=273.1px | y=1087.2 overflow=2.5px | -270.6px |
| pi=16 LAYOUT_OVERFLOW (Shape) | y=1357.8 overflow=273.1px | y=1087.2 overflow=2.5px | -270.6px |
| 3×3 접수증 표 x | 716.69 (body 우측 끝) | 63.69 (body left + margin) | -653.0px |
| cell-clip-174 (첫 셀) | x=716.69 y=774.19 | x=63.69 y=774.19 | x: -653.0px |
| cell-clip-177 (둘째 셀) | x=898.03 y=774.19 | x=245.03 y=774.19 | x: -653.0px |
| cell-clip-182 (셋째 셀) | x=1177.81 y=774.19 | x=524.81 y=774.19 | x: -653.0px |

**시각 정합**: PNG 렌더 결과 — "복 학 원 서 접 수 증" + "Filing Receipt" + "대학(Name of College) :" + "학과/학부(Department/Major) :" + "학번(Student No.)：" + "성명(Name)：" + "위 학생의 복학원서를 접수함" + "The above student's reinstatement form is hereby received" + "년(year) 월(month) 일(day) ㊞" + "※ 군필자..." + "※ Those who completed..." 모든 영역이 PDF 정답지와 동일 위치에 정합 출력

## 회귀 검증

### 결정적 검증

```
cargo test --release --lib       1155 passed (회귀 0)
cargo test --release             전체 GREEN
cargo clippy --release --lib     0 warnings (lib 영역)
cargo build --release            빌드 성공
```

### 회귀 가드 통과

- svg_snapshot **7/7 passed** (issue_147 + issue_171 + issue_437 + issue_546 + issue_554 + issue_578 + issue_617)
- issue_554 **12/12 passed** (exam_kor 영역 회귀 0)
- issue_617 1/1 (exam_kor padding shrink 영역 회귀 0)
- issue_546 1/1

### U+F081C 영향 범위 좁힘

**광범위 fixture sweep** (samples/ 161+ HWP):

```
복학원서.hwp contains U+F081C
Total: 1 fixtures with U+F081C
```

→ U+F081C 변경의 영향을 받는 fixture 는 **본 fixture 1개만**. 다른 161+ fixture 의 회귀 위험 영역 0.

**케이스별 명시 가드 정합** (`feedback_hancom_compat_specific_over_general`) — U+F081C 만 명시 정정, 다른 PUA 영역 (U+F02B0..F02FF, U+F0000..F00CF, U+F00D0..F09FF 등) 모두 무영향.

## HWP 정합 영역의 단일 룰 정합 (`feedback_rule_not_heuristic`)

### PP y 리셋 룰

- **HWP IR 표준 직접 사용**: LineSeg.vpos 정합 위치 = `para_start_y[para_index] + (ls[start_line].vpos - ls[0].vpos)` — 측정 의존 없음
- **조건 가드**: 3개 모두 HWP IR 영역 검사 (start_line / para.controls / para_start_y) — 휴리스틱 없음

### U+F081C 폭 룰

- **HWP PUA 채움 문자 정합**: 한컴 인라인 TAC 표/도형 placeholder 채움 문자, 시각 폭 0
- **HWP 명세 정합**: 채움 문자가 표 너비 ≈ 97 chars × 1 char width = 표 너비 → 표 placeholder 영역 정합 (표 가 채움 영역 위에 겹쳐져 column-left 출력)
- **단일 char 매핑**: U+F081C 만 명시, 광범위 PUA 룰 없음

## 잔존 영역 (Stage 3 영역)

**결함 #2 워터마크 효과 미적용** — 본 PNG 에서 여전히 고려대학교 엠블럼이 진한 회색으로 본문 가림 영역. Stage 3 에서 본질 정정.

**※ 군필자 영역** — 본 PNG 에서 출력 위치 정합 (페이지 하단에 정상 배치). 워터마크 영역 외 모든 receipt 영역 시각 정합.

## 디버그 코드 제거 확인

`src/renderer/layout/paragraph_layout.rs` 임시 `eprintln!` (`[DBG677]` 태그) 모두 제거 + clean build 통과.

## 변경 LOC

| 파일 | 변경 | 영역 |
|------|------|------|
| `src/renderer/layout.rs` | +30 / -2 | PageItem::PartialParagraph y 리셋 + max 누적 |
| `src/renderer/layout/text_measurement.rs` | +20 / 0 | U+F081C 폭 0 (5 사이트) |
| **합계** | **+50 / -2** | 단일 룰 2개 |

## 승인 요청

본 Stage 2 결과 승인 후 **Stage 3 (결함 #2 워터마크 정정)** 진행하겠습니다.

대상 영역 (Stage 3): `src/renderer/svg.rs::ensure_brightness_contrast_filter` + `src/renderer/web_canvas.rs::compose_image_filter` — `is_watermark()` 게이트 + 한컴 워터마크 변환 적용. Stage 1 진단의 가설 A (한컴 편집기 워터마크 모드 시 저장값 변환) 영역 검증 + 정정.
