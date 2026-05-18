# Task m100-949 Stage 25: GenShape CTRL_HEADER 계약 디코드

## 1. 목적

Stage 24에서 `hwpx-h-03` 파일손상 후보로 남은 drawText/image group의 GenShape
`CTRL_HEADER` 차이를 필드 단위로 해석한다.

대상 record:

```text
#824 outer rect CTRL_HEADER
#831 first child picture CTRL_HEADER
#835 second child picture CTRL_HEADER
```

## 2. 산출물

```text
output/poc/hwpx2hwp/task949/stage25_genshape_ctrl_header_decode/h03_ctrl_header_field_diff.md
output/poc/hwpx2hwp/task949/stage25_genshape_ctrl_header_decode/h03_source_to_hwp5_mapping.md
output/poc/hwpx2hwp/task949/stage25_genshape_ctrl_header_decode/guard_applicability.md
```

## 3. 핵심 필드 차이

| record | oracle attr | generated attr | diff | 추가 차이 |
|---:|---:|---:|---:|---|
| #824 | `0x042a4311` | `0x002a0311` | `0x04004000` | bit 14, bit 26 누락 |
| #831 | `0x042a2211` | `0x002a0211` | `0x04002000` | bit 13, bit 26 누락, `vertical_offset=-2429` 누락 |
| #835 | `0x042a6311` | `0x002a0311` | `0x04006000` | bit 13, bit 14, bit 26 누락 |

표준 필드인 TAC, 위치 기준, 정렬, 크기 기준, text wrap은 현재 parser/writer의 decode 기준으로
oracle과 generated가 같은 의미를 가진다. 차이는 현재 모델이 표현하지 않는 HWPX object contract
비트와 첫 번째 child picture의 세로 오프셋에 있다.

## 4. HWPX 원본과의 대응

`hwpx-h-03` target 구조:

```text
hp:rect id="1875692958"
  hp:drawText
    hp:subList
      hp:p
        hp:pic id="1875692960"
        hp:t>     </hp:t>
        hp:pic id="1875692962"
```

대응 관계:

```text
bit 13 후보: hp:pos@flowWithText
bit 14 후보: hp:pos@allowOverlap
bit 26 후보: GenShape 공통 high-bit. table adapter의 0x08000000과 다르므로 별도 계약으로 취급한다.
```

특히 `#831`은 HWPX 원본에 `vertOffset="4294964867"`이 있으며, 이는 signed `-2429`다.
oracle `CTRL_HEADER`에는 이 값이 그대로 들어가지만 현재 generated에서는 `0`으로 손실된다.

## 5. 현재 구현의 한계

현재 `CommonObjAttr`에는 다음 필드가 없다.

```text
flowWithText
allowOverlap
```

관련 경로:

```text
src/model/shape.rs
src/parser/hwpx/section.rs
src/document_core/converters/common_obj_attr_writer.rs
```

`pack_common_attr_bits()`도 bit 13, bit 14, bit 26을 합성하지 않는다.

## 6. Stage 25 판정

판정: **A, 단 guarded candidate**

```text
CTRL_HEADER field mapping만으로 Stage 26 구현 후보를 만들 수 있다.
```

단, 이것이 최종 원인 전체라는 뜻은 아니다. Stage 24에서 `SHAPE_COMPONENT`와 `SHAPE_PICTURE`
payload hash도 여전히 다르다. 따라서 Stage 26 후보가 `hwpx-h-03` 파일손상을 완전히 해소하지
못하면 다음 단계에서 component/picture payload 디코드로 넘어간다.

## 7. 다음 단계

Stage 26 권장 작업:

```text
1. `CommonObjAttr`에 HWPX object contract 필드를 정식 추가한다.
   - flow_with_text
   - allow_overlap
2. HWPX parser에서 hp:pos@flowWithText / hp:pos@allowOverlap을 파싱한다.
3. CommonObjAttr writer에서 source 값에 따라 bit 13/14를 합성한다.
4. GenShape HWPX 출처 object에 필요한 bit 26 후보를 조건부로 합성한다.
5. #831 first child picture의 vertical_offset 손실 경로를 확인하고 보정한다.
6. guard 파일을 동시에 생성한다.
```

Stage 26 guard:

```text
hwpx-h-01
hwpx-h-02
hwpx-h-03
hy-001
```

성공 기준:

```text
hwpx-h-01/hwpx-h-02: 기존 성공 유지
hwpx-h-03: 파일손상 해소 또는 중단 위치 전진
hy-001: #974 시각 guard 유지
```

## 8. 실행한 검증

```text
target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/hwpx-h-03-current.hwp \
  --section 0 --focus ctrl --report hints

rg '"record_index":824,|..."' \
  output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/*section0.jsonl

python3 -c '... attr bit diff decode ...'
```

`git diff --check` 통과.
