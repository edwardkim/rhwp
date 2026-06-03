# Task M100-993 Stage 13 계획서

## 1. 목적

Stage 12 진행 중 작업지시자가 글자 그림자 효과가 HWP 저장 결과에 반영되지 않는 문제를 확인했다.

`mel-001.hwpx`에는 `hh:shadow`가 존재한다.

```xml
<hh:shadow type="CONTINUOUS" color="#CCCCCC" offsetX="8" offsetY="8"/>
```

그러나 현재 HWPX header parser는 `type`과 `color`만 읽고 `offsetX`, `offsetY`를 IR에 반영하지 않는다.
Stage 13의 목적은 이 문제를 `CharShape shadow` contract로 분리해 정답 HWP와 비교하고,
필요한 최소 구현 범위를 확정하는 것이다.

## 2. 현재 가설

```text
1. HWP5 parser/serializer와 IR에는 shadow offset 필드가 이미 있다.
2. HWPX parser가 offsetX/offsetY를 누락해 HWP 저장 시 shadow offset이 0으로 저장된다.
3. HWPX type="CONTINUOUS"의 HWP5 shadow_type 값은 정답지 비교가 필요하다.
```

따라서 이번 단계는 shadow를 표/셀 배치, 고스트 선, 대각선, gradient와 섞지 않는다.

## 3. 비교 대상

HWPX 원본:

```text
samples/hwpx/mel-001.hwpx
```

정답 HWP:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

현재 생성 기준:

```text
output/poc/hwpx2hwp/task993/stage11_borderfill_diagonal_contract/04_diagonal_attr_payload.hwp
```

## 4. 수행 절차

1. HWPX header에서 `hh:shadow`가 있는 `charPr` ID를 수집한다.
2. 정답 HWP의 같은 CHAR_SHAPE record를 추출한다.
3. 생성 HWP의 같은 CHAR_SHAPE record를 추출한다.
4. 다음 필드를 비교한다.

```text
- shadow_type
- shadow_offset_x
- shadow_offset_y
- shadow_color
- attr bit 11-12
```

5. `CONTINUOUS`와 `DROP`의 HWP5 type 매핑을 정답지 기준으로 확인한다.
6. 구현 후보는 HWPX parser의 shadow offset/type 매핑으로 제한한다.

## 5. 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage13_charshape_shadow_contract/
```

분석 문서:

```text
output/poc/hwpx2hwp/task993/stage13_charshape_shadow_contract/charshape_shadow_diff.md
```

필요 시 판정용 HWP:

| file | 목적 |
|---|---|
| `01_current.hwp` | 현재 기준 |
| `02_shadow_offset_fixed.hwp` | offsetX/offsetY 반영 후보 |
| `03_shadow_type_oracle.hwp` | shadow_type 정답지 투영 후보 |
| `04_shadow_full_candidate.hwp` | type/offset/color 전체 후보 |

## 6. 성공 기준

```text
1. 그림자 효과 누락 원인을 CHAR_SHAPE 필드 차이로 특정한다.
2. HWPX shadow offsetX/offsetY를 HWP5 shadow offset으로 매핑한다.
3. CONTINUOUS shadow type 매핑을 정답지 기준으로 검증한다.
4. 표/셀 배치, gradient, 대각선, 고스트 선 문제와 섞지 않는다.
```

## 7. 승인 요청

이 계획으로 Stage 13 `CharShape shadow` contract 추적을 진행한다.
