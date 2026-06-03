# Task M100-1099 Stage 1 작업 기록

## 1. 목적

`exam-kor-1p.hwpx`를 현재 HWP 저장기로 저장했을 때 한컴 에디터가 파일손상으로 판정하는
원인을 1페이지 축소 샘플 기준으로 좁힌다.

이번 단계는 판정용 변종을 많이 만드는 단계가 아니다. 정답 HWP와 생성 HWP의 HWP5 record
contract 차이를 먼저 압축하는 단계다.

## 2. 입력과 산출물

입력:

```text
source HWPX: samples/hwpx/exam-kor-1p.hwpx
oracle HWP : samples/exam-kor-1p.hwp
generated  : output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/exam-kor-1p-generated.hwp
```

생성 파일:

```text
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/exam-kor-1p-generated.hwp
```

파일 크기:

| file | size |
|---|---:|
| `samples/exam-kor-1p.hwp` | 976,384 bytes |
| `samples/hwpx/exam-kor-1p.hwpx` | 885,009 bytes |
| `output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/exam-kor-1p-generated.hwp` | 683,008 bytes |

rhwp reload 기준:

```text
oracle    : ok, pages=1
generated : ok, pages=1
```

주의:

```text
rhwp reload 성공은 한컴 에디터 정상 로딩의 충분 조건이 아니다.
이번 이슈의 성공 기준은 한컴 에디터 파일손상 해소다.
```

## 3. 생성한 분석 산출물

```text
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/oracle.section0.inventory.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/generated.section0.inventory.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/record_tree_diff.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/contract_violation_hints.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/record_bundles.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/table_field_diff.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/table_probe_plan.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/ctrl_bundles.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/docinfo_diff.md
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/contract_graph/
```

## 4. 전체 diff 요약

`contract_violation_hints.md` 기준:

```text
diff_count = 363
matched    = 243
changed    = 247
missing    = 60
extra      = 56
```

상위 차이 영역:

| area | 핵심 |
|---|---|
| DocInfo | 정답지에만 `FORBIDDEN_CHAR`, `COMPATIBLE_DOCUMENT`, `LAYOUT_COMPATIBILITY`, `TRACKCHANGE` 존재 |
| Table tuple | 일부 `CTRL_HEADER(Table)` / `TABLE` payload 크기와 parent-child graph 차이 |
| SectionDef subtree | 정답지보다 generated 쪽 SectionDef 자식 record가 과다하게 물림 |
| ParaShape / FaceName | count는 같지만 payload 크기와 내용 차이가 큼 |
| Shape tuple | GenShape / Picture / Polygon tuple의 payload와 배치 차이 |

## 5. 가장 강한 파일손상 후보

### 후보 1: `CTRL_HEADER(Table)` 축약 payload

`table_field_diff.md`에서 가장 구조적인 차이가 확인된다.

```text
BodyText.Section0#101~BodyText.Section0#157

oracle:
  CTRL_HEADER(Table) size = 46

generated:
  CTRL_HEADER(Table) size = 4
```

같은 위치의 `TABLE` record도 차이가 있다.

```text
BodyText.Section0#102~BodyText.Section0#158

oracle:
  TABLE size = 24
  row_count_hint = 1
  tail_after_0x16 = 00 00

generated:
  TABLE size = 22
  row_count_hint = 1428
  tail_after_0x16 = 없음
```

해석:

```text
generated는 특정 table tuple에서 control wrapper를 4바이트 ctrl id만 저장하고 있다.
정답지는 같은 table tuple을 46바이트 CTRL_HEADER payload와 24바이트 TABLE payload로 저장한다.
이 차이는 단순 조판 차이가 아니라 HWP5 record contract 차이로 보아야 한다.
```

### 후보 2: SectionDef 아래 master page / footer subtree materialization 차이

`bodytext_control_graph.md` 기준:

