# Task m100 #949 Stage 1 계획서: hwp5-inventory P0 구현

## 1. 목적

#949는 #946에서 설계한 HWPX -> HWP contract 추출 도구 중 P0 도구를 실제 구현하는 작업이다.

Stage 1에서는 저장기 본 로직을 수정하지 않고, 한컴 HWP oracle과 rhwp generated HWP를 같은
HWP5 record/control 언어로 읽기 위한 `hwp5-inventory` 기반을 만든다.

## 2. 현재 코드 조사 결과

기존 코드에서 재사용 가능한 지점은 다음과 같다.

```text
src/parser/record.rs
  Record::read_all(data)
  tag_id / level / size / data 파싱
  extended size record 처리

src/parser/cfb_reader.rs
  CfbReader::open(data)
  read_stream_raw()
  read_doc_info(compressed)
  read_body_text_section(index, compressed, distribution)
  list_streams()
  section_count()

src/main.rs
  dump-records 명령
  현재는 BodyText/Section0만 사람이 읽는 stdout으로 출력
```

현 `dump-records`의 한계:

```text
Section0만 처리한다.
DocInfo record inventory를 만들지 않는다.
JSONL/Markdown 산출물이 없다.
owner / parent_scope / key_payload가 표준화되어 있지 않다.
output/poc report 경로와 연결되어 있지 않다.
```

## 3. Stage 1 범위

Stage 1에서 구현할 것:

```text
1. diagnostics 모듈 추가
2. HWP5 inventory 내부 데이터 구조 추가
3. HWP CFB에서 FileHeader / DocInfo / BodyText section stream을 읽는 inventory reader 추가
4. `rhwp hwp5-inventory <file.hwp>` CLI 추가
5. JSONL + Markdown 출력 지원
6. `samples/hwpx/hancom-hwp/hwpx-h-01.hwp` smoke test
```

Stage 1에서 하지 않을 것:

```text
hwp5-inventory-diff 구현
HWPX control inventory 구현
hwp5-probe-gen 구현
HWPX -> HWP 저장기 수정
한컴 시각 판정용 신규 HWP 생성
```

## 4. 출력 스키마 초안

Stage 1 inventory item은 다음 필드를 갖는다.

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

Stage 1에서는 `owner`, `parent_scope`, `key_payload`를 완전 확정하지 않는다.
다만 다음 최소 규칙을 넣는다.

```text
owner:
  DocInfo / BodyText / unknown

parent_scope:
  record level stack 기반의 직전 상위 record 요약

key_payload:
  우선 tag별 첫 16~32바이트 hex
  CTRL_HEADER / LIST_HEADER / TABLE / PARA_HEADER 등은 후속 Stage에서 해석 확장
```

## 5. 구현 위치

새 모듈:

```text
src/diagnostics/mod.rs
src/diagnostics/hwp5_inventory.rs
```

CLI 연결:

```text
src/main.rs
  Some("hwp5-inventory") => diagnostics::hwp5_inventory::run(&args[2..])
```

라이브러리 공개:

```text
src/lib.rs
  pub mod diagnostics;
```

## 6. CLI 사용 형태

필수:

```bash
cargo run --bin rhwp -- hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

옵션:

```bash
cargo run --bin rhwp -- hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --format jsonl \
  --out output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/oracle.inventory.jsonl

cargo run --bin rhwp -- hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --format md \
  --out output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/oracle.inventory.md

cargo run --bin rhwp -- hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --section 0 \
  --format md
```

## 7. 검증 기준

Stage 1 완료 기준:

```text
1. cargo build 통과
2. hwp5-inventory 명령이 한컴 oracle HWP를 읽고 JSONL/Markdown을 생성한다.
3. DocInfo와 BodyText/SectionN record가 모두 inventory에 포함된다.
4. BodyText section_count와 실제 Section stream 목록이 report에 보인다.
5. output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/ 아래 산출물이 생성된다.
```

예상 산출물:

```text
output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/
  oracle.inventory.jsonl
  oracle.inventory.md
  stage1_report.md
```

## 8. 후속 Stage

Stage 1 이후:

```text
Stage 2:
  owner / parent_scope / key_payload 정교화
  CTRL_HEADER / LIST_HEADER / TABLE / SHAPE / PIC tuple scope 추적

Stage 3:
  hwp5-inventory-diff 구현
  oracle/generated 누락/추가/값 다름/순서 다름 분류

Stage 4:
  hwpx-h-01 web_save_repro 비교
  contract_violation_hints.md 생성
```

## 9. 승인 요청

이 계획은 #949의 첫 구현 단계다.

승인 후 Stage 1 구현을 시작한다.
