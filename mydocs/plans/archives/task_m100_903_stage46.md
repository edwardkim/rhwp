# Task m100 #903 Stage 46 계획

## 1. 목적

Stage45 판정은 4개 후보 모두 같은 결과였다.

```text
한컴 에디터:
  - 열기 성공
  - 이미지 출력 실패
  - 표/셀 배치 실패

rhwp-studio:
  - 조판 실패
```

따라서 Stage45에서 분리한 `SHAPE_PICTURE raw_picture_extra`,
`SHAPE_COMPONENT raw_rendering`, `group child rendering_count=2`는 단독 원인이 아니다.

Stage46의 목적은 Stage44/45 baseline 위에서 다음 두 축을 다시 조합해,
이미지 실패와 조판 실패의 직접 원인을 분리하는 것이다.

```text
1. DocInfo BIN_DATA metadata
2. Stage40 필수 TABLE payload/tail
```

## 2. 근거

Stage35:

```text
DocInfo BIN_DATA 보정 여부가 rhwp-studio 이미지 렌더링에 영향을 주었다.
BIN_DATA 보정 있음: 1페이지 표 안 이미지 2개 중 1개만 렌더링
BIN_DATA 보정 없음: 이미지 렌더링 하지 않음
```

Stage37:

```text
성공 조합:
  SHAPE_COMPONENT + SHAPE_PICTURE + TABLE
```

Stage40:

```text
필수 TABLE payload index:
  48, 103, 286, 433, 563, 742, 1619, 2944, 6466

비필수로 확인된 후보:
  819, 6596, 6986, 7376 등
```

Stage44:

```text
파일 구조 안정화에는 성공했다.
한컴 에디터에서 읽기 오류 없이 열린다.
하지만 이미지 출력과 표 배치는 실패했다.
```

Stage45:

```text
positive raw_picture_extra와 raw_rendering을 적용해도 이미지/표 배치는 회복되지 않았다.
```

따라서 다음 probe는 shape 잔여 필드를 더 쪼개지 않고,
이미 알려진 `DocInfo BIN_DATA`와 `TABLE payload/tail` 축을 Stage44/45 baseline에 결합한다.

## 3. 작업 범위

### 3.1 할 것

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/
```

생성 후보:

```text
01_docinfo_bindata_only.hwp
02_table_required_payload_only.hwp
03_docinfo_bindata_plus_table_required.hwp
04_shape_full_payload_plus_table_required.hwp
05_docinfo_bindata_plus_shape_full_payload.hwp
06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp
```

상세 보고서:

```text
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/integration_detail.md
```

각 후보 의미:

```text
01_docinfo_bindata_only:
  이미지 실패가 DocInfo BIN_DATA metadata 때문인지 확인한다.

02_table_required_payload_only:
  조판 실패가 Stage40 필수 TABLE payload/tail 때문인지 확인한다.

03_docinfo_bindata_plus_table_required:
  Stage44 shape materialization에 DocInfo BIN_DATA + 필수 TABLE만 더하면 충분한지 확인한다.

04_shape_full_payload_plus_table_required:
  DocInfo BIN_DATA 없이 shape full payload + TABLE만으로 이미지/조판이 회복되는지 확인한다.

05_docinfo_bindata_plus_shape_full_payload:
  TABLE payload 없이 DocInfo BIN_DATA + shape full payload만으로 이미지가 회복되는지 확인한다.

06_docinfo_bindata_plus_shape_full_payload_plus_table_required:
  Stage37/40의 성공 조건을 Stage44 baseline에서 재현하는 상한 후보로 사용한다.
```

### 3.2 하지 않을 것

```text
- Stage45의 raw_rendering matrix를 더 세분화
- group child rendering_count 후보 반복
- Section0 전체 raw stream graft
- FileHeader/DocInfo 전체 graft
- clean production 구현 확정
```

Stage46은 아직 원인 분리 probe다.
production 구현은 Stage46 판정 이후 최소 필드로 다시 계획한다.

## 4. 구현 방식

테스트 파일:

```text
tests/hwpx_to_hwp_adapter.rs
```

추가 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage46_generate_bindata_table_integration_probe -- --nocapture
```

기준 파일:

```text
base:
  output/poc/hwpx2hwp/task903/stage44_shape_materialized_candidate/hwpx-h-01.hwp

positive:
  output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

비교/적용 대상:

```text
DocInfo:
  HWPTAG_BIN_DATA record 5개

BodyText/Section0:
  SHAPE_COMPONENT [21, 35, 807, 808, 810, 812]
  SHAPE_PICTURE   [22, 36, 809, 811, 813]
  TABLE           [48, 103, 286, 433, 563, 742, 1619, 2944, 6466]
```

## 5. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_docinfo_bindata_only |  |  |  |  |  |
| 02_table_required_payload_only |  |  |  |  |  |
| 03_docinfo_bindata_plus_table_required |  |  |  |  |  |
| 04_shape_full_payload_plus_table_required |  |  |  |  |  |
| 05_docinfo_bindata_plus_shape_full_payload |  |  |  |  |  |
| 06_docinfo_bindata_plus_shape_full_payload_plus_table_required |  |  |  |  |  |

## 6. 판정 해석

```text
01에서 이미지만 회복:
  DocInfo BIN_DATA metadata가 이미지 출력의 직접 원인이다.

02에서 표/셀 배치만 회복:
  Stage40 필수 TABLE payload/tail이 조판의 직접 원인이다.

03이 정상:
  Stage44 shape materialization은 충분하고, 남은 핵심은 DocInfo BIN_DATA + TABLE이다.

04 또는 05만 개선:
  Stage44 shape materialization이 아직 부족하며, full shape payload 쪽 구현이 더 필요하다.

06만 정상:
  DocInfo BIN_DATA + SHAPE full payload + TABLE payload가 모두 필요하다.
  이후 Stage47에서 clean 구현 범위를 세 축으로 나눠야 한다.

06도 실패:
  Stage46 probe 구성 자체가 Stage37/40 성공 조건을 재현하지 못한 것이므로,
  기준 파일/record index/stream graft 방식부터 재점검한다.
```

## 7. 성공 기준

```text
1. Stage45에서 막힌 이미지 실패와 조판 실패를 서로 다른 축으로 다시 분리한다.
2. Stage37/40 성공 조건을 Stage44 baseline 위에서 재현한다.
3. 다음 production 구현 범위를 DocInfo BIN_DATA, TABLE payload, SHAPE payload 중 어디까지로 할지 확정한다.
```
