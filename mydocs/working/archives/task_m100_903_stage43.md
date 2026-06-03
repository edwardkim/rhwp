# Task m100 #903 Stage 43 작업 기록

## 1. 목적

Stage42에서 TABLE `row_sizes` 보정은 성공했지만, 한컴 에디터는 여전히 읽기오류를 냈다.

Stage43은 Stage37에서 좁힌 잔여 원인인 `SHAPE_COMPONENT` / `SHAPE_PICTURE` payload를
Stage42 산출물과 positive 기준 파일 사이에서 필드 단위로 비교한다.

## 2. 기준 파일

Target:

```text
output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp
```

Positive:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

출력:

```text
output/poc/hwpx2hwp/task903/stage43_shape_payload_diff/shape_payload_diff.md
```

## 3. 내부 검증

실행:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage43_generate_shape_payload_diff_report -- --nocapture
```

결과:

```text
test task903_stage43_generate_shape_payload_diff_report ... ok
```

검증 내용:

```text
record count: 7879
SHAPE_COMPONENT diff records: 6
SHAPE_PICTURE diff records: 5
```

## 4. Diff 목록

| tag | diff index |
|---|---|
| `SHAPE_COMPONENT` | `[21, 35, 807, 808, 810, 812]` |
| `SHAPE_PICTURE` | `[22, 36, 809, 811, 813]` |

이는 Stage37의 inventory와 일치한다.

## 5. SHAPE_COMPONENT 분석

`SHAPE_COMPONENT` 6개 모두 첫 차이는 `local_file_version`이다.

공통 패턴:

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

또한 그룹/묶음 쪽에서는 `current_width/current_height`가 target에서 0이다.

| idx | 대상 | target 주요값 | positive 주요값 |
|---:|---|---|---|
| 21 | 1페이지 표 안 그림 1 | `local=0`, `current=8307x2567`, `flip=0`, `center=(0,0)` | `local=1`, `current=8307x2567`, `flip=0x24000000`, `center=(4154,1284)` |
| 35 | 1페이지 표 안 그림 2 | `local=0`, `current=10659x2558`, `flip=0`, `center=(0,0)` | `local=1`, `current=10659x2558`, `flip=0x24000000`, `center=(5330,1279)` |
| 807 | 2페이지 묶음 컨테이너 | `local=0`, `current=0x0`, `flip=0`, `center=(0,0)` | `local=1`, `current=47509x3721`, `flip=0x00090000`, `center=(23754,1860)` |
| 808 | 묶음 자식 그림 | `local=0`, `current=0x0`, `flip=0`, `center=(0,0)` | `local=1`, `current=9480x3300`, `flip=0x24080000`, `center=(3430,1194)` |
| 810 | 묶음 자식 그림 | `local=0`, `current=0x0`, `flip=0`, `center=(0,0)` | `local=1`, `current=53640x8340`, `flip=0x24080000`, `center=(13889,1860)` |
| 812 | 묶음 자식 그림 | `local=0`, `current=0x0`, `flip=0`, `center=(0,0)` | `local=1`, `current=6082x2457`, `flip=0x24000000`, `center=(4475,1581)` |

판단:

```text
SHAPE_COMPONENT 차이는 raw_rendering 크기나 record size 문제가 아니다.
record size는 모두 positive와 같다.

문제는 ShapeComponentAttr의 HWP 저장용 필드가 HWPX 경로에서 0/default로 남는 것이다.
```

## 6. SHAPE_PICTURE 분석

`SHAPE_PICTURE` 5개는 size, crop, padding, effect, bin_data_id, raw_picture_extra 길이가 모두
positive와 같다.

차이는 `border_x[4]` / `border_y[4]` 좌표 배열이다.

| idx | binData | target border | positive border |
|---:|---:|---|---|
| 22 | 1 | `border_x=[0,0,0,0]`, `border_y=[0,0,0,0]` | `border_x=[0,0,37680,0]`, `border_y=[37680,9780,0,9780]` |
| 36 | 2 | `border_x=[0,0,0,0]`, `border_y=[0,0,0,0]` | `border_x=[0,0,189360,0]`, `border_y=[189360,76980,0,76980]` |
| 809 | 3 | `border_x=[0,0,0,0]`, `border_y=[0,0,0,0]` | `border_x=[0,0,9480,0]`, `border_y=[9480,3300,0,3300]` |
| 811 | 4 | `border_x=[0,0,0,0]`, `border_y=[0,0,0,0]` | `border_x=[0,0,53640,0]`, `border_y=[53640,8340,0,8340]` |
| 813 | 5 | `border_x=[0,0,0,0]`, `border_y=[0,0,0,0]` | `border_x=[0,0,6082,0]`, `border_y=[6082,2457,0,2457]` |

판단:

```text
SHAPE_PICTURE 차이는 BinData나 crop 문제가 아니다.
HWPX -> HWP 저장 경로에서 Picture border polygon 좌표가 채워지지 않고 있다.
```

## 7. 종합 해석

Stage43 결과는 다음 구현 후보를 지지한다.

```text
1. HWPX 출처 Picture/Group ShapeComponentAttr를 HWP 저장용 필드로 materialize한다.
   - local_file_version
   - current_width/current_height
   - flip
   - rotation_center

2. HWPX 출처 Picture의 border_x/border_y 좌표 배열을 materialize한다.
```

단, `rotation_center`와 `flip` 값은 단순한 하나의 상수로 볼 수 없다.
특히 그룹 자식 그림의 `rotation_center`는 단순 `current_width/2`, `current_height/2`와 다르다.
따라서 Stage44는 구현 전에 HWPX parser가 해당 값을 이미 읽을 수 있는지, 아니면 adapter에서
추정해야 하는지 확인해야 한다.

## 8. 하지 않은 것

```text
- 후보 HWP 생성 없음
- raw payload graft 없음
- TABLE attr/n_zones 구현 없음
```

Stage43은 보고서 단계로 종료한다.

## 9. 다음 단계

Stage44 계획은 다음 순서가 맞다.

```text
1. HWPX parser/model에 ShapeComponentAttr 필드가 들어오는지 확인한다.
2. 값이 모델에 있는데 serializer가 누락하는지, 모델 자체가 0인지 분리한다.
3. Picture border_x/border_y는 HWPX 원본에서 얻을 수 있는지 확인한다.
4. 확정된 필드만 adapter/serializer에 materialize한다.
5. 후보 HWP를 output/poc/hwpx2hwp/task903/stage44_... 아래 생성해 한컴 판정을 받는다.
```
