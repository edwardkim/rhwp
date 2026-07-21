# PR #????: HWPX charPr outline type / shadow type 매핑 소실 수정

## 이슈
- **Issue**: #2695 — HWPX charPr outline type 8값 중 4값·shadow type DROP 이 파싱/직렬화 양쪽에서 소실

## 분석

### 문제 1: outline type 8값 중 4값 소실

`<hh:outline type>`의 8개 열거값 중 `DASH_DOT(4)`, `DASH_DOT_DOT(5)`, `LONG_DASH(6)`, `CIRCLE(7)` 4값이 파서에서 `_ => 0`, 직렬화에서 `_ => "NONE"`으로 떨어져 외곽선이 완전히 사라짐.

IR 필드(`outline_type: u8`)는 HWP5 3비트(0~7)를 정확히 운반 중이었으나 HWPX 매핑만 누락.

### 문제 2: shadow type DROP/CONTINUOUS 붕괴

`<hh:shadow type>`에서 `DROP(1)`과 `CONTINUOUS(2)`가 파서에서 `"DROP" | "CONTINUOUS" => 1`로 합쳐지고, 직렬화는 `"DROP"`을 방출할 수 없어 HWP5→HWPX→HWP5 경로에서 비트값이 2→1로 변조됨 (L4 바이너리 손상).

## 변경

### 파서 (`src/parser/hwpx/header.rs`)
- outline_type: 8개 값 전체 매핑 (DASH_DOT/CIRCLE 등 4값 추가)
- shadow_type: DROP(1)과 CONTINUOUS(2) 분리

### 직렬화 (`src/serializer/hwpx/header.rs`)
- outline_type_str: 0~7 전체 매핑
- shadow_type: NONE/DROP/CONTINUOUS 3분기 match

### 테스트 (신규 3종)
- `outline_type_str_emits_all_eight_values` — 단위
- `shadow_type_emits_drop_and_continuous_separately` — 단위
- `outline_type_hwpx_roundtrip` — 왕복
- `shadow_type_drop_hwpx_roundtrip` — 왕복
- `shadow_type_continuous_hwpx_roundtrip` — 왕복

## 검증
- `cargo test --lib -- hwpx::header` — 71/71 통과 (기존 68 + 신규 3)
- `cargo fmt --all -- --check` — 통과
- `cargo clippy --all-targets -- -D warnings` — 통과
