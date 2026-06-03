# Task m100 #903 Stage 44 계획

## 1. 목적

Stage43에서 `SHAPE_COMPONENT` / `SHAPE_PICTURE` payload 차이를 필드 단위로 확인했다.

남은 핵심 차이는 두 축이다.

```text
1. ShapeComponentAttr의 HWP 저장 호환 필드
   - local_file_version
   - current_width/current_height
   - flip
   - rotation_center

2. Picture border polygon 좌표
   - border_x[4]
   - border_y[4]
```

Stage44는 이 값을 clean adapter/serializer 경로에서 materialize하고 후보 HWP를 생성한다.

## 2. 근거

Stage43 diff:

```text
SHAPE_COMPONENT diff records:
  [21, 35, 807, 808, 810, 812]

SHAPE_PICTURE diff records:
  [22, 36, 809, 811, 813]
```

`SHAPE_COMPONENT` 관찰:

```text
target:
  local_file_version = 0
  flip = 0
  rotation_center = (0, 0)

positive:
  local_file_version = 1
  flip = non-zero
  rotation_center = non-zero
```

그룹/묶음 쪽 추가 관찰:

```text
target:
  current_width/current_height = 0

positive:
  current_width/current_height = original_width/original_height
```

`SHAPE_PICTURE` 관찰:

```text
target:
  border_x = [0,0,0,0]
  border_y = [0,0,0,0]

positive:
  original size 기반 non-zero polygon
```

## 3. 코드 사전 확인

현재 HWPX parser 상태:

```text
src/parser/hwpx/section.rs

parse_picture():
  - orgSz/curSz/offset/renderingInfo는 일부 ShapeComponentAttr에 매핑한다.
  - local_file_version, flip, rotation_center는 채우지 않는다.
  - border_x/border_y는 채우지 않는다.

parse_container():
  - orgSz/curSz/offset/renderingInfo는 공통 helper로 매핑한다.
  - local_file_version, flip, rotation_center는 채우지 않는다.

materialize_shape_current_size_from_original():
  - current size 0이면 original size로 보정하는 helper가 이미 있다.
```

따라서 Stage44는 parser/model 입력 누락과 adapter materialization을 나누어 처리한다.

## 4. 구현 범위

### 4.1 할 것

대상 파일 후보:

```text
src/document_core/converters/hwpx_to_hwp.rs
src/parser/hwpx/section.rs
```

우선순위:

```text
1. HWPX -> HWP adapter에서 Picture/Group ShapeComponentAttr 저장 호환 필드를 materialize한다.
2. HWPX -> HWP adapter에서 Picture border_x/border_y가 모두 0이면 original size 기반 좌표를 materialize한다.
3. 필요한 경우 HWPX parser의 parse_picture()/parse_container()에서 더 이른 시점에 값을 채운다.
```

구체 후보:

```text
ShapeComponentAttr:
  local_file_version == 0이면 1
  current_width/current_height == 0이면 original_width/original_height
  rotation_center == (0,0)이면 HWPX rendering/current/original 정보를 기준으로 보정
  flip == 0이면 HWP picture/group 저장 관례에 맞는 flag 후보를 적용

Picture:
  border_x/border_y가 모두 0이면 original_width/original_height 기반 polygon 좌표를 materialize
```

### 4.2 하지 않을 것

```text
- Stage37/38처럼 raw payload graft로 후보 파일 생성
- Section0 raw stream 전체 보존
- TABLE attr/n_zones=0 tail 구현
- BinData payload 변환 설계 변경
```

TABLE attr/n_zones는 Stage44 후보 판정 이후에도 문제가 남을 때 별도 Stage로 분리한다.

## 5. 검증 방식

추가 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage44_generate_shape_materialized_candidate -- --nocapture
```

검증 내용:

```text
1. Stage43의 SHAPE_COMPONENT 6개에 대해 주요 필드가 positive에 가까워졌는지 보고한다.
2. Stage43의 SHAPE_PICTURE 5개에 대해 border_x/border_y가 더 이상 0 배열이 아닌지 보고한다.
3. rhwp 재로드 기준 9페이지인지 확인한다.
```

출력:

```text
output/poc/hwpx2hwp/task903/stage44_shape_materialized_candidate/hwpx-h-01.hwp
output/poc/hwpx2hwp/task903/stage44_shape_materialized_candidate/shape_payload_after_materialize.md
```

## 6. 작업지시자 판정 요청

| 파일 | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task903/stage44_shape_materialized_candidate/hwpx-h-01.hwp` |  |  |  |  |  |  |  |

## 7. 판정 해석

```text
한컴 성공:
  Stage42 TABLE row_sizes + Stage44 SHAPE materialization 조합으로 #903 핵심 호환성 해결.

파일손상/읽기오류 위치가 뒤로 이동:
  SHAPE materialization이 유효하며, 남은 TABLE attr/n_zones 또는 다른 late object 축을 분리한다.

실패 위치가 Stage42와 동일:
  Stage44 materialization 규칙이 positive payload를 충분히 재현하지 못함.
  이 경우 Stage43 report 기준으로 개별 필드별 후보를 더 쪼갠다.
```

## 8. 성공 기준

```text
1. raw graft 없이 clean adapter/serializer 변경으로 후보 HWP를 만든다.
2. Stage43에서 확인한 SHAPE payload 차이를 줄인다.
3. 판정 파일은 output/poc 아래에 둔다.
```
