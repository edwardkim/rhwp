# Task m100 #903 Stage 45 작업 기록

## 1. 목적

Stage44 판정에서 한컴 에디터 열기는 성공했지만 이미지 출력과 표 배치가 실패했다.

Stage45는 남은 두 축을 분리한다.

```text
1. SHAPE_PICTURE raw_picture_extra / image reference
2. SHAPE_COMPONENT raw_rendering / group child rendering_count
```

## 2. 생성 파일

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/
```

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/01_picture_extra_from_positive.hwp
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/02_shape_raw_rendering_from_positive.hwp
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/03_picture_extra_plus_shape_raw_rendering.hwp
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/04_group_child_rendering_count2_clean_candidate.hwp
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/shape_remaining_detail.md
```

파일 크기:

```text
01: 366K
02: 366K
03: 366K
04: 366K
```

초기 생성 중 01~03이 27K로 작게 생성되는 문제가 있었다. 원인은 HWP 재파싱 문서를 base로
쓰면서 BinData content가 probe에 포함되지 않은 것이다. 판정 파일로 부적합하므로 폐기하고,
HWPX 원본의 BinData content를 보존한 clean document에 Stage44 raw section을 주입하는 방식으로
다시 생성했다.

## 3. 실행한 검증

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage45_generate_shape_remaining_probe -- --nocapture
```

결과:

```text
test task903_stage45_generate_shape_remaining_probe ... ok
```

내부 재로드:

```text
01 rhwp pages = 9
02 rhwp pages = 9
03 rhwp pages = 9
04 rhwp pages = 9
```

## 4. 후보 의미

### 01_picture_extra_from_positive

Stage44 output의 `SHAPE_PICTURE` record 중 잔여 index의 `raw_picture_extra` 18바이트만
positive에서 가져온다.

목적:

```text
한컴의 그림 경로 찾기 대화창이 raw_picture_extra의 instance_id 누락 때문인지 확인한다.
```

### 02_shape_raw_rendering_from_positive

Stage44 output의 `SHAPE_COMPONENT` 잔여 index record를 positive에서 가져온다.

목적:

```text
표/개체 배치 엉망이 SHAPE_COMPONENT raw_rendering sequence 때문인지 확인한다.
```

주의:

```text
이 후보는 record 단위 graft다.
최종 구현 후보가 아니라 원인 분리용 상한 probe다.
```

### 03_picture_extra_plus_shape_raw_rendering

01과 02를 함께 적용한다.

목적:

```text
이미지 참조와 shape rendering sequence가 동시에 필요할 때의 상한을 확인한다.
```

### 04_group_child_rendering_count2_clean_candidate

positive raw graft 없이 HWPX -> IR -> HWP clean path에서 group child picture의 explicit
rendering matrix를 지우고 serializer의 group child 기본 `rendering_count=2` 경로를 타게 한다.

목적:

```text
raw graft 없이 rendering_count=2 직렬화 규칙만으로 배치가 회복되는지 확인한다.
```

## 5. 상세 분석 요약

상세 리포트:

```text
output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe/shape_remaining_detail.md
```

### SHAPE_PICTURE raw_picture_extra

Stage44 base와 positive의 가장 중요한 차이는 `instance_id`다.

예:

```text
idx 22
base:
  instance_id = 0

positive:
  instance_id = 489953954
