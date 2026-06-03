# Task m100 #949 Stage 4 작업 보고서: tuple-aware alignment diff

## 1. 목적

Stage 4에서는 Stage 3의 `record_index` 기반 diff가 만드는 shift noise를 줄이기 위해
`hwp5-inventory-diff`에 alignment mode를 추가했다.

새 모드:

```text
--align index
--align lcs
```

`index`는 Stage 3 방식이고, `lcs`는 stream별 record sequence를 structural signature 기준으로
정렬한다.

## 2. 구현 내용

수정한 파일:

```text
src/diagnostics/hwp5_inventory_diff.rs
src/main.rs
```

CLI:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  [--align index|lcs] [--format jsonl|md] [--section N] [--out <path>]
```

기본값은 기존 동작을 유지하기 위해 `index`다.

## 3. LCS alignment 방식

Stage 4의 `lcs`는 stream 단위로 계산한다.

P0 structural signature:

```text
tag_id
tuple_role
control_id
```

이 signature가 같은 record를 LCS anchor로 삼는다.
payload hash와 size는 alignment 이후 changed field로 분리한다.

LCS mode에서 출력 item에는 다음 필드가 포함된다.

```text
align_mode
alignment_status
oracle_record_index
generated_record_index
oracle_record_uid
generated_record_uid
signature
changed_fields
```

## 4. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage4/hwpx-h-01/inventory.diff.index.jsonl
output/poc/hwpx2hwp/task949/stage4/hwpx-h-01/inventory.diff.index.md
output/poc/hwpx2hwp/task949/stage4/hwpx-h-01/inventory.diff.lcs.jsonl
output/poc/hwpx2hwp/task949/stage4/hwpx-h-01/inventory.diff.lcs.md
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

line count:

```text
inventory.diff.index.jsonl 30106
inventory.diff.index.md    30182
inventory.diff.lcs.jsonl    3890
inventory.diff.lcs.md       3991
```

Stage 3/index mode summary:

```text
matched 520
changed 30099
missing 7
extra   0

control_changed 350
missing         7
payload_changed 7968
scope_changed   7069
size_changed    7646
tag_changed     7066
```

Stage 4/LCS mode summary:

```text
matched 4607
changed 3883
missing 7
extra   0

changed 3883
missing 7
```

따라서 P0 LCS alignment는 diff row를 다음과 같이 줄였다.

```text
30106 -> 3890
```

## 6. Tuple Anchor Summary 관찰

LCS mode tuple summary:

```text
docinfo         changed 521 / missing 4
para_header     changed 1582
list_header     changed 1453
para_char_shape changed 159
para_line_seg   changed 93
table           changed 26
ctrl_header Table changed 26
pic             changed 5
shape_component changed 5
ctrl_data       missing 2
forbidden_char  missing 1
```

이 결과는 Stage 3의 대량 `tag_changed` noise를 줄이고, 실제로 payload/size가 달라진 tuple role을
더 직접적으로 보여준다.

## 7. Markdown 추가 섹션

Stage 4 Markdown에는 다음 섹션이 추가되었다.

```text
Alignment Summary
Tuple Anchor Summary
Divergence Blocks
Diff Rows
```

`Divergence Blocks`는 현재 상위 50개 구간만 출력한다.

## 8. 해석

Stage 4는 record index가 밀린 뒤 발생하는 연쇄 `tag_changed` noise를 상당히 줄였다.

특히 다음 변화가 의미 있다.

```text
index mode:
  tag_changed 7066
  scope_changed 7069

lcs mode:
  tag_changed/scope_changed를 독립 diff row로 만들지 않고,
  structural match 이후 size/payload 중심 changed row로 압축
```

다만 LCS mode도 아직 최종 contract 판정기는 아니다.

주의:

```text
1. structural signature가 같으면 payload가 완전히 달라도 같은 record로 align된다.
2. 중복 signature가 많은 문단/셀 영역에서는 LCS anchor가 사람이 기대한 tuple과 다를 수 있다.
3. scope_path는 index shift noise가 커서 LCS changed_fields에서는 제외했다.
4. TABLE / PIC / SHAPE 묶음을 하나의 semantic tuple로 비교하지는 않는다.
```

## 9. 다음 단계

Stage 5에서는 LCS 결과를 이용해 사람이 바로 볼 수 있는 contract 후보 문서를 만든다.

후보 산출물:

```text
output/poc/hwpx2hwp/task949/stage5/hwpx-h-01/contract_violation_hints.md
```

포함할 내용:

```text
1. missing record 후보
2. TABLE / PIC / SHAPE_COMPONENT 변경 후보
3. DocInfo count/id mapping 변경 후보
4. payload changed가 많은 record type 순위
5. 한컴 판정과 연결할 probe 후보 목록
```
