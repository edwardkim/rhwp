# Task m100 #949 Stage 13 작업 보고서: h03 contract source trace

## 1. 목적

Stage 12에서 확인한 `hwpx-h-03`의 차이를 소스 책임 경로로 연결했다.

이번 단계에서는 HWP 파일을 새로 만들지 않았다. 작업지시자 시각 판정도 요청하지 않았다.
생성된 것은 oracle/generated HWP의 Section0 inventory와 그 분석 문서뿐이다.

## 2. 입력

```text
HWPX source:
samples/hwpx/hwpx-h-03.hwpx

oracle HWP:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated HWP:
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp
```

## 3. 보조 산출물

```text
output/poc/hwpx2hwp/task949/stage13/hwpx-h-03/oracle_section0_inventory.jsonl
output/poc/hwpx2hwp/task949/stage13/hwpx-h-03/generated_section0_inventory.jsonl
output/poc/hwpx2hwp/task949/stage13/hwpx-h-03/source_contract_trace.md
```

inventory는 구현 책임 추적용이며, 작업지시자 판정 대상 HWP가 아니다.

## 4. Stage 12에서 확정된 사실

### 4.1 BodyText control count는 원인이 아니다

```text
CTRL_HEADER: oracle=207, generated=207
TABLE: oracle=26, generated=26
SHAPE_COMPONENT: oracle=4, generated=4
SHAPE_PICTURE: oracle=3, generated=3
```

`hwpx-h-03`의 파일손상은 컨트롤 개수 누락 문제가 아니다.

### 4.2 차이는 특정 GenShape tuple 내부 contract에 집중된다

```text
CTRL_DATA:
  oracle=1
  generated=0
```

Stage 13 inventory에서 oracle의 누락 record는 다음 위치로 좁혀졌다.

```text
BodyText/Section0/PARA_HEADER#820
  /CTRL_HEADER#824
  /SHAPE_COMPONENT#825
  /PARA_HEADER#827
  /CTRL_HEADER#831
  /SHAPE_COMPONENT#832
  /CTRL_DATA#833
```

해석:

```text
문제는 "모든 그림에 CTRL_DATA를 붙이면 된다"가 아니다.
중첩된 GenShape/Picture tuple에서 HWP5가 요구하는 CTRL_DATA contract가 generated
경로에서 materialize되지 않은 것이다.
```

### 4.3 DocInfo에도 count/reference 차이가 있다

```text
MEMO_SHAPE:
  oracle=1
  generated=0

ID_MAPPINGS memo_shape_count:
  oracle=1
  generated=0
```

이 차이는 확정된 contract 차이지만, `hwpx-h-03` 파일손상의 직접 원인으로는 아직
확정하지 않는다. `MEMO_SHAPE`는 count만 올리면 되는 필드가 아니라 record payload와
ID_MAPPINGS가 함께 맞아야 하는 DocInfo contract이기 때문이다.

## 5. 소스 책임 경로

| layer | source | 확인 내용 | 결론 |
|---|---|---|---|
| HWPX parser | `src/parser/hwpx/section.rs` | `<hp:tbl>`, `<hp:pic>`, `<hp:container>`를 `Control`로 만든다. | 의미 컨트롤은 생성된다. |
| HWPX parser | `src/parser/hwpx/section.rs` | HWP5 `CTRL_DATA`에 대응하는 `para.ctrl_data_records`를 채우지 않는다. | HWP5 binary contract는 parser 단계에서 생성되지 않는다. |
| HWP parser | `src/parser/body_text.rs` | HWP `CTRL_DATA`를 읽으면 `para.ctrl_data_records`에 저장한다. | HWP round-trip은 원본 `CTRL_DATA`를 보존할 수 있다. |
| BodyText serializer | `src/serializer/body_text.rs` | `para.ctrl_data_records[ctrl_idx]`가 있어야 control serializer에 전달한다. | IR에 없으면 `CTRL_DATA`는 출력되지 않는다. |
| Control serializer | `src/serializer/control.rs` | 그림 control에서 `ctrl_data_record`가 있으면 `SHAPE_COMPONENT` 자식으로 `CTRL_DATA`를 쓴다. | serializer는 이미 출력 경로를 갖고 있다. |
| HWPX -> HWP adapter | `src/document_core/converters/hwpx_to_hwp.rs` | 현재 `adapt_paragraph`는 `Control::Table`만 보강한다. | 그림/그룹 GenShape contract 보강이 없다. |
| DocInfo serializer | `src/serializer/doc_info.rs` | `memo_shape_count`를 `ID_MAPPINGS`에 기록한다. | 모델 값이 0이면 generated도 0이 된다. |

