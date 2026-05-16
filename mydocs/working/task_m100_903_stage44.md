# Task m100 #903 Stage 44 작업 기록

## 1. 목적

Stage43에서 확인한 `SHAPE_COMPONENT` / `SHAPE_PICTURE` payload 차이를
raw graft 없이 clean parser/serializer 경로에서 줄이는 후보를 만든다.

대상 샘플:

```text
samples/hwpx/hwpx-h-01.hwpx
```

정답 기준:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

## 2. 구현 내용

수정 대상:

```text
src/parser/hwpx/section.rs
tests/hwpx_to_hwp_adapter.rs
```

Stage43 차분을 바탕으로 HWPX 원본 XML을 재확인했다. 누락 필드는 HWPX 원본에 존재했다.

```text
hp:flip
hp:rotationInfo
hp:imgRect
```

따라서 Stage44는 adapter에서 추정값을 만드는 대신, HWPX parser가 원본 값을 IR에 싣도록 수정했다.

구현된 매핑:

```text
hp:flip
  -> ShapeComponentAttr.horz_flip / vert_flip
  -> HWP 저장용 flip 기본 flag materialize

hp:rotationInfo
  -> ShapeComponentAttr.rotation_angle
  -> ShapeComponentAttr.rotation_center

hp:imgRect/hc:pt0..pt3
  -> Picture.border_x / border_y
  -> 한컴 정답 HWP payload와 같은 스칼라 packing 순서 사용

ShapeComponentAttr
  -> local_file_version 기본값 1 materialize
  -> current_width/current_height 0이면 original_width/original_height로 materialize
```

## 3. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage44_shape_materialized_candidate/hwpx-h-01.hwp
output/poc/hwpx2hwp/task903/stage44_shape_materialized_candidate/shape_payload_after_materialize.md
```

크기:

```text
366K
```

## 4. 실행한 검증

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage44_generate_shape_materialized_candidate -- --nocapture
```

결과:

```text
test task903_stage44_generate_shape_materialized_candidate ... ok
```

내부 확인:

```text
rhwp reload pages = 9
record count = 7879
```

## 5. Stage43 대비 개선

`SHAPE_PICTURE`의 `imgRect` 기반 border 좌표는 정답지와 일치했다.

```text
idx 22  match true
idx 36  match true
idx 809 match true
idx 811 match true
idx 813 match true
```

예:

```text
output border_x = [0, 0, 37680, 0]
output border_y = [37680, 9780, 0, 9780]
positive와 일치
```

`SHAPE_COMPONENT` 주요 필드도 정답지에 가까워졌다.

```text
local_file_version = 1
rotation_center = non-zero
current_width/current_height = non-zero
flip = non-zero
```

## 6. 남은 차이

리포트 기준으로 아직 다음 diff가 남아 있다.

```text
SHAPE_COMPONENT remaining diff records:
  [21, 35, 807, 808, 810, 812]

SHAPE_PICTURE remaining diff records:
  [22, 36, 809, 811, 813]
```

남은 차이의 성격:

```text
SHAPE_COMPONENT:
  - raw_rendering 바이트 차이
  - 그룹 자식 picture에서 rendering_count 1 vs positive 2
  - 일부 group child flip flag 0x24000000 vs 0x24080000

SHAPE_PICTURE:
  - border_x/border_y는 일치
  - raw_picture_extra 18바이트 내용 차이
```

즉 Stage44는 Stage43에서 보인 구조 필드 누락은 상당 부분 줄였지만,
한컴 정답 HWP와 완전히 같은 shape payload는 아직 아니다.

## 7. 작업지시자 판정 요청

아래 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

| 파일 | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task903/stage44_shape_materialized_candidate/hwpx-h-01.hwp` | 열기 성공 | - | - | 엉망 | 실패 | 열림, 이미지 실패, 표 배치 엉망 | 한컴은 그림 경로 찾기 대화창 표시 |

## 8. 판정 해석 기준

```text
한컴 성공:
  Stage42 row_sizes + Stage44 shape materialization 조합으로 핵심 문제 해결.

한컴 실패 위치가 뒤로 이동:
  Stage44 방향은 유효. 남은 raw_rendering/raw_picture_extra 또는 late TABLE 축을 다음 stage에서 분리.

Stage42와 동일한 읽기 오류:
  shape 기본 필드 materialization만으로는 부족.
  Stage43/44 리포트의 raw_rendering, raw_picture_extra 차이를 다음 stage에서 우선 조사.
```

## 9. 판정 해석

Stage44 판정은 Stage42와 다르다.

```text
Stage42:
  한컴 읽기 오류
  rhwp-studio 비정상 렌더링

Stage44:
  한컴 열기 성공
  하지만 이미지 출력 실패
  표 배치 엉망
  rhwp-studio도 이미지 출력 실패 + 표 배치 엉망
```

따라서 Stage44에서 추가한 `flip`, `rotationInfo`, `imgRect`,
`local_file_version`, `current_width/current_height` materialization은
파일 구조 안정화에는 효과가 있었다고 판단한다.

그러나 렌더링 의미는 여전히 깨져 있다.

```text
이미지 출력 실패:
  SHAPE_PICTURE의 raw_picture_extra 또는 image instance/bindata 연결 정보가 아직 부족할 가능성이 높다.
  Stage44 리포트에서도 border 좌표는 정답지와 일치했지만 raw_picture_extra 18바이트 차이가 남았다.

표 배치 엉망:
  SHAPE_COMPONENT의 raw_rendering 차이가 남아 있다.
  특히 그룹 자식 picture에서 positive는 rendering_count=2인데 Stage44 output은 rendering_count=1이다.
  이 차이가 이미지 묶음/표 주변 배치를 깨뜨릴 가능성이 높다.
```

다음 단계는 TABLE row_sizes나 DocProperties가 아니다. Stage30/41/42에서 이미 그 축은 정리되었다.
Stage45는 Stage44에 남은 두 축만 다룬다.

```text
1. SHAPE_PICTURE raw_picture_extra / image reference
2. SHAPE_COMPONENT raw_rendering / group child rendering_count
```
