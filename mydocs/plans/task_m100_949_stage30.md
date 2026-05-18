# Task m100 #949 Stage 30 계획 - HWPX id/instid 분리 보존

## 1. 목표

Stage 29에서 확인한 가장 강한 contract 차이를 실제 구현 후보로 반영한다.

핵심은 HWPX 개체의 `id`와 `instid`를 같은 값처럼 취급하지 않고 HWP5 record의 서로 다른
필드로 내려 보내는 것이다.

```text
HWPX hp:*@id     -> HWP5 CTRL_HEADER CommonObjAttr.instance_id
HWPX hp:*@instid -> HWP5 SHAPE_COMPONENT tail / SHAPE_PICTURE extra tail instance_id
```

## 2. 근거

Stage 29 field map에서 `hwpx-h-03` 손상 지점은 record sequence 누락이 아니었다.

```text
BodyText tag/control graph: oracle == generated
CTRL_DATA #833: oracle == generated
SHAPE_RECTANGLE #838: oracle == generated
```

반면 같은 record 위치에서 다음 차이가 반복되었다.

| HWPX source | HWP5 target | 정답 값 | 현재 생성 값 |
|---|---|---:|---:|
| `hp:rect@id` | `#824 CTRL_HEADER.instance_id` | 1875692958 | 801951135 |
| `hp:rect@instid` | `#825 SHAPE_COMPONENT tail.inst_id` | 801951135 | 0 |
| `hp:pic[0]@id` | `#831 CTRL_HEADER.instance_id` | 1875692960 | 801951137 |
| `hp:pic[0]@instid` | `#834 SHAPE_PICTURE extra.instance_id` | 801951137 | 0 |
| `hp:pic[1]@id` | `#835 CTRL_HEADER.instance_id` | 1875692962 | 801951139 |
| `hp:pic[1]@instid` | `#837 SHAPE_PICTURE extra.instance_id` | 801951139 | 0 |

`hy-001`에서도 같은 패턴이 반복된다.

## 3. 구현 범위

### 3.1 HWPX parser

`src/parser/hwpx/section.rs`에서 HWPX 개체 속성을 다음처럼 분리한다.

```text
hp:*@id
  -> common.instance_id

hp:*@instid
  -> DrawingObjAttr.inst_id 또는 Picture.instance_id
```

우선 적용 대상:

```text
hp:rect / hp:ellipse / hp:line / hp:arc / hp:polygon / hp:curve
hp:pic
```

`hp:container`는 별도 tail 저장 위치가 현재 모델에 없으므로 이번 단계에서는
`CommonObjAttr.instance_id <- id`만 확인하고, `instid`는 후속 stage에서 group contract로 분리한다.

### 3.2 serializer

serializer는 이미 필요한 필드 기록 경로를 갖고 있으므로 최대한 건드리지 않는다.

```text
src/serializer/control.rs
- serialize_common_obj_attr(): CommonObjAttr.instance_id 기록
- serialize_drawing_shape_component(): DrawingObjAttr.inst_id 기록
- serialize_picture(): Picture.instance_id 기록
```

이번 단계는 serializer에 값을 공급하는 IR 보존 경로를 회복하는 데 집중한다.

### 3.3 테스트

다음 단위 테스트를 추가 또는 보강한다.

```text
hwpx-h-03 rect/drawText contract:
- outer rect common.instance_id == hp:rect@id
- outer rect drawing.inst_id == hp:rect@instid
- inner picture common.instance_id == hp:pic@id
- inner picture instance_id == hp:pic@instid
```

## 4. 생성/검증

구현 후 다음 파일을 생성한다.

```text
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hy-001.hwp
```

자동 검증:

```text
cargo fmt --check
cargo test hwpx_h_03_rect_draw_text_contract_from_source
cargo build
rhwp reload check for generated files
```

수동 판정 요청:

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-01.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-02.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-03.hwp` |  |  |  |  |  |  | target |
| `output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hy-001.hwp` |  |  |  |  |  |  | #974 guard |

## 5. 판정 기준

성공:

```text
hwpx-h-01/hwpx-h-02 guard 성공 유지
hwpx-h-03 파일손상 해소 또는 손상 위치가 다음 contract 축으로 이동
hy-001 #974 guard 회귀 없음
```

실패:

```text
hwpx-h-01/hwpx-h-02에서 표/이미지 회귀 발생
hwpx-h-03/hy-001 결과가 Stage 23/26/28과 완전히 동일
```

## 6. 다음 축

Stage 30 이후에도 손상이 남으면 다음 순서로 분리한다.

```text
1. text-box LIST_HEADER 13-byte tail
2. text-box PARA_HEADER tail `00 80 00 00`
3. drawing SHAPE_COMPONENT storage high bits
4. line/fill/shadow tail의 non-zero default materialization
```
