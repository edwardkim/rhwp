# Task m100 #949 Stage 6 계획서: candidate bundle report

## 1. 목적

Stage 5의 `contract_violation_hints.md`는 어떤 role/control이 다른지 요약했다.
하지만 실제 probe를 설계하려면 후보 record 앞뒤의 oracle/generated record 흐름을 같이 봐야 한다.

Stage 6에서는 `hwp5-inventory-diff`에 bundle report를 추가한다.

목표 산출물:

```text
output/poc/hwpx2hwp/task949/stage6/hwpx-h-01/table_bundles.md
```

## 2. 구현 범위

추가 CLI:

```bash
--report bundles
--focus all|table|shape|ctrl|missing|docinfo
--window N
```

의미:

```text
bundles:
  diff 후보별로 oracle/generated record 주변 window를 같이 출력한다.

focus:
  후보 record를 어떤 계열로 제한할지 선택한다.

window:
  후보 record 앞뒤로 출력할 record 개수다.
```

기본값:

```text
--report diff
--focus all
--window 2
```

## 3. Stage 6 산출물

우선 TABLE 후보를 대상으로 한다.

```bash
./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --align lcs \
  --report bundles \
  --focus table \
  --window 3 \
  --out output/poc/hwpx2hwp/task949/stage6/hwpx-h-01/table_bundles.md
```

## 4. bundle report 구성

```text
# HWP5 Candidate Bundles

## Input
## Alignment Summary
## Candidate Summary
## Bundles
```

각 bundle에는 다음을 넣는다.

```text
candidate key
status / changed fields
oracle focused record
generated focused record
oracle 주변 record window
generated 주변 record window
payload_head_hex
key_payload
payload_hash
scope_path
```

## 5. 완료 기준

```text
1. cargo build 통과
2. 기존 --report diff/hints 동작 유지
3. --report bundles --focus table 산출물 생성
4. 산출물에서 CTRL_HEADER(Table) + TABLE 주변 record 흐름을 확인 가능
5. Stage 6 작업 보고서 작성
```

## 6. 다음 단계

Stage 7에서는 Stage 6 bundle report를 바탕으로 TABLE tuple contract probe 또는
PIC/SHAPE bundle report로 확장한다.
