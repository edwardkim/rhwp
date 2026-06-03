# Task m100 #944 Stage 0 조사 보고서

## 1. 질문

작업을 시작하기 전에 다음을 확인한다.

```text
rhwp에서 HWP 파일을 열고 IR을 구성한 뒤,
편집을 반영해서 다시 HWP로 저장하는 과정은 왜 성공하는가?
```

이 질문은 `hwpx -> IR -> hwp` 저장 문제를 다시 정의하기 위한 선행 조사다.

## 2. 결론

HWP 원본의 열기-편집-저장 경로가 성공하는 이유는 단순히 IR 재구성이 정확해서가 아니다.

핵심은 다음 두 가지다.

```text
1. HWP 파서는 원본 HWP record stream과 raw payload를 대량 보존한다.
2. HWP 저장기는 가능한 한 이 raw stream/payload를 그대로 재사용한다.
```

따라서 HWP 경로는 "IR만으로 HWP를 새로 만드는 경로"가 아니라,
원본 HWP의 record contract를 보존하면서 필요한 부분만 재직렬화하는 경로다.

반면 HWPX 경로는 HWP record stream 원본이 없다. 따라서 `hwpx -> hwp` 저장은
HWPX control을 HWP5 control/record로 1:1 매핑하는 별도 저장기 문제로 보아야 한다.

## 3. HWP 파싱 경로의 raw 보존

### FileHeader

`src/parser/mod.rs`의 `parse_hwp_with_cfb()`는 HWP FileHeader 원본 바이트를
`ModelFileHeader.raw_data`에 저장한다.

```text
src/parser/mod.rs
model_header.raw_data = Some(header_data)
```

### DocInfo

DocInfo는 파싱 후 원본 decompressed record stream 전체를 보존한다.

```text
src/parser/mod.rs
doc_info.raw_stream = Some(doc_info_data)
```

### BodyText section

각 BodyText section도 파싱 후 원본 decompressed record stream 전체를 보존한다.

```text
src/parser/mod.rs
section.raw_stream = Some(section_data)
```

이 값은 `Section.raw_stream`에 저장되며, 저장 시 빠른 경로로 사용된다.

## 4. HWP 저장 경로의 raw 재사용

### BodyText

`src/serializer/body_text.rs::serialize_section()`은 `section.raw_stream`이 있으면
모델을 다시 직렬화하지 않고 원본 스트림을 그대로 반환한다.

```rust
pub fn serialize_section(section: &Section) -> Vec<u8> {
    if let Some(ref raw) = section.raw_stream {
        return raw.clone();
    }
    ...
}
```

즉, 편집되지 않은 section은 HWP 원본 record stream이 그대로 유지된다.

### DocInfo

`src/serializer/doc_info.rs::serialize_doc_info()`는 `raw_stream_dirty == false`이고
`doc_info.raw_stream`이 있으면 원본 DocInfo stream을 그대로 반환한다.

```rust
if !doc_info.raw_stream_dirty {
    if let Some(ref raw) = doc_info.raw_stream {
        let mut result = raw.clone();
        ...
        return result;
    }
}
```

따라서 일반 텍스트 편집에서는 DocInfo 전체를 다시 만들지 않는다.

## 5. 편집 시 raw invalidation은 국소적이다

텍스트 삽입/삭제는 해당 section의 `raw_stream`만 무효화한다.

```text
src/document_core/commands/text_editing.rs
self.document.sections[section_idx].raw_stream = None;
```

반면 DocInfo는 보통 유지한다. 캐럿 위치처럼 꼭 필요한 변경은
`serializer::doc_info::surgical_update_caret()`로 raw_stream 내부를 국소 수정한다.

```text
DocInfo raw_stream 전체 재생성 대신 caret 3필드만 in-place 수정
```

이 구조 때문에 HWP 편집 저장은 다음처럼 동작한다.

```text
편집 안 된 section:
  원본 BodyText raw_stream 그대로 저장

편집된 section:
  section.raw_stream=None 이므로 모델에서 재직렬화

DocInfo:
  대부분 원본 raw_stream 보존
  필요한 작은 값만 surgical update
```

## 6. 편집된 section도 raw payload가 많이 남아 있다

편집된 section은 `raw_stream`을 잃지만, HWP parser가 개별 model 필드에 원본 payload를
복사해 두기 때문에 재직렬화 손실이 줄어든다.

대표 예:

### Paragraph

`src/parser/body_text.rs`는 `PARA_HEADER`의 12바이트 이후 추가 데이터를
`Paragraph.raw_header_extra`에 보존한다.

```text
para.raw_header_extra = data[12..].to_vec()
```

