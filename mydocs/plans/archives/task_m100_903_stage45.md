# Task m100 #903 Stage 45 계획

## 1. 목적

Stage44 후보는 한컴 에디터에서 열리기는 성공했지만, 렌더링 의미가 깨졌다.

판정:

```text
한컴 에디터:
  - 열기 성공
  - 이미지 출력 실패
  - 그림 경로 찾기 대화창 표시
  - 표 배치 엉망

rhwp-studio:
  - 이미지 출력 실패
  - 표 배치 엉망
```

Stage45의 목적은 Stage44에서 남은 shape payload 차이를 더 분리해,
이미지 참조 실패와 표/개체 배치 실패의 직접 원인을 확인하는 것이다.

## 2. 현재 확정된 것

Stage30:

```text
마지막 페이지 미출력:
  DocProperties.section_count 누락

표/셀 세로 배치 비정상:
  HWPX ParaShape margin 계열 값 누락
```

Stage41/42:

```text
TABLE row_sizes는 cell height가 아니라 row별 cell count로 materialize해야 한다.
Stage42에서 row_sizes는 정답 기준으로 맞추었다.
```

Stage43:

```text
Stage42 이후 남은 핵심 payload 차이:
  SHAPE_COMPONENT [21, 35, 807, 808, 810, 812]
  SHAPE_PICTURE   [22, 36, 809, 811, 813]
```

Stage44:

```text
SHAPE_PICTURE border_x/border_y는 정답지와 일치
ShapeComponent 기본 storage 필드는 일부 회복
하지만 raw_rendering / raw_picture_extra 차이 잔존
```

따라서 Stage45에서는 다시 TABLE row_sizes, section_count, ParaShape로 돌아가지 않는다.

## 3. Stage44 잔여 차이

Stage44 report:

```text
SHAPE_COMPONENT remaining diff records:
  [21, 35, 807, 808, 810, 812]

SHAPE_PICTURE remaining diff records:
  [22, 36, 809, 811, 813]
```

잔여 필드:

```text
SHAPE_COMPONENT:
  - raw_rendering 바이트 차이
  - group child picture rendering_count 1 vs positive 2
  - 일부 group child flip flag 0x24000000 vs 0x24080000

SHAPE_PICTURE:
  - border_x/border_y는 일치
  - raw_picture_extra 18바이트 내용 차이
```

판정과 연결하면 다음 가설이 된다.

```text
이미지 출력 실패:
  SHAPE_PICTURE raw_picture_extra 또는 image instance/bindata reference 필드 누락/오매핑

표 배치 엉망:
  SHAPE_COMPONENT raw_rendering sequence 손실 또는 group child rendering_count 불일치
```

## 4. 작업 범위

### 4.1 분석 리포트 추가

Stage45는 먼저 상세 리포트를 만든다.

출력:

```text
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/shape_remaining_detail.md
```

리포트 내용:

```text
1. Stage44 output vs positive의 SHAPE_COMPONENT raw_rendering hex 전체 비교
2. Stage44 output vs positive의 SHAPE_PICTURE raw_picture_extra hex 전체 비교
3. 각 picture의 bin_data_id, common.instance_id, picture.instance_id 비교
4. rendering_count와 matrix raw byte 길이 비교
```

### 4.2 후보 HWP 생성

분리 판정을 위해 다음 후보를 만든다.

```text
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/01_picture_extra_from_positive.hwp
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/02_shape_raw_rendering_from_positive.hwp
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/03_picture_extra_plus_shape_raw_rendering.hwp
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/04_group_child_rendering_count2_clean_candidate.hwp
```

각 후보의 의미:

```text
01_picture_extra_from_positive:
  이미지 경로 찾기 대화창이 raw_picture_extra 때문인지 확인한다.

02_shape_raw_rendering_from_positive:
  표/개체 배치 엉망이 raw_rendering 때문인지 확인한다.

03_picture_extra_plus_shape_raw_rendering:
  두 축이 함께 필요할 때의 상한 후보를 확인한다.

04_group_child_rendering_count2_clean_candidate:
  positive raw graft 없이 group child rendering_count=2 직렬화 규칙만 적용한 clean 후보를 확인한다.
```

주의:

```text
01~03은 원인 분리용 probe다.
최종 구현 후보가 아니다.
최종 구현은 04 또는 04 이후 clean parser/serializer 규칙으로만 정리한다.
```

## 5. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage45_generate_shape_remaining_probe -- --nocapture
```

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_picture_extra_from_positive |  |  |  |  |  |
| 02_shape_raw_rendering_from_positive |  |  |  |  |  |
| 03_picture_extra_plus_shape_raw_rendering |  |  |  |  |  |
| 04_group_child_rendering_count2_clean_candidate |  |  |  |  |  |

## 7. 판정 해석

```text
01에서 이미지만 회복:
  raw_picture_extra/image instance 계열을 parser/serializer에 clean 매핑한다.

02에서 배치만 회복:
  raw_rendering sequence 보존 또는 rendering_count=2 규칙이 배치 핵심 원인이다.

03만 정상:
  이미지 참조와 raw_rendering이 모두 필요하다.

04가 정상:
  raw graft 없이 clean rendering_count=2 규칙으로 해결 가능하다.

모두 실패:
  Stage44 이후 남은 차이 외에 DocInfo BinData 또는 late TABLE payload 축을 다시 열어야 한다.
```

## 8. 성공 기준

```text
1. Stage44보다 한컴/rhwp 렌더링 의미가 개선되는 후보를 찾는다.
2. 이미지 출력 실패와 표 배치 실패의 원인을 서로 분리한다.
3. probe 결과를 바탕으로 다음 clean 구현 범위를 확정한다.
```
