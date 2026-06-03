# Task m100 #903 Stage 41 계획

## 1. 목적

Stage40에서 TABLE 최소 후보가 다음으로 좁혀졌다.

```text
필수 TABLE index:
48,103,286,433,563,742,1619,2944,6466

제외 가능 TABLE index:
819,6596,6986,7376
```

Stage41은 필수 TABLE과 제외 가능 TABLE의 byte-level payload 차이를 비교하여,
구현해야 할 serializer/parser 필드를 추적한다.

## 2. 기준 파일

실패 baseline:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
```

성공 raw source:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

Stage40 최소 성공 후보:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/02_base_without_819.hwp
```

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage41_table_payload_diff/
```

## 3. 분석 대상

필수 TABLE:

```text
48,103,286,433,563,742,1619,2944,6466
```

제외 가능 TABLE:

```text
819,6596,6986,7376
```

## 4. 생성 리포트

Stage41은 우선 한컴 판정 파일을 만들지 않고, byte diff 리포트를 만든다.

```text
output/poc/hwpx2hwp/task903/stage41_table_payload_diff/table_payload_diff.md
```

리포트 항목:

```text
1. record index
2. 필수/선택 분류
3. record level
4. target payload size
5. positive payload size
6. target payload hex
7. positive payload hex
8. first data diff
9. positive가 target prefix인지 여부
10. positive extra tail bytes
11. decoded table model summary
```

## 5. 분석 질문

### 5.1 TABLE tail 2 bytes

대부분 TABLE diff는 다음 패턴이다.

```text
target size = positive size - 2
```

확인할 것:

```text
- positive payload의 마지막 2 bytes가 공통 의미를 가지는지
- target payload가 positive payload의 prefix인지
- 필수 TABLE과 제외 가능 TABLE의 tail 값이 다른지
```

### 5.2 idx 48 data-only diff

idx 48은 size가 같고 data만 다르다.

```text
target size = 22
positive size = 22
first diff at payload offset 18:
  target 0x0b
  positive 0x01
```

확인할 것:

```text
- payload offset 18이 어떤 TABLE field인지
- 이 field가 table valid zone, cell count tail, border/fill ref, unknown flags 중 무엇인지
```

### 5.3 필수/선택 TABLE 차이

확인할 것:

```text
- 제외 가능 TABLE index도 동일한 +2 tail diff인지
- 같은 +2 tail diff인데도 선택 가능한 이유가 문서 위치 때문인지
- 필수 TABLE만 특별한 field 조합을 가지는지
```

## 6. 코드 조사 범위

리포트 생성 후 다음 소스를 대조한다.

```text
src/parser/*table*
src/serializer/*table*
src/model/table*
src/parser/hwpx/*table*
```

확인할 것:

```text
1. HWP TABLE record serializer가 쓰는 payload 구성
2. HWP parser가 읽되 serializer가 버리는 TABLE tail field
3. HWPX parser에서 table 관련 field 누락 여부
4. Stage30 positive의 TABLE raw payload가 모델 필드로 이미 보존되는지 여부
```

## 7. 산출물

작업 기록:

```text
mydocs/working/task_m100_903_stage41.md
```

분석 리포트:

```text
output/poc/hwpx2hwp/task903/stage41_table_payload_diff/table_payload_diff.md
```

## 8. 기대 해석

```text
target payload가 positive prefix이고 extra tail 2 bytes가 공통:
  TABLE serializer에 누락 tail field가 있다.

extra tail 2 bytes가 index별로 다름:
  table model에 보존해야 할 per-table field가 있다.

idx 48 offset 18이 기존 model field와 대응:
  HWPX parser 또는 adapter가 해당 field를 잘못 매핑한다.

idx 48 offset 18이 미모델링 field:
  TABLE raw tail/unknown field 보존 전략이 필요하다.
```

## 9. 하지 않을 것

```text
- 아직 TABLE serializer를 수정하지 않는다.
- TABLE raw payload를 전체 보존하는 방식으로 바로 구현하지 않는다.
- SHAPE payload 분석과 섞지 않는다.
```
