# Task m100 #949 Stage 3 계획서: hwp5-inventory-diff P0 구현

## 1. 목적

Stage 1/2에서 만든 `hwp5-inventory`를 바탕으로 한컴 HWP oracle과 rhwp generated HWP의
record 차이를 자동 분류하는 `hwp5-inventory-diff`를 구현한다.

이 도구는 HWPX -> HWP 저장기 구현 전에 다음 질문에 답하기 위한 P0 도구다.

```text
oracle에는 있는데 generated에는 없는 HWP5 record는 무엇인가?
generated에는 있는데 oracle에는 없는 record는 무엇인가?
같은 위치의 record인데 tag/size/payload/scope/control 해석이 달라지는 지점은 어디인가?
차이가 어느 stream, section, tuple_role, control_name에 집중되는가?
```

## 2. Stage 3 범위

구현할 것:

```text
1. `rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp>` CLI 추가
2. HWP 파일 두 개를 직접 읽어 inventory를 생성한 뒤 비교
3. JSONL + Markdown diff 출력 지원
4. diff kind 분류
   - missing
   - extra
   - tag_changed
   - size_changed
   - payload_changed
   - scope_changed
   - control_changed
5. stream_path + record_index 기반 P0 비교
6. `hwpx-h-01` oracle/generated smoke 산출물 생성
```

구현하지 않을 것:

```text
1. LCS 기반 record sequence alignment
2. HWPX inventory와의 직접 비교
3. probe HWP 자동 생성
4. HWPX -> HWP 저장기 수정
5. 한컴 판정용 신규 variant 생성
```

## 3. 비교 키

Stage 3 P0 비교 키:

```text
stream_path
record_index
```

이 키는 삽입/삭제가 많을 때 shift noise가 생길 수 있다.
하지만 P0 단계에서는 먼저 record 차이가 어느 stream/record range에서 시작되는지 보는 것이 목적이다.

후속 Stage에서 다음 키를 조합한 정렬을 추가한다.

```text
section
tuple_role
control_id
control_name
scope_path
payload_hash
```

## 4. 출력 스키마 초안

JSONL item:

```text
diff_kind
key
stream_path
section
record_index
oracle
generated
note
```

`oracle`과 `generated` summary:

```text
record_uid
tag_id
tag_name
size
tuple_role
tuple_index
control_id
control_name
scope_path
payload_hash
```

Markdown:

```text
summary count
diff table
```

## 5. 구현 위치

새 모듈:

```text
src/diagnostics/hwp5_inventory_diff.rs
```

모듈 등록:

```text
src/diagnostics/mod.rs
```

CLI 연결:

```text
src/main.rs
  Some("hwp5-inventory-diff") => rhwp::diagnostics::hwp5_inventory_diff::run(&args[2..])
```

## 6. 검증 명령

```bash
cargo build

./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --format jsonl \
  --out output/poc/hwpx2hwp/task949/stage3/hwpx-h-01/inventory.diff.jsonl

./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --format md \
  --out output/poc/hwpx2hwp/task949/stage3/hwpx-h-01/inventory.diff.md
```

## 7. 완료 기준

```text
1. cargo build 통과
2. CLI help 출력
3. JSONL diff 생성
4. Markdown diff 생성
5. summary count가 출력됨
6. hwpx-h-01 oracle/generated 비교에서 차이 row가 생성됨
7. Stage 3 작업 보고서 작성
```

## 8. 다음 단계

Stage 4에서는 P0 diff의 shift noise를 줄이기 위해 tuple-aware alignment를 설계한다.

후보:

```text
1. stream 단위 LCS
2. tuple_role + control_id 기반 구간 매칭
3. TABLE / PIC / SHAPE_COMPONENT tuple 묶음 비교
4. diff 결과를 contract_violation_hints.md 형태로 요약
```
