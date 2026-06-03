# Task m100 #903 Stage 43 계획

## 1. 목적

Stage42에서 HWPX -> HWP adapter의 TABLE `row_sizes` 보정은 구현되었고,
필수 TABLE 9개의 `row_sizes`가 positive와 일치함을 확인했다.

하지만 작업지시자 판정은 다음과 같았다.

```text
한컴 에디터: 읽기오류
rhwp-studio: 비정상 렌더링
```

따라서 Stage42는 Stage30/37/40의 결론을 뒤집지 않는다.
오히려 다음 결론을 확정한다.

```text
1. TABLE row_sizes는 필요한 축이지만 충분 조건은 아니다.
2. Stage37에서 확인한 SHAPE_COMPONENT / SHAPE_PICTURE payload 축이 남아 있다.
3. broad raw graft가 아니라 SHAPE payload 차이를 필드 단위로 해석해야 한다.
```

Stage43의 목적은 `SHAPE_COMPONENT` / `SHAPE_PICTURE` 차이를 object 단위와 필드 단위로
분해하여, clean adapter/serializer에서 구현 가능한 최소 변경 후보를 확정하는 것이다.

## 2. 근거

Stage37:

```text
성공 조건:
  SHAPE_COMPONENT + SHAPE_PICTURE + TABLE

차이 record:
  SHAPE_COMPONENT 6개
  SHAPE_PICTURE 5개
  TABLE 13개
```

Stage40:

```text
필수 TABLE index:
  48,103,286,433,563,742,1619,2944,6466
```

Stage42:

```text
필수 TABLE row_sizes:
  모두 positive와 일치

남은 차이:
  - SHAPE_COMPONENT / SHAPE_PICTURE payload
  - TABLE attr
  - TABLE n_zones=0 tail
```

## 3. 작업 범위

### 3.1 할 것

```text
1. Stage42 산출물과 positive 기준 파일의 Section0 record를 비교한다.
2. SHAPE_COMPONENT 6개, SHAPE_PICTURE 5개를 index/object 단위로 목록화한다.
3. 각 payload 차이를 hex와 가능한 decoded field로 보고한다.
4. 차이가 IR/model 필드 누락인지, serializer packing 문제인지 분류한다.
5. 단일하고 확정적인 매핑이 나오면 최소 구현 후보를 작성한다.
```

비교 기준 파일:

```text
target:
  output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp

positive:
  output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp

reference:
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage43_shape_payload_diff/
```

보고서:

```text
output/poc/hwpx2hwp/task903/stage43_shape_payload_diff/shape_payload_diff.md
```

### 3.2 하지 않을 것

```text
- Section0 raw stream 전체 graft
- SHAPE/TABLE raw payload를 reference에서 그대로 복사하는 구현
- Stage37~40에서 이미 확인한 probe 반복
- TABLE attr/n_zones=0 tail 구현
```

TABLE attr/n_zones는 Stage43에서 원인 후보로 기록만 하고,
SHAPE payload 분석 후에도 필요할 때 Stage44로 분리한다.

## 4. 자동 검증

추가할 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage43_generate_shape_payload_diff_report -- --nocapture
```

검증 내용:

```text
1. Stage42 target과 positive의 record count가 비교 가능해야 한다.
2. SHAPE_COMPONENT diff record가 6개인지 확인한다.
3. SHAPE_PICTURE diff record가 5개인지 확인한다.
4. 보고서가 object/index 단위로 생성되는지 확인한다.
```

## 5. 산출물

```text
mydocs/working/task_m100_903_stage43.md
output/poc/hwpx2hwp/task903/stage43_shape_payload_diff/shape_payload_diff.md
```

필드 매핑이 충분히 확정되면 추가 산출물:

```text
output/poc/hwpx2hwp/task903/stage43_shape_payload_candidate/hwpx-h-01.hwp
```

단, 이 후보 HWP는 raw graft가 아니라 clean adapter/serializer 변경으로만 생성한다.

## 6. 판정 요청 조건

Stage43에서 후보 HWP를 생성하는 경우에만 작업지시자 판정을 요청한다.

| 파일 | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task903/stage43_shape_payload_candidate/hwpx-h-01.hwp` |  |  |  |  |  |  |  |

보고서만 생성되는 경우에는 한컴 판정 없이 Stage44 구현 계획으로 넘어간다.

## 7. 기대 해석

```text
SHAPE payload 차이가 명확한 모델/serializer 필드로 매핑됨:
  Stage43에서 최소 구현 후보를 만들고 HWP를 생성한다.

SHAPE payload 차이가 여러 필드 조합으로 남음:
  Stage43은 기술 보고서로 종료하고, Stage44에서 세부 필드별 구현 계획을 세운다.

SHAPE payload를 맞춰도 한컴 실패가 유지됨:
  TABLE attr/n_zones=0 tail을 Stage44에서 분리한다.
```

## 8. 성공 기준

```text
1. Stage42 실패를 "row_sizes 실패"로 오판하지 않는다.
2. Stage37의 SHAPE_COMPONENT / SHAPE_PICTURE 차이를 필드 단위로 설명한다.
3. broad raw graft 없이 clean implementation 후보를 확정한다.
```
