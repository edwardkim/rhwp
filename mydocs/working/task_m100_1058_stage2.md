# Task #1058 Stage 2 보고서 — TextBox LIST_HEADER serializer 정정

- 이슈: [#1058](https://github.com/edwardkim/rhwp/issues/1058)
- 단계: Stage 2 (구현)
- 일시: 2026-05-21

## 1. 결과 요약

`src/serializer/control.rs::serialize_text_box_if_present` 에 한컴 default 13 byte (`hwplib::ForTextBox::listHeader` 정합) 추가. HWP 정답지 vs rhwp 저장본 TextBox LIST_HEADER **raw byte 완전 동일** (size=33 정합).

## 2. 변경 사항

### 2.1 `src/serializer/control.rs` (+10 라인)

```rust
fn serialize_text_box_if_present(drawing: &DrawingObjAttr, level: u16, records: &mut Vec<Record>) {
    if let Some(ref text_box) = drawing.text_box {
        let mut w = ByteWriter::new();
        w.write_u32(text_box.paragraphs.len() as u32).unwrap();
        w.write_u32(text_box.list_attr).unwrap();
        w.write_i16(text_box.margin_left).unwrap();
        w.write_i16(text_box.margin_right).unwrap();
        w.write_i16(text_box.margin_top).unwrap();
        w.write_i16(text_box.margin_bottom).unwrap();
        w.write_u32(text_box.max_width).unwrap();
        // [Task #1058] hwplib::ForTextBox::listHeader 정합:
        //   sw.writeZero(8);                 // 8 byte zero padding
        //   sw.writeSInt4(editableAtFormMode); // 4 byte (0 = false)
        //   sw.writeUInt1(fieldNameFlag);     // 1 byte (0 = no fieldName)
        // 한컴은 이 contract 가 누락되면 글상자 안 paragraph 를 본문 list 로 인식하여
        // 신규 paragraph (각주) 추가 시 다단계 목록 번호 "1.1.1.1.1.1" 자동 부여.
        if !text_box.raw_list_header_extra.is_empty() {
            w.write_bytes(&text_box.raw_list_header_extra).unwrap();
        } else {
            // HWPX 출처: 한컴 default 13 byte
            w.write_bytes(&[0u8; 13]).unwrap();
        }
        records.push(Record { ... });
        serialize_paragraph_list(&text_box.paragraphs, level, records);
    }
}
```

## 3. 정량 입증 — raw byte 정합

### 3.1 hwp5-inventory-diff LIST_HEADER 영역

```bash
./target/release/rhwp hwp5-inventory-diff samples/footnote-tbox-01.hwp \
    output/poc/issue_1058/footnote-tbox-01-stage2.hwp \
    | grep -iE "LIST_HEADER.*size|LIST_HEADER.*payload"
```

→ **LIST_HEADER 차이 0건** (size=33 정합).

### 3.2 Raw byte 비교

```
ORACLE: 02000000 20000000 1b011b011b011b01 ea300000 0000000000000000 00000000 00
rhwp:   02000000 20000000 1b011b011b011b01 ea300000 0000000000000000 00000000 00
                                                    └─ zero(8) ─┘  └editable┘ flag
```

→ **33 byte byte-by-byte 완전 동일**.

## 4. 검증

| 항목 | 결과 |
|------|------|
| cargo build --lib | OK |
| cargo build --release --bin rhwp | OK |
| convert HWPX → HWP | OK (14 KB 정답지 동등 크기) |
| TextBox LIST_HEADER raw 정합 | byte-by-byte 동일 |

## 5. 다음 단계

Stage 3 — 회귀 가드 + 광범위 sweep + WASM 빌드.
