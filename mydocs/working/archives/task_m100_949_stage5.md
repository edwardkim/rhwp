# Task m100 #949 Stage 5 작업 보고서: contract violation hints

## 1. 목적

Stage 4의 LCS alignment diff는 index shift 노이즈를 크게 줄였지만, 여전히 사람이
다음 probe 후보를 직접 골라야 했다.

Stage 5에서는 `hwp5-inventory-diff`에 `--report hints`를 추가하여, diff row를
contract 위반 후보 목록으로 재분류한다.

## 2. 구현 내용

변경 파일:

```text
src/diagnostics/hwp5_inventory_diff.rs
src/main.rs
```

추가 CLI:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report hints \
  --out <path>
```

`--report diff`는 기존 JSONL/Markdown diff row 출력이다.
`--report hints`는 Markdown 후보 보고서만 출력한다.

## 3. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage5/hwpx-h-01/contract_violation_hints.md
```

생성 명령:

```bash
./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --align lcs \
  --report hints \
  --out output/poc/hwpx2hwp/task949/stage5/hwpx-h-01/contract_violation_hints.md
```

산출물 크기:

```text
186 lines
```

## 4. 주요 결과

LCS alignment summary:

```text
matched: 4607
changed: 3883
missing: 7
extra: 0
```

상위 role/control 버킷:

```text
para_header: 1582 changed
list_header: 1453 changed
docinfo: 521 changed, 4 missing
para_char_shape: 159 changed
para_line_seg: 93 changed
ctrl_header/Table: 26 changed
table: 26 changed
pic: 5 changed
shape_component: 5 changed
ctrl_header/GenShape: 3 changed
ctrl_data: 2 missing
forbidden_char: 1 missing
```

Missing record:

```text
BodyText.Section0#808 CTRL_DATA
BodyText.Section0#810 CTRL_DATA
DocInfo#523 DOC_DATA
DocInfo#524 FORBIDDEN_CHAR
DocInfo#525 COMPATIBLE_DOCUMENT
DocInfo#526 LAYOUT_COMPATIBILITY
DocInfo#527 TRACKCHANGE
```

## 5. 해석

Stage 5 리포트는 다음 probe 후보를 분리하기 위한 인덱스 역할을 한다.

```text
1. TABLE 계열:
   CTRL_HEADER(Table) 26건과 TABLE payload 26건이 함께 바뀐다.

2. 그림/도형 계열:
   PIC 5건, SHAPE_COMPONENT 5건, CTRL_HEADER(GenShape) 3건이 바뀐다.

3. DocInfo 계열:
   DocInfo payload changed가 521건으로 매우 많고, oracle에만 있는 tail record 4건이 있다.

4. BodyText missing:
   CTRL_DATA 2건이 oracle에만 있다.
```

이 단계는 contract를 확정하지 않는다. 다음 probe의 출발점을 좁히는 도구 단계다.

## 6. 검증

```text
cargo build: 통과
--report hints 산출물 생성: 통과
--report diff JSONL smoke: 통과
```

## 7. 다음 단계 후보

Stage 6에서는 hints report를 기준으로 다음 중 하나를 선택한다.

```text
1. TABLE tuple contract probe
2. PIC/SHAPE_COMPONENT + DocInfo BinData reference probe
3. DocInfo tail record probe
4. CTRL_DATA missing probe
```
