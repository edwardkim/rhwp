# Task #588 Stage 2 — Red 테스트 + 매핑 구현

## 목표

1. PUA 매핑 단위 테스트 추가 (Red → Green 전환)
2. `map_pua_bullet_char` 에 SPUA-A 저영역 분기 신설 + U+F003B → ↓ 매핑

## 변경 영역

### 1. `src/renderer/layout/paragraph_layout.rs`

#### 1.1 `map_pua_bullet_char` SPUA-A 저영역 분기 신설

**위치**: 기존 `0xF02B0..=0xF02FF` 분기 직전 (코드포인트 오름차순 정합)

```rust
// Supplementary PUA-A 저영역 — 한컴 자체 영역 (Task #588 한컴 정답지 정합)
if (0xF0000..=0xF00CF).contains(&code) {
    return match code {
        // exam_eng.hwp p7 #40 요약형 문항 글상자 사이 화살표.
        // 한컴 PDF (HCRBatang 임베디드 폰트) 글리프 외곽 분석:
        //   stem 35% × arrowhead 100% × solid filled (1 contour, 7 pts) → ↓
        0xF003B => '\u{2193}', // ↓ DOWNWARDS ARROW
        _ => ch,
    };
}
```

#### 1.2 docstring 갱신

기존 docstring 의 "두 영역 분기" → 세 영역 분기 (Basic / SPUA-A 저영역 / SPUA-A 일반) 으로 정합 갱신.

#### 1.3 단위 테스트 +2

```rust
#[test]
fn supplementary_pua_a_low_range_maps_down_arrow() {
    // [Task #588] U+F003B → U+2193 ↓ (DOWNWARDS ARROW)
    assert_eq!(map_pua_bullet_char('\u{F003B}'), '\u{2193}');
}

#[test]
fn supplementary_pua_a_low_range_unmapped_returns_original() {
    // [Task #588] 0xF0000~0xF00CF 영역의 매핑 표 외 코드포인트는 원본 유지
    assert_eq!(map_pua_bullet_char('\u{F0090}'), '\u{F0090}');
    assert_eq!(map_pua_bullet_char('\u{F0000}'), '\u{F0000}');
    assert_eq!(map_pua_bullet_char('\u{F00CF}'), '\u{F00CF}');
}
```

## 검증

### 단위 테스트 (`pua_mapping_tests`)

```
running 7 tests
test ...basic_pua_outside_range_returns_original ... ok
test ...supplementary_pua_a_low_range_maps_down_arrow ... ok            ← 신규
test ...supplementary_pua_a_maps_circled_digits ... ok
test ...supplementary_pua_a_low_range_unmapped_returns_original ... ok  ← 신규
test ...basic_pua_arrow_e8 ... ok
test ...supplementary_pua_a_maps_middle_dot ... ok
test ...supplementary_pua_a_unmapped_returns_original ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1122 filtered out
```

기존 5 + 신규 2 = 7건 GREEN. 회귀 0.

### SVG 시각 확인 (`exam_eng.hwp` p7)

**변경 전** (`output/svg/exam_eng_p7/exam_eng_007.svg:4162`):
```
<text ... font-family="HY신명조,..." font-size="15.33...">󰀻</text>
```
(U+F003B 그대로 출력 → 폰트 글리프 미보유 시 두부)

**변경 후** (`output/svg/exam_eng_p7_after/exam_eng_007.svg:4162`):
```
<text ... font-family="HY신명조,..." font-size="15.33...">↓</text>
```
(U+2193 표준 코드포인트 → 모든 폰트에서 글리프 보유)

### 회귀 차단

- 신설 분기 (`0xF0000..=0xF00CF`) 와 기존 분기 (`0xF020..=0xF0FF`, `0xF00D0..=0xF09FF`, `0xF02B0..=0xF02FF`) 디스조인트 → 기존 매핑 영역 무영향
- `_ => ch` default 분기 → U+F003B 외 같은 영역 코드포인트 (예: U+F0090) 는 원본 유지 (회귀 0)

## 산출물

- `src/renderer/layout/paragraph_layout.rs` — 분기 신설 + 단위 테스트 +2
- `mydocs/working/task_m100_588_stage2.md` — 본 보고서

## 다음 단계

Stage 3 (광범위 회귀 점검) 진행 승인 요청:
- `cargo test --lib` (baseline 회귀 0)
- `cargo test --test svg_snapshot` 6/6
- `cargo clippy --lib -- -D warnings` 0건 신규
- WASM 빌드 정합
- 14 fixture 광범위 byte sweep (의도된 변경 외 회귀 0)

## 메모리 룰 정합

- `feedback_hancom_compat_specific_over_general` — 단일 코드포인트 한정 매핑
- `feedback_essential_fix_regression_risk` — 분기 디스조인트 설계로 회귀 위험 차단
