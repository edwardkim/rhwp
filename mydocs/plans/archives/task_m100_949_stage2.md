# Task m100 #949 Stage 2 계획서: HWP5 inventory scope 정교화

## 1. 목적

Stage 1의 `hwp5-inventory`는 HWP5 record를 stream 순서대로 안정적으로 나열했다.

Stage 2에서는 이 record 목록을 HWPX -> HWP contract 분석에 바로 쓸 수 있도록
control tuple 단위로 정교화한다.

핵심 목표는 다음 질문에 답할 수 있는 inventory를 만드는 것이다.

```text
이 record는 어떤 문단/컨트롤/표/그림/셀 tuple에 속하는가?
oracle에는 있는데 generated에는 빠진 tuple은 무엇인가?
한컴 에디터가 민감하게 보는 header/payload/tail 후보는 어느 tuple에 연결되는가?
```

Stage 2에서는 아직 diff 명령을 만들지 않는다.
diff가 가능하도록 inventory item의 식별자와 scope 정보를 먼저 안정화한다.

## 2. Stage 1 한계

Stage 1 item은 다음 값을 제공한다.

```text
stream_path
section
record_index
level
tag_id / tag_name
size
parent_scope
key_payload
payload_hash
```

하지만 다음 정보가 부족하다.

```text
1. record별 stable uid
2. parent record uid
3. control tuple scope
4. CTRL_HEADER의 ctrl_id / ctrl_name 구조화 필드
5. TABLE / SHAPE_COMPONENT / PIC / LIST_HEADER / PARA_HEADER의 역할 구분
6. top-level body order와 nested owner 구분
```

## 3. Stage 2 구현 범위

구현할 것:

```text
1. inventory item에 scope 분석 필드를 추가한다.
2. level stack 기반 parent_scope를 parent_uid / scope_path 형태로 보강한다.
3. CTRL_HEADER payload의 ctrl_id / ctrl_name을 별도 필드로 분리한다.
4. record tag와 level 흐름을 이용해 tuple_role을 부여한다.
5. TABLE, SHAPE_COMPONENT, PIC, LIST_HEADER, PARA_HEADER 주변 record를 group 관찰 가능하게 만든다.
6. oracle/generated hwpx-h-01 inventory를 새 스키마로 다시 생성한다.
```

구현하지 않을 것:

```text
hwp5-inventory-diff
HWPX inventory
hwp5-probe-gen
HWPX -> HWP 저장기 수정
한컴 판정용 HWP variant 생성
```

## 4. 추가 스키마 초안

기존 필드는 유지하고 다음 필드를 추가한다.

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

필드 의미:

```text
record_uid:
  stream_path + section + record_index 기반의 stable id

parent_uid:
  level stack에서 가장 가까운 상위 record의 uid

scope_path:
  stream / section / top-level paragraph / nested control 흐름을 사람이 읽을 수 있게 표현

body_order:
  BodyText stream 안에서 top-level record 흐름을 추적하기 위한 순번

control_id / control_name:
  CTRL_HEADER일 때 payload 첫 4바이트를 해석한 값

tuple_role:
  docinfo
  para_header
  para_text
  para_char_shape
  para_line_seg
  ctrl_header
  list_header
  table
  shape_component
  pic
  unknown

tuple_index:
  같은 tuple_role 안에서의 순번

payload_head_hex:
  key_payload와 별도로 기계 비교에 쓰기 쉬운 첫 32바이트 hex
```

## 5. 판정 기준

Stage 2 완료 기준:

```text
1. cargo build 통과
2. 기존 Stage 1 JSONL/Markdown 생성 기능 유지
3. JSONL에 새 필드가 출력됨
4. CTRL_HEADER 행에서 control_id/control_name이 별도 필드로 출력됨
5. TABLE / SHAPE_COMPONENT / PIC record에 tuple_role이 부여됨
6. hwpx-h-01 oracle/generated inventory 재생성 성공
7. output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/ 아래 산출물 생성
```

예상 산출물:

```text
output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/
  oracle.inventory.jsonl
  oracle.inventory.md
  generated.inventory.jsonl
  generated.inventory.md
  stage2_report.md
```

## 6. 구현 위치

주 구현 파일:

```text
src/diagnostics/hwp5_inventory.rs
```

필요할 경우 보조 모듈을 추가한다.

```text
src/diagnostics/hwp5_inventory_scope.rs
```

CLI 이름과 사용법은 유지한다.

```bash
rhwp hwp5-inventory <파일.hwp> [--format jsonl|md] [--section N] [--out <path>]
```

## 7. 검증 명령

```bash
cargo build

./target/debug/rhwp hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --format jsonl \
  --out output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/oracle.inventory.jsonl

./target/debug/rhwp hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --format md \
  --out output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/oracle.inventory.md

./target/debug/rhwp hwp5-inventory output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --format jsonl \
  --out output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/generated.inventory.jsonl

./target/debug/rhwp hwp5-inventory output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --format md \
  --out output/poc/hwpx2hwp/task949/stage2/hwpx-h-01/generated.inventory.md
```

## 8. 다음 단계

Stage 3에서 `hwp5-inventory-diff`를 구현한다.

Stage 3 diff는 Stage 2의 `record_uid`, `scope_path`, `tuple_role`, `payload_hash`를 기준으로
다음을 분류한다.

```text
missing
extra
tag_changed
size_changed
payload_changed
order_changed
scope_changed
```

## 9. 승인 요청

이 계획은 Stage 1 inventory를 HWP5 contract 분석에 쓸 수 있는 형태로 정교화하는 단계다.

승인 후 Stage 2 구현을 진행한다.