## 6. 핵심 결론

현재 실패는 serializer가 `CTRL_DATA`를 못 쓰는 문제가 아니다.

```text
HWPX parser:
  의미 컨트롤 생성
  ctrl_data_records 미생성

HWPX -> HWP adapter:
  table contract만 materialize
  shape/picture/group contract는 미구현

serializer:
  ctrl_data_records가 있으면 CTRL_DATA 출력 가능
```

따라서 다음 구현 위치는 serializer가 아니라 `src/document_core/converters/hwpx_to_hwp.rs`다.
이 파일에서 HWPX 출처 IR을 HWP5 저장 contract로 materialize해야 한다.

## 7. Stage 14 구현 타깃

### 7.1 우선순위 1: GenShape tuple contract materializer

목표:

```text
HWPX 출처의 Picture/Shape/Group control을 HWP5 serializer가 요구하는 GenShape tuple로
정확히 materialize한다.
```

구현 위치:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

작업:

```text
1. adapt_paragraph가 controls와 ctrl_data_records를 같은 index contract로 다루게 한다.
2. table뿐 아니라 Picture/Shape/Group을 순회한다.
3. nested paragraph가 있는 shape/group 내부도 재귀 순회한다.
4. oracle의 CTRL_DATA#833 payload 의미를 먼저 decode한 뒤 필요한 경우에만 합성한다.
```

주의:

```text
CTRL_DATA를 모든 picture에 일괄 삽입하지 않는다.
Stage 13의 증거는 특정 nested tuple에서 누락이 발생했다는 것이다.
합성 조건은 oracle payload 해석 후 정한다.
```

### 7.2 우선순위 2: DocInfo MEMO_SHAPE contract

목표:

```text
oracle에 존재하는 MEMO_SHAPE record와 ID_MAPPINGS memo_shape_count 차이를 별도 contract로
분석한다.
```

주의:

```text
memo_shape_count만 1로 올리면 안 된다.
MEMO_SHAPE record payload를 함께 만들거나, HWPX에서 대응 의미 정보를 찾은 뒤
DocInfo record + ID_MAPPINGS를 동시에 materialize해야 한다.
```

### 7.3 우선순위 3: SectionDef CTRL_HEADER tail

Stage 12에서 SectionDef payload 차이가 보였지만, 이전 성공 케이스에도 SectionDef 차이가
남아 있었다. 따라서 직접 원인으로 단정하지 않고 후순위로 둔다.

## 8. 이번 단계에서 하지 않은 일

```text
- HWP probe 생성
- 한컴 에디터 시각 판정 요청
- raw stream graft
- serializer를 특정 샘플에 맞춰 특수화
```

## 9. 다음 단계

Stage 14는 새 시각 판정 probe가 아니라 소스 구현 단계로 진행한다.

```text
1. oracle CTRL_DATA#833 payload를 decode할 수 있는 targeted diagnostic을 추가한다.
2. HWPX 출처 Picture/Shape/Group control에 대한 ctrl_data_records materialization 조건을 정의한다.
3. `hwpx_to_hwp.rs`에 GenShape contract materializer를 구현한다.
4. 기존 h01/h03 oracle 비교 보고서로 record contract가 회복되는지 먼저 확인한다.
5. 이후에만 작업지시자 판정을 요청한다.
```
