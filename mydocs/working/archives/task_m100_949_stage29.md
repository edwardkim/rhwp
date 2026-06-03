# Task m100 #949 Stage 29 - outer GenShape field map

## 1. 목적

Stage 23, 26, 28은 모두 `hwpx-h-03`의 한컴 파일손상 판정을 해소하지 못했다.
Stage 29에서는 새 후보 HWP를 만들지 않고, 손상 지점 주변의 HWPX source, IR, 정답 HWP,
현재 생성 HWP를 같은 record 좌표로 맞춰 원인을 좁힌다.

대상 지점은 `hwpx-h-03` 2페이지의 글상자 구조다.

```text
문단 시작부에 표가 먼저 나오고,
이후 텍스트 2줄 뒤에 글상자(rect/drawText)가 나오며,
그 글상자 안에 TAC 그림 2개와 space 문자열이 들어간다.
```

## 2. 입력

```text
samples/hwpx/hwpx-h-03.hwpx
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-03.hwp

samples/hwpx/hy-001.hwpx
samples/hwpx/hancom-hwp/hy-001.hwp
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hy-001.hwp
```

## 3. 산출물

```text
output/poc/hwpx2hwp/task949/stage29_outer_genshape_field_map/
```

주요 파일:

```text
h03_outer_genshape_field_map.md
h03_outer_genshape_xml_trace.md
h03_outer_genshape_ir_trace.md
hy001_textbox_field_map.md
stage29_findings.md
stage29_field_decoder.py
```

## 4. 확인된 사실

`hwpx-h-03` 손상 지점의 record sequence와 control graph는 정답 HWP와 현재 생성 HWP가
같다.

```text
BodyText tag/control graph: oracle == generated
CTRL_DATA #833: oracle == generated
SHAPE_RECTANGLE #838: oracle == generated
LIST/PARA 내부 텍스트 record: oracle == generated
```

따라서 현재 단계에서는 "컨트롤이 통째로 누락되었다"가 1순위 원인이 아니다.
같은 record 위치에서 payload lowering이 다르다.

## 5. 핵심 패턴

가장 강한 패턴은 HWPX `id`와 `instid`의 역할 분리다.

`hwpx-h-03` HWPX source:

```text
hp:rect id=1875692958 instid=801951135
  hp:drawText
    hp:subList
      hp:p
        hp:pic id=1875692960 instid=801951137 href=...
        hp:pic id=1875692962 instid=801951139
        hp:t "     "
```

정답 HWP 대응:

| HWPX source | HWP5 target | 정답 값 | 현재 생성 값 | 해석 |
|---|---|---:|---:|---|
| `hp:rect@id` | `#824 CTRL_HEADER.instance_id` | 1875692958 | 801951135 | 현재는 `instid`를 기록 |
| `hp:rect@instid` | `#825 SHAPE_COMPONENT tail.inst_id` | 801951135 | 0 | 현재는 미기록 |
| `hp:pic[0]@id` | `#831 CTRL_HEADER.instance_id` | 1875692960 | 801951137 | 현재는 `instid`를 기록 |
| `hp:pic[0]@instid` | `#834 SHAPE_PICTURE extra.instance_id` | 801951137 | 0 | 현재는 미기록 |
| `hp:pic[1]@id` | `#835 CTRL_HEADER.instance_id` | 1875692962 | 801951139 | 현재는 `instid`를 기록 |
| `hp:pic[1]@instid` | `#837 SHAPE_PICTURE extra.instance_id` | 801951139 | 0 | 현재는 미기록 |

`hy-001`에서도 같은 패턴이 반복된다. 즉 `id/instid` 분리는 `hwpx-h-03` 전용 예외가 아니라
HWPX 글상자 내부 TAC 그림을 HWP5로 낮출 때 필요한 공통 contract 후보로 본다.

## 6. 구현 경로 해석

현재 소스 경로는 다음 차이를 설명한다.

```text
src/parser/hwpx/section.rs
- parse_object_element_attrs(): 현재 instid -> CommonObjAttr.instance_id
- parse_picture(): 현재 instid -> CommonObjAttr.instance_id, Picture.instance_id는 보존하지 않음
- parse_shape_object(): DrawingObjAttr.inst_id는 기본값 0, shadow도 사실상 미반영

src/serializer/control.rs
- serialize_common_obj_attr(): CommonObjAttr.instance_id를 CTRL_HEADER instance_id로 기록
- serialize_drawing_shape_component(): DrawingObjAttr.inst_id를 SHAPE_COMPONENT tail에 기록
- serialize_picture(): Picture.instance_id를 SHAPE_PICTURE extra tail에 기록
```

따라서 현재 생성물이 보이는 차이는 우연한 byte drift가 아니다. HWPX parser가 `id`와
`instid`를 분리 보존하지 않아 serializer가 이미 준비된 tail 필드에 값을 공급받지 못하고 있다.

## 7. 아직 남은 차이

`id/instid` 외에도 다음 payload 차이가 남아 있다.

| record | 차이 | 다음 우선순위 |
|---:|---|---|
| #825 / hy #223 | outer rect `SHAPE_COMPONENT` attr offset 36 | drawing storage high bits 후보 |
| #825 / hy #223 | line/fill/shadow/instid tail | `instid` 반영 후 재판정 |
| #826 / hy #224 | text-box `LIST_HEADER` 13-byte tail 없음 | 후순위 |
| #827 / hy #225 | text-box `PARA_HEADER` tail `00 80 00 00` vs `00 00` | 후순위 |
| #834/#837 / hy #231/#234 | picture extra tail의 `instid`가 0 | `id/instid` 반영 대상 |

## 8. 다음 단계

Stage 30은 임의 graft가 아니라 다음 구현 후보로 진행한다.

```text
1. HWPX parser에서 hp:*@id와 hp:*@instid를 분리 보존한다.
   - CommonObjAttr.instance_id <- id
   - DrawingObjAttr.inst_id / Picture.instance_id <- instid
2. serializer는 기존 경로를 활용한다.
   - CTRL_HEADER에는 CommonObjAttr.instance_id
   - SHAPE_COMPONENT tail에는 DrawingObjAttr.inst_id
   - SHAPE_PICTURE extra tail에는 Picture.instance_id
3. guard:
   - hwpx-h-01
   - hwpx-h-02
   - hwpx-h-03
   - hy-001
4. 그래도 h03 손상이 남으면 LIST_HEADER/PARA_HEADER tail 또는 drawing storage bits를 다음 축으로 분리한다.
```

## 9. 실행한 검증

```text
python3 output/poc/hwpx2hwp/task949/stage29_outer_genshape_field_map/stage29_field_decoder.py
```

Stage 29는 새 HWP 후보를 만들지 않았다. 이번 단계의 산출물은 다음 구현 후보를 결정하기 위한
field map과 trace 문서다.