```

모든 대상 picture에서 같은 패턴이 보인다.

```text
idx 22: base instance_id=0, positive instance_id=489953954
idx 36: base instance_id=0, positive instance_id=489953956
idx 809: base instance_id=0, positive instance_id=402773715
idx 811: base instance_id=0, positive instance_id=402773716
idx 813: base instance_id=0, positive instance_id=402773717
```

따라서 이미지 경로 찾기 대화창은 `SHAPE_PICTURE.raw_picture_extra` 안의 instance_id 누락과
강하게 연결되어 있을 가능성이 있다.

### SHAPE_COMPONENT raw_rendering

top-level picture 일부는 `rendering_count=1`로 같지만 matrix byte가 다르다.

group child picture는 차이가 더 크다.

```text
idx 808: base rendering_count=1, positive rendering_count=2
idx 810: base rendering_count=1, positive rendering_count=2
idx 812: base rendering_count=1, positive rendering_count=2
```

또 일부 group child는 flip flag도 다르다.

```text
base:     0x24000000
positive: 0x24080000
```

따라서 표/개체 배치 엉망은 `raw_rendering` sequence, 특히 group child의 `rendering_count=2`
직렬화 차이와 연결되어 있을 가능성이 있다.

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_picture_extra_from_positive | 성공 | 실패 | 실패 | 조판 실패 |  |
| 02_shape_raw_rendering_from_positive | 성공 | 실패 | 실패 | 조판 실패 |  |
| 03_picture_extra_plus_shape_raw_rendering | 성공 | 실패 | 실패 | 조판 실패 |  |
| 04_group_child_rendering_count2_clean_candidate | 성공 | 실패 | 실패 | 조판 실패 |  |

## 7. 판정 해석 기준

```text
01에서 이미지가 회복:
  raw_picture_extra의 instance_id 계열을 clean parser/serializer에 매핑해야 한다.

02에서 배치가 회복:
  SHAPE_COMPONENT raw_rendering sequence가 배치 핵심 원인이다.

03만 정상:
  이미지 참조와 raw_rendering 두 축이 모두 필요하다.

04가 정상:
  raw graft 없이 group child rendering_count=2 clean 규칙으로 해결 가능하다.

모두 실패:
  Stage44 잔여 shape payload 외에 BinData metadata 또는 late TABLE payload 축을 다시 열어야 한다.
```

## 8. 판정 해석

Stage45 판정으로 다음을 확인했다.

```text
1. 한컴 파일 읽기 오류/파일손상은 사라진 상태가 유지된다.
2. 그러나 이미지 출력은 4개 후보 모두 실패했다.
3. 표/셀 배치도 4개 후보 모두 실패했다.
4. rhwp-studio도 4개 후보 모두 조판 실패다.
```

따라서 Stage45에서 분리한 두 잔여 shape 축은 단독 원인이 아니다.

```text
SHAPE_PICTURE raw_picture_extra:
  positive의 instance_id를 가져와도 이미지 출력이 회복되지 않았다.
  따라서 그림 경로/이미지 출력 실패는 raw_picture_extra 단독 문제가 아니다.

SHAPE_COMPONENT raw_rendering:
  positive raw_rendering을 가져와도 표/셀 배치가 회복되지 않았다.
  따라서 조판 실패는 raw_rendering 단독 문제가 아니다.

group child rendering_count=2:
  clean 후보에서도 배치가 회복되지 않았다.
  따라서 이 규칙만으로는 충분하지 않다.
```

Stage37~40의 성공 조건과 결합하면 다음 해석이 더 타당하다.

```text
1. 이미지 출력은 SHAPE_PICTURE payload만이 아니라 DocInfo BIN_DATA metadata와 함께 맞아야 한다.
2. 표/셀 배치는 Shape payload만이 아니라 Stage40에서 확인한 필수 TABLE payload/tail과 함께 맞아야 한다.
3. Stage44/45는 파일 구조 안정화에는 성공했지만, 이미지/조판 의미를 회복하기 위한 DocInfo BIN_DATA + TABLE payload 축이 아직 부족하다.
```

## 9. 다음 단계

Stage46에서는 Stage45에서 실패한 shape 잔여 필드를 더 쪼개지 않는다.

다음 두 축을 Stage44/45 baseline 위에 조합해 확인한다.

```text
1. DocInfo BIN_DATA metadata
   - 이미지 경로 찾기/이미지 미출력 원인 후보

2. Stage40 필수 TABLE payload/tail
   - 표/셀 조판 실패 원인 후보
```

판정 목적은 `이미지 실패`와 `조판 실패`를 다음처럼 다시 분리하는 것이다.

```text
DocInfo BIN_DATA만으로 이미지가 회복되는지
TABLE payload/tail만으로 표/셀 조판이 회복되는지
둘을 함께 적용해야 하는지
shape payload 전체 graft까지 함께 필요할지
```
