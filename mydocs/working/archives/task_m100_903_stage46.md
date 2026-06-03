# Task m100 #903 Stage 46 작업 기록

## 1. 목적

Stage45 판정에서 shape 잔여 필드 후보 4개가 모두 같은 실패 양상을 보였다.

```text
한컴 에디터:
  - 열기 성공
  - 이미지 출력 실패
  - 표/셀 배치 실패

rhwp-studio:
  - 조판 실패
```

따라서 Stage46은 shape 잔여 필드를 더 쪼개지 않고,
Stage37~40에서 이미 성공 조건으로 확인된 두 축을 Stage44/45 baseline 위에 조합한다.

```text
1. DocInfo BIN_DATA metadata
2. Stage40 필수 TABLE payload/tail
```

## 2. 생성 파일

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/
```

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/01_docinfo_bindata_only.hwp
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/02_table_required_payload_only.hwp
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/03_docinfo_bindata_plus_table_required.hwp
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/04_shape_full_payload_plus_table_required.hwp
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/05_docinfo_bindata_plus_shape_full_payload.hwp
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/integration_detail.md
```

## 3. 실행한 검증

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage46_generate_bindata_table_integration_probe -- --nocapture
```

결과:

```text
test task903_stage46_generate_bindata_table_integration_probe ... ok
```

내부 재로드:

| variant | bytes | rhwp reload |
|---|---:|---|
| `01_docinfo_bindata_only.hwp` | 374272 | ok, pages=9 |
| `02_table_required_payload_only.hwp` | 374272 | ok, pages=9 |
| `03_docinfo_bindata_plus_table_required.hwp` | 374272 | ok, pages=9 |
| `04_shape_full_payload_plus_table_required.hwp` | 374272 | ok, pages=9 |
| `05_docinfo_bindata_plus_shape_full_payload.hwp` | 374272 | ok, pages=9 |
| `06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp` | 374272 | ok, pages=9 |

## 4. 후보 의미

### 01_docinfo_bindata_only

Stage44/45 baseline에 positive의 `DocInfo BIN_DATA` metadata만 적용한다.

목적:

```text
이미지 출력 실패가 DocInfo BIN_DATA metadata 부족 때문인지 확인한다.
```

### 02_table_required_payload_only

Stage44/45 baseline에 Stage40 필수 TABLE payload/tail만 적용한다.

적용 index:

```text
48, 103, 286, 433, 563, 742, 1619, 2944, 6466
```

목적:

```text
표/셀 조판 실패가 TABLE payload/tail 부족 때문인지 확인한다.
```

### 03_docinfo_bindata_plus_table_required

`DocInfo BIN_DATA`와 Stage40 필수 TABLE payload/tail을 함께 적용한다.

목적:

```text
Stage44 shape materialization이 충분하고, 남은 핵심이 BIN_DATA + TABLE인지 확인한다.
```

### 04_shape_full_payload_plus_table_required

DocInfo BIN_DATA 없이 Stage37의 shape full payload와 Stage40 필수 TABLE payload/tail을 적용한다.

shape full index:

```text
21, 22, 35, 36, 807, 808, 809, 810, 811, 812, 813
```

목적:

```text
DocInfo BIN_DATA 없이 shape + TABLE만으로 이미지/조판이 회복되는지 확인한다.
```

### 05_docinfo_bindata_plus_shape_full_payload

TABLE payload 없이 DocInfo BIN_DATA와 shape full payload를 적용한다.

목적:

```text
TABLE 없이 이미지 출력만 회복 가능한지 확인한다.
```

### 06_docinfo_bindata_plus_shape_full_payload_plus_table_required

DocInfo BIN_DATA, shape full payload, Stage40 필수 TABLE payload/tail을 모두 적용한다.

목적:

```text
Stage37/40의 성공 조건을 Stage44 baseline 위에서 재현하는 상한 후보를 확인한다.
```

## 5. 상세 리포트

상세 리포트:

```text
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/integration_detail.md
```

핵심 차이:

```text
base BIN_DATA:
  attr=0x0
  status=NotAccessed
  raw_data 없음

positive BIN_DATA:
  attr=0x101
  status=Success
  raw_data 있음
```

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_docinfo_bindata_only | 성공 | 성공 | 실패 | 표 배치 실패 |  |
| 02_table_required_payload_only | 성공 | 실패 | 표 배치 실패 | 표 배치 실패 |  |
| 03_docinfo_bindata_plus_table_required | 성공 | 성공 | 실패 | 표 배치 실패 |  |
| 04_shape_full_payload_plus_table_required | 성공 | 실패 | 표 배치 실패 | 표 배치 실패 |  |
| 05_docinfo_bindata_plus_shape_full_payload | 성공 | 성공 | 실패 | 표 배치 실패 |  |
| 06_docinfo_bindata_plus_shape_full_payload_plus_table_required | 성공 | 성공 | 실패 | 표 배치 실패 |  |

## 7. 판정 해석 기준

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

06도 실패:
  Stage46 probe 구성 자체가 Stage37/40 성공 조건을 재현하지 못한 것이므로,
  기준 파일/record index/stream graft 방식부터 재점검한다.
```

## 8. 판정 해석

Stage46 판정으로 이미지 출력 원인은 확정되었다.

```text
DocInfo BIN_DATA 적용 있음:
  01, 03, 05, 06 모두 이미지 출력 성공

DocInfo BIN_DATA 적용 없음:
  02, 04 모두 이미지 출력 실패
```

따라서 이미지 출력 실패의 직접 원인은 `DocInfo BIN_DATA` metadata다.

```text
base:
  attr=0x0
  status=NotAccessed
  raw_data 없음

positive:
  attr=0x101
  status=Success
  raw_data 있음
```

반면 표/셀 배치는 Stage46의 모든 후보에서 실패했다.

```text
TABLE_REQUIRED 적용 있음:
  02, 03, 04, 06 모두 표 배치 실패

SHAPE_FULL + TABLE_REQUIRED 적용 있음:
  04, 06 모두 표 배치 실패
```

이는 Stage40의 성공 조건과 겉으로 충돌한다.
하지만 Stage40 성공 후보는 `Stage36 05_ctrl_list_para_headers`를 baseline으로 삼았다.
즉 Stage40의 성공 조건에는 다음 전제가 숨어 있었다.

```text
DocInfo BIN_DATA/PARA_SHAPE 보정
CTRL_HEADER/LIST_HEADER/PARA_HEADER payload graft
SHAPE_FULL payload graft
TABLE payload graft
```

Stage37에서 `CTRL_HEADER/LIST_HEADER/PARA_HEADER`는 "단독 원인"이 아니라고 보았지만,
Stage46 결과상 이 payload들은 `SHAPE + TABLE`이 의미를 갖기 위한 전제 조건일 가능성이 다시 열린다.

## 9. 결론

Stage46으로 확정된 것:

```text
1. 이미지 출력:
   DocInfo BIN_DATA metadata를 clean 구현해야 한다.

2. 표/셀 배치:
   Stage44 baseline에 TABLE_REQUIRED를 얹는 것만으로는 부족하다.
   Stage40 성공 baseline과 Stage46 06 실패 후보의 Section0 차이를 비교해야 한다.
```

다음 단계:

```text
Stage47에서 Stage40 성공 파일과 Stage46 06 실패 파일을 직접 비교한다.
비교 결과를 바탕으로 CTRL_HEADER/LIST_HEADER/PARA_HEADER payload가
SHAPE/TABLE payload와 함께 필요했는지 확인한다.
```
