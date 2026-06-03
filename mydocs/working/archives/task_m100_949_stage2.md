# Task m100 #949 Stage 2 작업 보고서: HWP5 inventory scope 정교화

## 1. 목적

Stage 1의 `hwp5-inventory`는 HWP5 record를 stream 순서대로 나열했다.

Stage 2에서는 후속 `hwp5-inventory-diff`가 record를 기계적으로 비교할 수 있도록
inventory row에 record 식별자, parent 관계, scope path, control id, tuple role을 추가했다.

## 2. 구현 내용

수정한 핵심 파일:

```text
src/diagnostics/hwp5_inventory.rs
```

Stage 1에서 추가한 CLI와 사용법은 그대로 유지했다.

```bash
rhwp hwp5-inventory <파일.hwp> [--format jsonl|md] [--section N] [--out <path>]
```

## 3. 추가된 필드

JSONL item에 다음 필드를 추가했다.

```text
record_uid
parent_uid
scope_path
body_order
control_id
control_name
tuple_role
tuple_index
payload_head_hex
```

필드 해석:

```text
record_uid:
  stream path와 record index 기반의 stable id

parent_uid:
  record level stack에서 가장 가까운 상위 record의 uid

scope_path:
  stream / parent record / current record 흐름을 사람이 읽을 수 있는 경로로 표현

body_order:
  BodyText top-level record 흐름 기준 순번

control_id / control_name:
  CTRL_HEADER payload 첫 4바이트를 해석한 값

tuple_role:
  docinfo, para_header, para_text, para_char_shape, para_line_seg,
  ctrl_header, list_header, table, shape_component, pic 등

tuple_index:
  stream 안에서 같은 tuple_role이 몇 번째로 나타났는지

payload_head_hex:
  첫 32바이트 payload를 기계 비교하기 쉬운 hex 문자열로 분리
```

## 4. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/oracle.inventory.jsonl
output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/oracle.inventory.md
output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/generated.inventory.jsonl
output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/generated.inventory.md
```

입력 파일:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp
```

## 5. 검증 결과

빌드:

```text
cargo build 성공
```

라인 수:

```text
oracle.inventory.jsonl       8497
generated.inventory.jsonl    8490
oracle.inventory.md          8528
generated.inventory.md       8515
```

role count:

```text
ctrl_header      oracle 196 / generated 196
table            oracle 26  / generated 26
pic              oracle 5   / generated 5
shape_component  oracle 6   / generated 6
list_header      oracle 1453 / generated 1453
para_header      oracle 1582 / generated 1582
```

CTRL_HEADER 구조화 예:

```text
record_uid: BodyText.Section0#13
tag_name: CTRL_HEADER
tuple_role: ctrl_header
control_id: 0x74626c20
control_name: Table
scope_path: BodyText/Section0/PARA_HEADER#0/CTRL_HEADER#13
```

Markdown 출력도 새 컬럼을 포함한다.

```text
uid
role
tuple
ctrl
scope
```

## 6. 현재 한계

Stage 2의 scope는 HWP5 record level stack 기반이다.
아직 semantic owner를 완전히 판정하지는 않는다.

예를 들어 다음은 후속 Stage에서 더 정교하게 해야 한다.

```text
1. TABLE ctrl header와 TABLE record를 하나의 table tuple로 묶기
2. table cell LIST_HEADER / PARA_HEADER 구간을 cell tuple로 묶기
3. SHAPE_COMPONENT / SHAPE_COMPONENT_PICTURE / CTRL_DATA를 picture tuple로 묶기
4. nested control owner를 HWPX control id와 비교 가능한 형태로 정규화하기
```

## 7. 다음 단계

Stage 3에서는 `hwp5-inventory-diff`를 구현한다.

비교 기준 후보:

```text
stream_path
section
record_index
record_uid
tag_id / tag_name
tuple_role
control_id / control_name
size
payload_hash
scope_path
```

diff 분류 후보:

```text
missing
extra
tag_changed
size_changed
payload_changed
order_changed
scope_changed
```
