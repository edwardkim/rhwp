# Task m100 #949 Stage 1 작업 보고서: hwp5-inventory P0

## 1. 목적

#949의 첫 단계로 `rhwp hwp5-inventory` 명령을 추가했다.

이 도구는 HWPX -> HWP 저장기 구현 전에 한컴 HWP oracle과 rhwp generated HWP를 같은
HWP5 record/control inventory 언어로 관찰하기 위한 P0 진단 도구다.

Stage 1에서는 diff나 probe 생성은 하지 않고, HWP5 CFB 내부의 `DocInfo`와
`BodyText/SectionN` record 목록을 안정적으로 JSONL/Markdown으로 출력하는 데 집중했다.

## 2. 구현 내용

추가/수정한 소스:

```text
src/diagnostics/mod.rs
src/diagnostics/hwp5_inventory.rs
src/lib.rs
src/main.rs
```

추가 CLI:

```bash
rhwp hwp5-inventory <파일.hwp> [--format jsonl|md] [--section N] [--out <path>]
```

출력 item 필드:

```text
sample
source_path
stream_path
section
record_index
level
tag_id
tag_name
size
owner
parent_scope
key_payload
payload_hash
note
```

Stage 1의 `payload_hash`는 기존 의존성인 `blake3`를 사용한다.

## 3. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/oracle.inventory.jsonl
output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/oracle.inventory.md
output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/oracle.section0.inventory.jsonl
output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/oracle.section1.inventory.jsonl
output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/generated.inventory.jsonl
output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/generated.inventory.md
```

입력 파일:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp
```

## 4. 검증 결과

빌드:

```text
cargo fmt
cargo build
```

결과:

```text
cargo build 성공
```

라인 수:

```text
oracle.inventory.jsonl       8497
generated.inventory.jsonl    8490
oracle.inventory.md          8528
generated.inventory.md       8515
oracle.section0.inventory.jsonl 8409
oracle.section1.inventory.jsonl 616
```

stream별 record count:

```text
oracle /DocInfo              528
generated /DocInfo           523
oracle /BodyText/Section0    7881
generated /BodyText/Section0 7879
oracle /BodyText/Section1    88
generated /BodyText/Section1 88
```

`--section 0` 필터 확인:

```text
oracle.section0 /BodyText/Section0 7881
oracle.section0 /BodyText/Section1 0
oracle.section1 /BodyText/Section0 0
oracle.section1 /BodyText/Section1 88
```

Markdown header에는 stream 목록과 `section_count=2`가 출력된다.

## 5. 현재 한계

Stage 1은 inventory 기반을 만드는 단계라 다음은 아직 하지 않는다.

```text
hwp5-inventory-diff
HWPX control inventory
hwp5-probe-gen
HWPX -> HWP 저장기 수정
한컴 시각 판정용 신규 HWP 생성
```

`owner`, `parent_scope`, `key_payload`는 현재 최소 구현이다.
후속 Stage에서 `CTRL_HEADER`, `LIST_HEADER`, `TABLE`, `SHAPE_COMPONENT`, `PIC` tuple 단위로
정교화해야 한다.

## 6. 다음 단계

Stage 2 후보:

```text
1. inventory row의 tuple scope를 정교화한다.
2. CTRL_HEADER ctrl_id, TABLE, SHAPE/PIC 핵심 payload를 구조화한다.
3. hwp5-inventory-diff의 비교 키를 설계한다.
4. oracle/generated 사이의 누락, 추가, size, hash, 순서 차이를 분류한다.
```
