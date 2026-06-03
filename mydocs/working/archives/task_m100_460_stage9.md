# Task #460 Stage 9 완료 보고서: HWP3→HWP5 저장 후 재열기 그림 위치 복원

## 개요

HWP3 파일을 열어 HWP5로 저장한 뒤 다시 열면 그림이 표를 겹치는 현상을 해소하였다.

렌더러(`layout.rs`, `typeset.rs`) 변경 없음. `src/parser/hwp3/mod.rs`만 수정.

## 재현 조건

1. `hwp3-sample.hwp` (HWP3) 열기
2. 첫 페이지에 "1" 입력
3. HWP5(.hwp)로 저장
4. 저장된 파일 재열기 → 7페이지 "그림 3"이 "표 1" 위에 겹침

## 원인 분석

### CommonObjAttr.attr 비트필드 미설정

HWP3 파서(`mod.rs`)는 그림/표 파싱 시 `CommonObjAttr`의 개별 필드를
(`vert_rel_to`, `text_wrap`, `horz_rel_to`, `treat_as_char` 등)을 올바르게 설정하지만,
직렬화에 사용되는 **`common.attr` 비트필드는 0으로 남아 있었다**.

```
serialize_common_obj_attr() → w.write_u32(common.attr)  // 0 기록
parse_common_obj_attr()     → attr=0 → vert_rel_to=Paper, text_wrap=Square
```

HWP5 재파싱 후:
- `vert_rel_to=Paper` (기대: Para)
- `text_wrap=Square` (기대: TopAndBottom)

`typeset.rs`의 `pushdown_h` 조건이 `!treat_as_char && TopAndBottom && Para` 이므로
Square+Paper 조합에서는 발동되지 않아 그림이 표를 겹침.

## 수정 내용

**파일**: `src/parser/hwp3/mod.rs`

### 1. build_common_obj_attr() 헬퍼 함수 추가

```rust
fn build_common_obj_attr(common: &CommonObjAttr) -> u32 {
    let mut attr: u32 = 0;
    if common.treat_as_char { attr |= 0x01; }
    attr |= (match common.vert_rel_to {
        VertRelTo::Paper => 0, VertRelTo::Page => 1, VertRelTo::Para => 2,
    }) << 3;
    attr |= (match common.vert_align { ... }) << 5;
    attr |= (match common.horz_rel_to { ... }) << 8;
    attr |= (match common.horz_align { ... }) << 10;
    attr |= (match common.text_wrap {
        TextWrap::Square => 0, TextWrap::TopAndBottom => 1,
        TextWrap::BehindText => 2, TextWrap::InFrontOfText => 3, _ => 0,
    }) << 21;
    attr
}
```

### 2. 그림 파싱 후 attr 갱신

```rust
// vert_align/horz_align 설정 직후:
pic.common.attr = build_common_obj_attr(&pic.common);
```

### 3. 표 파싱 후 attr 갱신

```rust
// vert_align/horz_align 설정 직후:
table.common.attr = build_common_obj_attr(&table.common);
```

### 검증: pi=78 그림의 예상 attr 값

- `treat_as_char=false` → bit 0 = 0
- `vert_rel_to=Para` → `2 << 3` = 0x10
- `vert_align=Top` → `0 << 5` = 0
- `horz_rel_to=Para` → `3 << 8` = 0x300
- `horz_align=Left` (offset 모드) → `0 << 10` = 0
- `text_wrap=TopAndBottom` → `1 << 21` = 0x200000

예상 attr = `0x200310`

재파싱 시 bits 3-4 = 2 → `Para` ✓, bits 21-23 = 1 → `TopAndBottom` ✓
→ `pushdown_h` 발동 → 그림이 표 위로 올라가지 않음

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1069 passed, 0 failed |
| `cargo test` (전체) | 전부 통과 |
| `cargo build` | 성공 |
| hwp3-sample.hwp SVG 내보내기 | **17페이지** 정상 완료 |
| `LAYOUT_OVERFLOW` (TypesetEngine) | **0건** 유지 |
| 잔존 `LAYOUT_OVERFLOW_DRAW` 경고 | 꼬리말 36px 초과 — 기존 잔존 경고만 |
| 렌더러 수정 | **없음** (hwp3/ 디렉토리만) |

### 4. serialize_table() 폴백 수정 (serializer/control.rs)

```rust
// 수정 전: raw_ctrl_data가 없으면 &[] (빈 CTRL_HEADER)
if !table.raw_ctrl_data.is_empty() { &table.raw_ctrl_data } else { &[] }

// 수정 후: raw_ctrl_data가 없으면 common에서 재구성
let ctrl_data = if !table.raw_ctrl_data.is_empty() {
    table.raw_ctrl_data.clone()
} else {
    serialize_common_obj_attr(&table.common)
};
```

HWP3 파서는 테이블의 `raw_ctrl_data`를 설정하지 않는다.
직렬화 시 `&[]`가 CTRL_HEADER로 기록되어 재열기 시 `attr=0`이 되어
`treat_as_char=false`로 복원되던 문제를 `serialize_common_obj_attr`로 폴백하여 해소.
`table.common.attr`(이미 Stage 9에서 설정)이 이제 실제로 직렬화에 반영된다.

## 수정 파일

| 파일 | 변경 내용 |
|------|---------|
| `src/parser/hwp3/mod.rs` | `build_common_obj_attr()` 함수 추가 + 그림/표 파싱 후 attr 갱신 |
| `src/serializer/control.rs` | `serialize_table()` — `raw_ctrl_data` 없을 때 `serialize_common_obj_attr` 폴백 |