```text
SectionDef#0

oracle:
  CTRL_HEADER=12
  LIST_HEADER=10
  PARA_HEADER=12
  PARA_TEXT=11
  SHAPE_COMPONENT=6
  TABLE=2

generated:
  CTRL_HEADER=19
  LIST_HEADER=16
  PARA_HEADER=21
  PARA_TEXT=19
  SHAPE_COMPONENT=9
  TABLE=4
```

해석:

```text
HWPX 바탕쪽/머리말/꼬리말/본문 주변 컨트롤을 HWP5 SectionDef subtree에 materialize하는 방식이
정답지와 다르다. 단순히 control count가 부족한 문제가 아니라, 같은 1페이지 안에서 record graph가
다르게 구성된다.
```

### 후보 3: DocInfo compatibility 계열 누락

`docinfo_contract.md` 기준 정답지에만 존재:

```text
FORBIDDEN_CHAR
COMPATIBLE_DOCUMENT
LAYOUT_COMPATIBILITY
TRACKCHANGE
```

또한 count는 같지만 payload가 전부 다른 영역:

```text
FACE_NAME  : 56/56 changed
PARA_SHAPE : 45/45 changed
```

해석:

```text
DocInfo 누락도 HWP5 저장 contract 차이다. 다만 현재 1순위는 Table tuple의 4바이트 CTRL_HEADER와
22바이트 TABLE payload다. 해당 차이는 직접적으로 record parser contract를 깨뜨릴 가능성이 더 높다.
DocInfo compatibility 계열은 Table tuple 수정 후에도 파일손상이 남을 경우 다음 후보로 다룬다.
```

## 6. HWPX/IR/Serializer 관찰

`hwpx_ir_serializer_trace.md` 기준:

```text
HWPX XML:
  hp:p   = 109
  hp:pic = 11
  hp:tbl = 6

IR:
  paragraphs = 39
  table_cells = 2
  controls:
    Table = 2
    Picture = 8
    Shape = 4
    Footer = 1
    SectionDef = 1
```

반면 raw HWP BodyText tag count는 정답지와 generated 모두 다음처럼 같다.

```text
TABLE = 6
CTRL_HEADER = 39
SHAPE_COMPONENT = 21
SHAPE_PICTURE = 11
```

해석:

```text
개수만 보면 저장기가 큰 컨트롤을 누락했다고 단정하기 어렵다.
문제는 HWPX/IR에서 렌더링 가능한 의미 구조를 HWP5의 SectionDef/Footer/Table tuple graph로
materialize할 때, 일부 tuple의 payload 크기와 parent-child 관계가 정답지와 달라지는 점이다.
```

## 7. Stage 2 제안

다음 단계는 1페이지 파일에만 집중한다.

우선순위:

```text
1. `BodyText.Section0#101~#157` 부근의 Table tuple을 정답지와 비교한다.
2. generated에서 `CTRL_HEADER(Table)`이 4바이트로 저장되는 조건을 소스에서 찾는다.
3. 해당 table tuple을 46바이트 CTRL_HEADER + 24바이트 TABLE payload로 저장하는 후보를 만든다.
4. `exam-kor-1p`에서 한컴 에디터 파일손상 해소 여부를 확인한다.
```

보류:

```text
1. 2/3/4페이지 확장 샘플 생성
2. 전체 `exam_kor.hwpx` 저장 검증
3. DocInfo compatibility record 복원
4. FaceName / ParaShape payload 전체 정합화
```

1페이지가 한컴 에디터에서 정상 로딩될 때까지 확장 샘플을 만들지 않는다.

## 8. 판정 요청 파일

현재 한컴 에디터 확인 대상:

```text
output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/exam-kor-1p-generated.hwp
```

| file | 한컴 판정 유형 | 이미지 출력 | 바탕쪽 출력 | 지문 박스 출력 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1099/stage1_1p_contract_trace/exam-kor-1p-generated.hwp` |  |  |  |  |  | Stage 1 baseline |
