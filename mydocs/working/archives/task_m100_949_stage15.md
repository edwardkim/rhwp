# Task M100-949 Stage 15 Working Notes

## 1. 구현 요약

Stage 14에서 확인한 누락 `CTRL_DATA`는 그림 바이너리 자체가 아니라 HWPX `hp:pic@href`
값을 저장하는 HWP5 ParameterSet 계약이었다.

이번 단계에서는 이 값을 다음 경로로 연결했다.

```text
HWPX parser
  hp:pic@href
    -> Picture.href

HWPX serializer
  Picture.href
    -> hp:pic@href

HWPX -> HWP adapter
  Picture.href
    -> Paragraph.ctrl_data_records[control_index]
    -> HWP CTRL_DATA record
```

## 2. 변경 파일

```text
src/model/image.rs
src/parser/hwpx/section.rs
src/serializer/hwpx/picture.rs
src/serializer/control.rs
src/document_core/converters/hwpx_to_hwp.rs
src/wasm_api/tests.rs
```

## 3. CTRL_DATA payload

`samples/hwpx/hancom-hwp/hwpx-h-03.hwp` 정답지의 `CTRL_DATA#833`와 같은 구조를 생성한다.

```text
ParameterSet ps_id=0x021b count=1 dummy=0x0000
  item#0 id=0x026f type=0x8000(ParameterSet)
    ParameterSet ps_id=0x026f count=1 dummy=0x0000
      item#0 id=0x0265 type=0x0001(String)
        string len=27 value="http\\://www.korea.kr;1;0;0;"
```

바이트 길이:

```text
22 byte header/item structure + UTF-16LE 27 code units = 76 bytes
```

## 4. 재귀 처리

`hwpx-h-03`의 해당 그림은 top-level paragraph가 아니라 다음 중첩 위치에 있었다.

```text
SHAPE_COMPONENT(rect)
  -> drawText/textBox
    -> paragraph
      -> picture control
```

따라서 adapter의 `adapt_paragraph`는 다음을 처리하도록 확장했다.

```text
Control::Table    -> 기존 표 보강 + 셀 문단 재귀
Control::Picture  -> Picture.href를 CTRL_DATA로 materialize
Control::Shape    -> shape text_box paragraph 재귀
```

## 5. 검증

실행한 검사:

```bash
cargo check --quiet
cargo test --quiet picture_href_ctrl_data
cargo test --quiet hwpx_h_03_href_ctrl_data_from_source_contract
```

결과:

```text
cargo check: pass
picture_href_ctrl_data: 3 passed
hwpx_h_03_href_ctrl_data_from_source_contract: 1 passed
```

테스트가 확인하는 내용:

```text
1. 한컴 정답지에서 확인한 76바이트 ParameterSet 구조와 동일한 payload 생성
2. picture control index와 ctrl_data_records index의 1:1 대응
3. shape text box 내부 paragraph의 picture control에도 payload materialize
4. samples/hwpx/hwpx-h-03.hwpx 파싱 직후 CTRL_DATA 0개
5. adapter 적용 후 Picture.href 기반 CTRL_DATA 1개
```

## 6. 남은 판정

이번 단계는 `hp:pic@href -> CTRL_DATA` 계약을 코드에 반영한 것이다.
판정용 HWP 산출물은 다음 경로에 생성했다.

```text
output/poc/hwpx2hwp/task949/stage15/hwpx-h-03/hwpx-h-03-stage15.hwp
```

## 7. 정답지 대비 CTRL_DATA 재검증

처음 산출 시 `CTRL_DATA` payload 자체는 정답지와 같았지만, 공통 control serializer가
`CTRL_HEADER` 바로 뒤에 한 번 더 넣어 `CTRL_DATA`가 2개가 되는 문제가 확인되었다.

원인:

```text
serialize_control()
  -> 모든 ctrl_data_records를 CTRL_HEADER 직후에 삽입

serialize_picture_control()
  -> 같은 ctrl_data_records를 SHAPE_COMPONENT 자식으로 삽입
```

한컴 정답지의 picture href `CTRL_DATA` 위치는 `SHAPE_COMPONENT` 자식이다. 따라서
`Picture`/`Shape` control은 공통 삽입 경로에서 제외하고, 각 shape serializer가 책임지도록
정리했다.

재실행 결과:

```text
output/poc/hwpx2hwp/task949/stage15/hwpx-h-03/ctrl_data_trace.md
```

요약:

```text
oracle    CTRL_DATA records = 1, total bytes = 76, hash = 024e873ad9c2bd92
generated CTRL_DATA records = 1, total bytes = 76, hash = 024e873ad9c2bd92
record_index = 833, level = 6, parent = SHAPE_COMPONENT#832@lv5
```

즉 이번 단계에서 `hp:pic@href -> HWP CTRL_DATA` 계약은 정답지와 record 위치, level, payload가
일치한다.
