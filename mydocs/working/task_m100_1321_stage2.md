# Task M100 #1321 — Stage 2 완료보고서

## 개요

| 항목 | 내용 |
|------|------|
| 이슈 | #1321 — 빈 문단(text == "") 0-length field fieldBegin/fieldEnd 순서 역전 수정 |
| 단계 | Stage 2: 단위 테스트 추가 |
| 브랜치 | `local/task1321` |
| 커밋 | `7127a7c9` (Stage 1과 동일 커밋) |

## 수행 내용

`src/serializer/hwpx/section.rs` — `mod tests` 하단에 다음 테스트를 추가했다:

```rust
#[test]
fn task1321_zero_length_field_in_empty_paragraph() {
    // para.text = ""  (빈 문단)
    // field_ranges = [{ start: 0, end: 0, control_idx: 0 }]  (0-length field)
    // 기대: fieldBegin이 fieldEnd보다 앞서 출현해야 한다.
    let mut f = Field::default();
    f.field_type = FieldType::ClickHere;
    f.field_id = 99;
    let mut para = Paragraph::default();
    para.text = "".to_string();
    para.char_count = 17;
    para.char_offsets = vec![];
    para.controls.push(Control::Field(f));
    para.field_ranges.push(FieldRange {
        start_char_idx: 0,
        end_char_idx: 0,
        control_idx: 0,
    });
    let xml = serialize_para_for_test(&para);
    let begin_pos = xml.find("fieldBegin").expect("fieldBegin 없음");
    let end_pos = xml.find("fieldEnd").expect("fieldEnd 없음");
    assert!(
        begin_pos < end_pos,
        "fieldBegin({begin_pos})이 fieldEnd({end_pos})보다 뒤에 위치: {xml}"
    );
}
```

## 테스트 실행 결과

```
cargo test --lib -- section::tests
```

```
running 51 tests
...
test serializer::hwpx::section::tests::task1321_zero_length_field_in_empty_paragraph ... ok
...
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- 신규 테스트 포함 51개 전체 통과

## 상태

완료
