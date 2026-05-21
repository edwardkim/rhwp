# Task #1058 Stage 1 보고서 — TextBox LIST_HEADER 13 byte contract 완전 규명

- 이슈: [#1058](https://github.com/edwardkim/rhwp/issues/1058)
- 단계: Stage 1 (의미 완전 규명)
- 일시: 2026-05-21

## 1. 결과 요약

`hwplib` (`/home/edward/vsworks/shwp/hwplib/src/main/java/kr/dogfoot/hwplib/writer/bodytext/paragraph/control/gso/part/ForTextBox.java`) 의 `ForTextBox::listHeader` 권위 자료로 글상자 LIST_HEADER 의 정확한 33 byte 구조 완전 규명.

## 2. TextBox LIST_HEADER contract (총 33 byte)

```java
sw.writeSInt4(lh.getParaCount());                       // offset 0..3
sw.writeUInt4(lh.getProperty().getValue());             // offset 4..7
sw.writeUInt2(lh.getLeftMargin());                      // offset 8..9
sw.writeUInt2(lh.getRightMargin());                     // offset 10..11
sw.writeUInt2(lh.getTopMargin());                       // offset 12..13
sw.writeUInt2(lh.getBottomMargin());                    // offset 14..15
sw.writeUInt4(lh.getTextWidth());                       // offset 16..19
sw.writeZero(8);                                        // offset 20..27 (zero padding)
sw.writeSInt4(editableAtFormMode ? 1 : 0);              // offset 28..31
sw.writeUInt1(fieldNameFlag);                           // offset 32 (0x00 or 0xff)
// if fieldNameFlag == 0xff: ParameterSet 추가 (fieldName)
```

## 3. rhwp 이전 vs 정답지 비교

| Offset | Field | rhwp 이전 | 정답지 | 의미 |
|--------|-------|----------|--------|------|
| 0-3 | paraCount | ✓ | ✓ | 글상자 안 paragraph 수 |
| 4-7 | property | ✓ | ✓ | ListHeaderProperty (TextDirection 등) |
| 8-15 | margin × 4 | ✓ | ✓ | L/R/T/B (UInt2) |
| 16-19 | textWidth | ✓ | ✓ | 최대 폭 |
| **20-27** | **zero padding** | ❌ 누락 | 8 byte zero | hwplib `writeZero(8)` |
| **28-31** | **editableAtFormMode** | ❌ 누락 | 4 byte (0) | 양식 모드 편집 가능 |
| **32** | **fieldName flag** | ❌ 누락 | 1 byte (0x00) | 필드 이름 ParameterSet 존재 여부 |

→ **rhwp 누락 13 byte** = `zero(8) + editable(4) + flag(1)`

## 4. 한컴 정답지 raw byte (`samples/footnote-tbox-01.hwp` rec#18 LIST_HEADER)

```
02 00 00 00          paraCount = 2
20 00 00 00          property = 0x20 (TextDirection=Horizontal + LineWrap=BREAK 등 bit flags)
1b 01 1b 01 1b 01 1b 01    L=283 R=283 T=283 B=283 (UInt2 × 4)
ea 30 00 00          textWidth = 0x30ea = 12522
00 00 00 00 00 00 00 00    zero padding (8 byte)
00 00 00 00          editableAtFormMode = 0
00                   fieldName flag = 0 (no fieldName)
```

→ size = 4+4+8+4+8+4+1 = **33 byte** (정답지 정합)

## 5. 한컴편집기 결함 발생 메커니즘 가설

글상자 LIST_HEADER 의 contract 가 짧으면 한컴이 글상자 내부 paragraph 를 **본문 list 의 일부**로 잘못 해석. 결과:
- 글상자 안 신규 paragraph 추가 시 본문 다단계 목록 number 자동 부여 (예: "1.1.1.1.1.1.")
- 한컴편집기의 다단계 목록 처리 로직이 글상자 안 paragraph 를 별도 scope 으로 처리하지 못함

## 6. 정정 방향 (Stage 2)

### 6.1 `src/serializer/control.rs::serialize_text_box_if_present`

이미 `raw_list_header_extra` 보존 영역 존재 → **비어있을 때만 default 13 byte zero 적용**:

```rust
if !text_box.raw_list_header_extra.is_empty() {
    w.write_bytes(&text_box.raw_list_header_extra).unwrap();
} else {
    // [Task #1058] HWPX 출처: 한컴 default 13 byte (zero 8 + editable 0 + flag 0)
    w.write_bytes(&[0u8; 13]).unwrap();
}
```

### 6.2 후방 호환

- HWP 출처: 기존 `raw_list_header_extra` 보존 그대로 (회귀 부재)
- HWPX 출처: 새 default 13 byte 적용 → 한컴 정합
- editableAtFormMode = 0 + fieldName 없음 = 일반 글상자 default

## 7. 다음 단계

Stage 2 — serializer 정정 + 정량 입증 (hwp5-inventory-diff LIST_HEADER 차이 0건).
