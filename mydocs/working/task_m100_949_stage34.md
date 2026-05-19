# Task M100-949 Stage 34 작업 기록

## 1. 목적

Stage 33 판정에서 `hwpx-h-03.hwp`는 rhwp-studio에서 정상 조판되었지만, 한컴 에디터에서는
여전히 파일손상 판정을 받았다. 따라서 Stage 34는 조판 문제가 아니라 한컴 HWP5 loader가
요구하는 남은 record contract를 정답 HWP와 비교해 분리한다.

## 2. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage34_hancom_loader_contract/
```

주요 파일:

```text
h03_shape_hints.md
h03_shape_bundles.md
h03_outer_genshape_field_map.md
h03_outer_genshape_xml_trace.md
hy_shape_hints.md
hy_shape_bundles.md
hy001_textbox_field_map.md
hy001_textbox_xml_trace.md
```

## 3. 닫힌 축

Stage 33 생성물 기준으로 다음 record는 정답 HWP와 바이트 단위로 일치한다.

### `hwpx-h-03`

```text
#824 CTRL_HEADER
#826 LIST_HEADER
#827 PARA_HEADER
#831 CTRL_HEADER
#834 SHAPE_PICTURE
#835 CTRL_HEADER
#837 SHAPE_PICTURE
#838 SHAPE_RECTANGLE
```

### `hy-001`

```text
#222 CTRL_HEADER
#224 LIST_HEADER
#225 PARA_HEADER
#229 CTRL_HEADER
#231 SHAPE_PICTURE
#232 CTRL_HEADER
#234 SHAPE_PICTURE
#235 SHAPE_RECTANGLE
```

따라서 다음 축은 현 단계에서 다시 열지 않는다.

```text
- hp:pic@href -> CTRL_DATA
- id/instid 분리
- drawText LIST_HEADER 13-byte tail
- drawText 내부 PARA_HEADER tail
- SHAPE_PICTURE extra tail
- SHAPE_RECTANGLE 좌표 record
- lineSegArray.vpos 보존
```

## 4. 남은 축

남은 차이는 두 샘플 모두 `SHAPE_COMPONENT`에 집중된다.

### `hwpx-h-03`

```text
#825 outer text-box rect SHAPE_COMPONENT
#832 first inner picture SHAPE_COMPONENT
#836 second inner picture SHAPE_COMPONENT
```

### `hy-001`

```text
#223 outer text-box rect SHAPE_COMPONENT
#230 first inner picture SHAPE_COMPONENT
#233 second inner picture SHAPE_COMPONENT
```

## 5. 필드 단위 관찰

`SHAPE_COMPONENT`의 head 영역은 대부분 이미 정답과 일치한다.

```text
- ctrl_id
- offset_x / offset_y
- group_level
- local_file_version
- orgSz / curSz
- rotation angle / center
- rendering matrix block
```

남은 차이는 다음이다.

```text
1. offset 36 storage flag
   - h03 outer rect: oracle 0x01000000, generated 0
   - hy outer rect: oracle 0x01080000, generated 0
   - hy inner pictures: oracle 0x24080000, generated 0x24000000

2. outer rect rendering 뒤 drawing tail
   - lineShape color/attr
   - fillBrush color/pattern
   - shadow color/alpha
   - instid tail은 이미 일치
```

## 6. HWPX source와 연결되는 근거

두 샘플 모두 문제 지점의 HWPX source에는 같은 구조가 있다.

```text
hp:rect
  hp:lineShape
  hc:fillBrush
  hp:shadow
  hp:drawText
    hp:subList
      hp:p
        hp:pic
        hp:t
        hp:pic
```

특히 Stage 34 현재 구현은 `hp:shadow`를 무시하고 있으며, `hp:rotationInfo@rotateimage`도
`SHAPE_COMPONENT` offset 36의 `0x00080000` bit로 반영하지 않는다.

## 7. 해석

Stage 34의 결론은 다음이다.

```text
남은 한컴 파일손상 후보는 drawText/textbox envelope 자체가 아니라,
그 외곽 rect 및 내부 picture의 SHAPE_COMPONENT storage payload다.
```

다음 단계에서는 임의 graft가 아니라 다음 HWPX 필드를 HWP5 `SHAPE_COMPONENT`로 정확히
낮추는 후보를 구현해야 한다.

```text
1. hp:rotationInfo@rotateimage -> SHAPE_COMPONENT offset 36 bit 0x00080000
2. hp:rect drawText 외곽 SHAPE_COMPONENT 기본 storage bit 0x01000000
3. hp:lineShape color/style/headfill/tailfill/alpha 계열 -> drawing border_line
4. hp:shadow type/color/offset/alpha -> drawing shadow fields
```

