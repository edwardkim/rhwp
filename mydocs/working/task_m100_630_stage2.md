---
issue: 630
milestone: m100
branch: local/task630
stage: 2 — 단위 테스트 작성 (RED)
created: 2026-05-06
status: 완료 — 승인 대기
---

# Task #630 Stage 2 완료 보고서 — 단위 테스트 작성 (RED)

## 1. 작성한 테스트 (3 건)

### 1-1. 단위 테스트 (`src/renderer/layout/text_measurement.rs::tests`)

#### `test_630_middle_dot_full_width_in_registered_font`

- **목적**: 등록된 한글 폰트 (DotumChe) 에서 `·` (U+00B7) advance 가 전각 (= em_size = font_size) 으로 측정되는지 검증.
- **현재 RED**: `dot_advance = 8.65` (반각). 기대 = 17.33 (전각).
- **정정 효과 (Stage 3)**: `is_halfwidth_punct` 의 `U+00B7` 제거 → 폰트 메트릭 그대로 사용 → 전각.

#### `test_630_native_inline_tab_right_align`

- **목적**: native 경로에서 인라인 탭 RIGHT (HWP 인코딩 `(2<<8)|3 = 515`) 가 LEFT fallback 이 아닌 정확한 RIGHT 정렬로 처리되는지 검증.
- **현재 RED**: `B_x = 405.00` (LEFT fallback = a_width + tab_width_px). 기대 `B_x = 395.00` (RIGHT = expected_right - seg_w).
- **정정 효과 (Stage 4)**: `tab_type = ext[2]` → `inline_tab_type(ext)` + match `1/2 → 2/3` 로 정합.

### 1-2. 통합 테스트 (`tests/issue_630.rs`)

#### `test_630_aift_p4_toc_paren_alignment`

- **목적**: aift.hwp 페이지 4 의 모든 `(페이지 표기)` 시작 `(` x 좌표가 단일 정렬 그룹 (±1.0px) 안에 들어오는지 검증.
- **현재 RED**: `lines=23 spread=9.08px` (예상 ≤1.0). 8.67px 이탈 = `·` 반각/전각 차이.
- **추출 로직**: SVG `<text>` 요소에서 y 별 글자 묶음 → "페이지" + "표기" 포함 + 본문 영역 시작 (x<200) 라인의 "페" 직전 마지막 `(` x 채택. 6-2 라인 같이 "(협약…)" 도 포함된 경우 "(페이지" 의 `(` 만 정확 추출.
- **정정 효과 (Stage 3+4)**: `·` 포함 6 라인이 `·` 미포함 16+ 라인과 동일 그룹으로 정합 → spread ≤ 1.0.

## 2. RED 확인 결과

```
$ cargo test --lib --release test_630
test test_630_middle_dot_full_width_in_registered_font ... FAILED
test test_630_native_inline_tab_right_align ... FAILED

test_630_middle_dot_full_width_in_registered_font:
  DotumChe `·` advance 가 전각 (=17.33) 으로 측정되어야 함, got 8.65
  정정 전: 반각 (≈8.67). is_halfwidth_punct 가 U+00B7 강제 반각 처리.

test_630_native_inline_tab_right_align:
  native RIGHT 인라인 탭이 적용되어야 함. B_x=405.00 expected=395.00 (diff=10.00px)
  LEFT fallback 시 B_x ≈ 405.00. seg_w=10.00.
```

```
$ cargo test --release --test issue_630
test test_630_aift_p4_toc_paren_alignment ... FAILED
  aift p4 목차 `(페이지 표기)` 시작 `(` 가 단일 그룹 (±1.0px) 안에 정렬되어야 함.
  lines=23 min_x=592.31 max_x=601.39 spread=9.08px (예상 ≤1.0)
  8.67px 이탈 = `·` 반각/전각 측정 차이 (Issue #630).
```

3 테스트 모두 본질 결함의 정량적 시그너처 명확히 측정. Stage 3 / Stage 4 정정 후 GREEN 전환 결정적 검증 가능.

## 3. 단위 테스트 설계 정합

| 테스트 | 측정 본질 | 정정 단계 | 회귀 가드 |
|--------|---------|----------|----------|
| `test_630_middle_dot_full_width_in_registered_font` | `·` 측정폭 = font_size | Stage 3 | 정정 1 본질 |
| `test_630_native_inline_tab_right_align` | RIGHT 인라인 탭 매치 | Stage 4 | 정정 2 본질 |
| `test_630_aift_p4_toc_paren_alignment` | 통합 — 결함 본질 (8.67px 이탈 0) | Stage 3+4 (둘 다 필요) | 두 정정 효과 통합 검증 |

`test_630_native_inline_tab_right_align` 은 미등록 폰트 (`UNREGISTERED_FONT`) 사용 — `·` 측정 변경 (Stage 3) 영향 받지 않음 → Stage 4 단독 효과 격리 측정.

## 4. 다음 단계 (Stage 3)

`src/renderer/layout/text_measurement.rs:859-862` 정정 — `is_halfwidth_punct` 에서 `'\u{00B7}'` 제거. `test_630_middle_dot_full_width_in_registered_font` GREEN 확인 + 통합 테스트 부분 GREEN / 잔존 영역 측정.

## 5. 산출물

- `tests/issue_630.rs` (새 파일, 통합 테스트 + SVG 파싱 헬퍼)
- `src/renderer/layout/text_measurement.rs` (`tests` mod 단위 테스트 2 건 추가)
- 본 단계별 보고서

## 6. 승인 요청

Stage 2 단위 테스트 작성 완료, 3 건 모두 RED 확인. Stage 3 (정정 1 적용 — `·` 측정 통일) 진행 승인 부탁드립니다.
