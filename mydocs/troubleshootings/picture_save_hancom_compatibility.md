---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 그림 저장 시 한컴 호환성 문제

## 증상

우리 편집기에서 그림을 삽입하고 저장한 HWP 파일을 한컴 워드프로세서에서 열었을 때:
1. 파일 손상 메시지 표시
2. 이미지가 표시되지 않음 (조판부호에서도 [이미지] 텍스트 미표시)
3. 이미지가 비정상적으로 큰 크기(425.20%)로 표시

## 원인 분석

### 1. CommonObjAttr `prevent_page_break` 필드 누락

CTRL_HEADER 직렬화 시 `prevent_page_break` (INT32, 4바이트) 필드가 누락되어
이후 필드(개체 설명문 길이 + 데이터)의 오프셋이 4바이트 밀림 → 파일 구조 손상.

**수정**: `serialize_common_obj_attr()`에 `prevent_page_break` 쓰기 추가,
`parse_common_obj_attr()`에 읽기 추가.

### 2. SHAPE_COMPONENT `ctrl_id` 값 오류

SHAPE_COMPONENT 레코드에서 그림의 ctrl_id가 `"gso "` (0x67736F20)으로 기록됨.
한컴 파일에서는 `"$pic"` (0x24706963)을 사용.

**수정**: `serialize_picture_control()`에서 `tags::SHAPE_PICTURE_ID` (`$pic`) 사용.
`local_file_version`도 1로 설정.

### 3. SHAPE_COMPONENT 렌더링 행렬 부재

렌더링 행렬이 `cnt=0 + 48바이트 zeros`로 직렬화되어 100바이트.
한컴 파일은 `cnt=1 + translation/scale/rotation 행렬 3세트`로 196바이트.

**수정**: `raw_rendering`이 비어있을 때 identity translation + scale(cur/orig) +
identity rotation 행렬을 생성.

### 4. SHAPE_COMPONENT_PICTURE `border_x/border_y`, `crop`, `extra` 부재

- `border_x/border_y`: 4 꼭짓점 좌표가 모두 0으로 설정됨
- `crop`: 이미지 원본 범위가 디스플레이 크기로 잘못 설정됨
- 직렬화 시 extra 9바이트(border_opacity + instance_id + image_effect) 누락

**수정**:
- border: `bx = [0, 0, width, 0]`, `by = [width, height, 0, height]`
- crop: 이미지 원본 픽셀 크기 × 75 (HWPUNIT/pixel at 96DPI)
- extra: `raw_picture_extra`가 비어있으면 9바이트 기본값 생성

### 5. PARA_LINE_SEG 기본값으로 이미지 미표시

그림 문단의 PARA_LINE_SEG가 텍스트 기본값(line_height=1000, text_height=1000)으로
설정되어 한컴에서 이미지 영역을 확보하지 못함.

**수정**:
- `line_height = height` (이미지 높이 HWPUNIT)
- `text_height = height`
- `baseline_distance = height × 850 / 1000`
- `segment_width = content_width` (페이지 컨텐츠 영역 너비)
- `tag = 0x00060000` (표준 LineSeg 태그)

**주의**: `segment_width`(offset 28)와 `tag`(offset 32) 필드를 혼동하기 쉬움.
LineSeg는 9개 필드 × 4바이트 = 36바이트/세그먼트.

### 6. CommonObjAttr `attr` 비트에 크기 기준 미설정

`attr` 필드의 `bit 15~17` (오브젝트 폭 기준)과 `bit 18~19` (높이 기준)이 0(paper)으로
설정되어 한컴이 width/height 값을 종이 대비 퍼센트로 해석.

- 42520 HWPUNIT → 425.20% (종이 대비)
- 22238 HWPUNIT → 222.38% (종이 대비)

**HWP 스펙 (표 72, CommonObjAttr 속성)**:
```
bit 15~17: 오브젝트 폭의 기준
  0=paper, 1=page, 2=column, 3=para, 4=absolute

bit 18~19: 오브젝트 높이의 기준
  0=paper, 1=page, 2=absolute
```

**수정**: attr에 `(4 << 15) | (2 << 18)` 추가 → width=absolute, height=absolute.
최종 attr: `0x000A0211` (기존 `0x00000211`).

### 7. `extract_str` JSON 이스케이프 미처리

개체 설명문에 줄바꿈(\n)이 포함될 때 JSON 이스케이프가 그대로 저장됨.
`"그림입니다.\n저장테스트"` → 리터럴 `\n`이 저장.

**수정**: `extract_str` 반환 타입을 `Option<&str>` → `Option<String>`으로 변경,
`\n`, `\r`, `\t`, `\\`, `\"` 디코딩 로직 추가.

## 바이너리 비교 방법

```python
import olefile, zlib, struct

ole = olefile.OleFileIO('file.hwp')
section = ole.openstream('BodyText/Section0').read()
data = zlib.decompress(section, -15)
# 레코드 파싱 후 필드별 비교
```

PARA_LINE_SEG 파싱 시 36바이트/세그먼트 (32바이트로 착각하기 쉬움):
```
offset 0:  text_start (u32)
offset 4:  vertical_pos (i32)
offset 8:  line_height (i32)
offset 12: text_height (i32)
offset 16: baseline_distance (i32)
offset 20: line_spacing (i32)
offset 24: column_start (i32)
offset 28: segment_width (i32)  ← offset 28
offset 32: tag (u32)            ← offset 32
```

## 영향 범위

- `src/wasm_api.rs`: insert_picture_native, extract_str, setPictureProperties
- `src/serializer/control.rs`: serialize_common_obj_attr, serialize_shape_component, serialize_picture_data, serialize_picture_control
- `src/parser/control.rs`: parse_common_obj_attr
- `src/model/shape.rs`: CommonObjAttr에 prevent_page_break 필드 추가
- `src/parser/tags.rs`: SHAPE_PICTURE_ID 상수 추가
- `src/serializer/byte_writer.rs`: write_f64 추가

## 교훈

1. HWP 바이너리 포맷은 필드 하나라도 누락되면 이후 모든 데이터 오프셋이 밀려 파일 손상 발생
2. 한컴은 `attr` 비트의 크기 기준 필드에 따라 width/height 해석 방법이 완전히 달라짐
3. 한컴 정상 파일과 우리 파일의 바이트 단위 비교가 필수적
4. PARA_LINE_SEG의 `segment_width`(offset 28)와 `tag`(offset 32) 혼동 주의
