# Task m100 #949 Stage 6 작업 보고서: candidate bundle report

## 1. 목적

Stage 5의 hints report는 후보 role/control을 요약했다.
Stage 6에서는 후보 record 주변의 oracle/generated record 흐름을 함께 출력하는
bundle report를 추가했다.

이 단계의 목적은 다음 probe를 만들 때 record 단위 맥락을 바로 확인할 수 있게 하는 것이다.

## 2. 구현 내용

수정 파일:

```text
src/diagnostics/hwp5_inventory.rs
src/diagnostics/hwp5_inventory_diff.rs
src/main.rs
```

추가 CLI:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report bundles \
  --focus all|table|shape|ctrl|missing|docinfo \
  --window N \
  --out <path>
```

변경 사항:

```text
1. `--report bundles` 추가
2. `--focus` 후보 필터 추가
3. `--window` 주변 record 범위 추가
4. diff 내부에 oracle/generated inventory item을 보존하여 context window 출력
5. bundle row에 payload_head_hex, key_payload, payload_hash, scope_path 출력
```

## 3. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage6/hwpx-h-01/table_bundles.md
```

생성 명령:

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

산출물 크기:

```text
1694 lines
```

## 4. 산출물 요약

TABLE focus 후보:

```text
candidate_count: 52
ctrl_header/Table: 26
table: 26
```

첫 bundle에서 관찰되는 대표 차이:

```text
CTRL_HEADER(Table):
  oracle    head: ... 1a 00 00 00 1b 01 1b 01
  generated head: ... 1a 00 00 00 00 00 00 00

TABLE:
  oracle    size 24, head starts 0e 00 00 00 ...
  generated size 22, head starts 05 00 00 00 ...

LIST_HEADER:
  oracle    size 65
  generated size 34

PARA_HEADER:
  oracle    size 24, tail ends 00 80 00 00
  generated size 22, tail ends 00 00
```

이제 TABLE 후보는 단일 record가 아니라 다음 tuple 흐름으로 볼 수 있다.

```text
CTRL_HEADER(Table)
TABLE
LIST_HEADER
PARA_HEADER
PARA_TEXT / PARA_CHAR_SHAPE / PARA_LINE_SEG
```

## 5. 검증

```text
cargo build: 통과
--report bundles --focus table: 산출물 생성 통과
--report hints smoke: 통과
--report diff --format jsonl smoke: 3890 lines
```

## 6. 주의 사항

bundle report는 판정기가 아니다.
현재 단계는 probe 설계에 필요한 record context를 보여주는 도구다.

특히 다음을 아직 확정하지 않는다.

```text
1. TABLE payload의 어느 필드가 한컴 contract에 필수인지
2. CTRL_HEADER(Table)의 tail 값 의미
3. LIST_HEADER/PARA_HEADER size 차이가 TABLE failure의 직접 원인인지
4. HWPX 저장기에서 어떤 IR 필드로부터 이 값을 계산해야 하는지
```

## 7. 다음 단계 후보

Stage 7 후보:

```text
1. TABLE bundle에서 CTRL_HEADER(Table) + TABLE + LIST_HEADER 차이를 필드 단위로 해석
2. `--focus shape`로 PIC/SHAPE_COMPONENT bundle report 생성
3. `--focus missing`으로 CTRL_DATA / DocInfo tail missing bundle report 생성
```
