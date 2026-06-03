# Task M100-1028 Stage 1 분석 보고서

## 1. 본질 진단

### A. HWPX XML 결정적 라인 (`samples/hwpx/tbox-v-flow-01.hwpx`)

```xml
<hp:drawText lastWidth="18664" name="" editable="0">
  <hp:subList id="" textDirection="VERTICALALL" lineWrap="BREAK" vertAlign="CENTER" ...>
    <hp:p ...>...</hp:p>
  </hp:subList>
</hp:drawText>
```

**`<hp:subList>` 의 `textDirection="VERTICALALL"` 속성**이 글상자 세로쓰기 플래그.

### B. 기존 cell HWPX 파싱 (`src/parser/hwpx/section.rs:1108`)

```rust
b"textDirection" => {
    let val = attr_str(&attr);
    cell.text_direction = if val == "VERTICAL" { 1 } else { 0 };
}
```

Cell 은 `"VERTICAL"` 처리. 글상자는 `"VERTICALALL"` 사용 — 처리 누락.

### C. 현 `parse_draw_text` (`section.rs:2249-2266`)

```rust
b"subList" => {
    for attr in ce.attributes().flatten() {
        match attr.key.as_ref() {
            b"vertAlign" => {
                // vertical_align 만 처리
                text_box.list_attr =
                    (text_box.list_attr & !(0b11 << 5)) | (align_code << 5);
            }
            _ => {}  // ⚠️ textDirection 무시
        }
    }
}
```

`subList` attribute 루프가 `vertAlign` 만 처리 + `list_attr` bit 5~6 (vertical_align) 만 set. **`textDirection` 무시** → renderer 의 `shape_layout.rs:1652` `(text_box.list_attr & 0x07)` 가 항상 0 → 세로쓰기 미발동.

### D. Renderer 경로 (변경 불필요 확인)

`src/renderer/layout/shape_layout.rs:1649-1652`:

```rust
// 세로쓰기 판정: 글상자 list_attr bit 0~2 = text_direction
// (0=가로, 1=영문 눕힘, 2=영문 세움)
let text_direction = (text_box.list_attr & 0x07) as u8;
```

**Renderer 는 이미 세로쓰기 분기 (`layout_vertical_textbox_text_with_paras`) 보유** — `text_direction != 0` 시 발동. HWP5 와 동일 경로 사용.

## 2. Root cause 정리

**단일 파서 누락**: HWPX `parse_draw_text` 가 `<hp:subList>` 의 `textDirection` 속성을 파싱하여 `text_box.list_attr` bit 0~2 (text_direction) 에 set 하는 처리 누락.

| 구성요소 | 상태 |
|---------|------|
| HWPX XML 속성 | `textDirection="VERTICALALL"` 존재 |
| HWPX parser (`parse_draw_text` subList) | **`textDirection` 미파싱** ⚠️ |
| HWP5 parser (`parser/control/shape.rs:175`) | `list_attr` 보존 (bit 0~2 포함) ✓ |
| Document IR (`text_box.list_attr` u32) | bit 0~2 = text_direction 자리 (미사용 상태) |
| Renderer (`shape_layout.rs:1652`) | `(list_attr & 0x07)` 로 분기 ✓ |

## 3. HWP5 vs HWPX 비트 매핑 차이 (주의)

| 컨텍스트 | text_direction 비트 위치 |
|---------|--------------------------|
| Cell LIST_HEADER (`control.rs:331`) | bit **16~18** (`>> 16 & 0x07`) |
| TextBox LIST_HEADER (`shape_layout.rs:1652`) | bit **0~2** (`& 0x07`) |

HWP5 spec 표 67 에 따라 같은 LIST_HEADER 구조이나 cell/textbox 비트 위치가 다름 (소스 주석에도 명시: "주의: 테이블 셀은 bit 16~18이지만 글상자 LIST_HEADER는 bit 0~2"). 본 task 는 textbox 만 다루므로 bit 0~2 에 set.

## 4. textDirection 값 매핑 추정

| 값 | 의미 | bit 0~2 코드 (추정) |
|----|------|---------------------|
| `HORIZONTAL` (또는 미지정) | 가로 | 0 |
| `VERTICAL` | 세로 (cell 기존 매핑) | 1 |
| `VERTICALALL` (글상자 사용) | 세로 (영문도 세움) | 1 또는 2 — 검증 필요 |

`shape_layout.rs:1650` 주석: "0=가로, 1=영문 눕힘, 2=영문 세움". `VERTICALALL` = "영문 세움" (`= 2`) 가능성. Stage 2 시 한컴 PDF (`pdf/hwpx/tbox-v-flow-01-2022.pdf`) 와 HWP5 변환본 (`samples/hwpx/hancom-hwp/tbox-v-flow-01.hwp`) 의 list_attr 값을 직접 dump 하여 확정.

## 5. 다른 textDirection 처리 위치 동시 점검

`section.rs` 의 textDirection 처리 위치 3 곳:
- `:75` `sec_def.text_direction` (섹션 단위, `VERTICAL` 만)
- `:1108` cell `tcPr` (`VERTICAL` 만)
- `:1139` cell legacy format (`VERTICAL` 만)

→ **모두 `VERTICAL` 만 매칭**. `VERTICALALL` 미처리.

Stage 2 시 세 곳도 함께 `VERTICALALL` 매칭 추가 권고 (또는 별도 task 분리).

## 6. 작업 권고 (Stage 2 구현 계획서 기초)

### 최소 변경 (본 task 범위)

`parse_draw_text` subList 루프에 1 분기 추가:

```rust
b"textDirection" => {
    let val = attr_str(&attr);
    let direction_code = match val.as_str() {
        "VERTICAL" | "VERTICALALL" => 1u32,
        _ => 0,
    };
    text_box.list_attr =
        (text_box.list_attr & !0b111) | direction_code;
}
```

### 확장 권고 (별도 분리 가능)

- cell `tcPr` / cell legacy / sec_def 의 `textDirection` 매칭에 `VERTICALALL` 추가
- "영문 눕힘 vs 영문 세움" 차이 명확화 (Stage 2 시 실측 후 결정)

## 7. 다음 단계

Stage 2 구현 계획서로 진행. 작업지시자 승인 후 실행.