### Table CTRL_HEADER

`src/parser/control.rs::parse_table_control()`은 table `CTRL_HEADER` payload를
`table.raw_ctrl_data`에 그대로 저장한다.

```text
table.raw_ctrl_data = ctrl_data.to_vec()
```

`src/serializer/control.rs::serialize_table()`은 이 값을 그대로 다시 쓴다.

```text
if !table.raw_ctrl_data.is_empty() {
    &table.raw_ctrl_data
}
```

### TABLE record

`parse_table_record()`는 TABLE record의 attr와 tail을 보존한다.

```text
table.raw_table_record_attr = attr
table.raw_table_record_extra = remaining bytes
```

직렬화 시 `raw_table_record_attr`와 `raw_table_record_extra`가 다시 쓰인다.

### Cell LIST_HEADER

`parse_cell()`은 LIST_HEADER의 추가 데이터를 `cell.raw_list_extra`에 보존한다.

```text
cell.raw_list_extra = remaining bytes
```

직렬화 시 이 tail이 다시 붙는다.

### DocInfo item

`src/serializer/doc_info.rs`는 `BinData`, `BorderFill`, `CharShape`, `ParaShape` 등에서
`raw_data`가 있으면 모델 필드에서 재생성하지 않고 원본 raw_data를 우선 사용한다.

```text
bin_data.raw_data.unwrap_or_else(serialize_bin_data)
char_shape.raw_data.unwrap_or_else(serialize_char_shape)
para_shape.raw_data.unwrap_or_else(serialize_para_shape)
...
```

## 7. HWPX 경로는 구조적으로 다르다

HWPX 파서는 XML/ZIP을 읽어 IR을 구성한다.

```text
src/parser/hwpx/mod.rs
src/parser/hwpx/section.rs
src/parser/hwpx/header.rs
```

하지만 HWPX에는 HWP5 BodyText record stream 원본이 없다.

따라서 HWPX에서 만들어진 `Document`는 HWP 경로와 달리 다음이 없다.

```text
FileHeader.raw_data
DocInfo.raw_stream
Section.raw_stream
table.raw_ctrl_data 원본 HWP payload
HWP TABLE/LIST_HEADER/PARA_HEADER tail의 원본 record contract
```

그래서 HWPX 저장 경로는 저장 직전에 adapter를 호출한다.

```text
src/document_core/commands/document.rs
export_hwp_with_adapter()
  -> convert_if_hwpx_source()
  -> convert_hwpx_to_hwp_ir()
  -> export_hwp_native()
```

`convert_if_hwpx_source()`는 HWP 출처에서는 no-op이고, HWPX/HWP3 출처에서만 동작한다.

```rust
if !matches!(source_format, FileFormat::Hwpx | FileFormat::Hwp3) {
    return AdapterReport::new().no_op("source_format != Hwpx/Hwp3");
}
```

## 8. 왜 HWP는 성공하고 HWPX->HWP는 실패하는가

HWP 저장 성공의 본질:

```text
원본 HWP record contract를 보존한다.
편집된 일부 section만 재직렬화한다.
재직렬화하더라도 개별 raw payload를 최대한 재사용한다.
```

HWPX->HWP 실패의 본질:

```text
HWPX에는 HWP5 record contract 원본이 없다.
현재 adapter는 HWPX의 semantic IR을 HWP 저장기가 기대하는 일부 raw payload로 보정한다.
그러나 이것은 HWPX control -> HWP5 control/record의 1:1 매핑 설계가 아니다.
```

따라서 #944의 근본 해결 방향은 다음이다.

```text
HWPX를 렌더링 IR로만 보고 HWP payload를 보정하지 않는다.
한컴 정답 HWP를 oracle로 삼아,
HWPX control이 HWP5 control/record로 어떻게 변환되는지 매핑표를 먼저 만든다.
그 다음 저장기 구현을 한다.
```

## 9. 다음 조사 항목

다음 Stage에서는 `hwpx-h-03`을 대상으로 세 개의 inventory를 나란히 만든다.

```text
1. HWPX 원본 control inventory
2. 한컴 변환 정답 HWP의 HWP5 record/control inventory
3. rhwp 저장 HWP의 HWP5 record/control inventory
```

비교 대상:

```text
section / paragraph / run
CTRL_HEADER
CTRL_DATA
LIST_HEADER
TABLE
SHAPE_COMPONENT
SHAPE_PICTURE
PARA_HEADER tail
PARA_TEXT control code
BinData 참조
DocInfo 참조 ID
```

이 비교로 누락/변형/추정 금지 항목을 먼저 확정한 뒤 구현한다.
