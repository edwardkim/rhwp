# Task m100 #949 Stage 12 작업 보고서: h03 contract graph 분석 전환

## 1. 목적

Stage 11의 graft probe 판정 방식을 중단하고, `hwpx-h-03`의 한컴 파일손상 원인을
정답 HWP와 generated HWP의 record/control contract 차이로 설명한다.

이번 단계에서는 HWP 파일을 추가 생성하지 않았다. 작업지시자 시각 판정도 요청하지 않았다.

## 2. 입력

```text
HWPX source:
samples/hwpx/hwpx-h-03.hwpx

oracle HWP:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated HWP:
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp
```

## 3. 구현

새 진단 명령을 추가했다.

```text
rhwp hwp5-contract-analyze <source.hwpx> <oracle.hwp> <generated.hwp> --out-dir <dir>
```

소스:

```text
src/diagnostics/hwp5_contract_analyze.rs
src/diagnostics/mod.rs
src/main.rs
```

역할:

```text
- HWPX XML 표면 marker count
- HWPX -> IR parse summary
- oracle/generated HWP DocInfo contract 비교
- oracle/generated HWP BodyText control graph 비교
- serializer 책임 경로 후보 표시
```

## 4. 실행 명령

```bash
cargo run --quiet --bin rhwp -- hwp5-contract-analyze \
  samples/hwpx/hwpx-h-03.hwpx \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp \
  --out-dir output/poc/hwpx2hwp/task949/stage12/hwpx-h-03
```

## 5. 산출물

```text
output/poc/hwpx2hwp/task949/stage12/hwpx-h-03/
```

| file | purpose |
|---|---|
| `stage12_index.md` | 보고서 인덱스 |
| `docinfo_contract.md` | DocInfo count/reference contract 분석 |
| `bodytext_control_graph.md` | BodyText control tuple contract 분석 |
| `hwpx_ir_serializer_trace.md` | HWPX XML / IR / serializer 책임 경로 요약 |

## 6. 주요 결과

### 6.1 DocInfo contract

`docinfo_contract.md`에서 확인된 핵심 차이:

```text
MEMO_SHAPE:
  oracle=1
  generated=0

ID_MAPPINGS:
  oracle size=72
  generated size=64

ID_MAPPINGS memo_shape_count(index 15):
  oracle=1
  generated=0
```

추가로 oracle에만 있는 DocInfo record:

```text
DOC_DATA
FORBIDDEN_CHAR
COMPATIBLE_DOCUMENT
LAYOUT_COMPATIBILITY
TRACKCHANGE
MEMO_SHAPE
```

해석:

```text
hwpx-h-03 generated HWP는 DocInfo count/reference table 관점에서
oracle HWP와 다른 contract를 갖는다. 특히 MEMO_SHAPE는 record 누락과
ID_MAPPINGS count 누락이 함께 발생한다.
```

### 6.2 BodyText control graph

`bodytext_control_graph.md`에서 확인된 핵심 차이:

```text
CTRL_DATA:
  oracle=1
  generated=0
```

차이가 집중된 control:

```text
/BodyText/Section0:GenShape#1
  oracle child: CTRL_DATA 포함
  generated child: CTRL_DATA 누락
  CTRL_HEADER size: oracle=60, generated=46

/BodyText/Section0:GenShape#2
  oracle child: CTRL_DATA 포함
  generated child: CTRL_DATA 누락
```

그 외:

```text
SectionDef CTRL_HEADER payload size 차이:
  oracle=47
  generated=28
```

해석:

```text
hwpx-h-03의 BodyText 차이는 TABLE 개수나 control 개수 차이가 아니다.
control type count는 oracle/generated가 같다. 문제는 특정 GenShape tuple 내부의
CTRL_DATA와 CTRL_HEADER payload contract가 generated 쪽에서 불완전하다는 점이다.
```

### 6.3 HWPX / IR trace

`hwpx_ir_serializer_trace.md`에서 확인된 표면 count:

```text
HWPX:
  section xml=2
  bindata files=3
  hp:tbl=26
  hp:pic=3
  hp:lineSegArray=0

IR:
  sections=2
  tables=26
  pictures=3
  shapes=1
  bin_data_items=3
  bin_data_payloads=3
  memo_shape_count=0
```

해석:

```text
HWPX는 렌더링 가능한 의미 정보를 제공하지만, HWP5에 필요한 모든 record contract를
명시적으로 제공하지 않는다. 특히 lineSegArray가 없고, DocInfo MEMO_SHAPE 계열도
IR에 들어오지 않는다.
```

## 7. 다음 구현 후보

Stage 12 기준으로 다음 구현 후보는 "probe 성공 조합"이 아니라 contract 단위로 잡는다.

```text
1. DocInfo MEMO_SHAPE / ID_MAPPINGS count contract
   - source 후보: src/parser/hwpx/header.rs
   - source 후보: src/serializer/doc_info.rs

2. GenShape CTRL_HEADER payload / CTRL_DATA tuple contract
   - source 후보: src/parser/hwpx/section.rs
   - source 후보: src/serializer/control.rs

3. SectionDef CTRL_HEADER payload tail contract
   - source 후보: src/parser/hwpx/section.rs
   - source 후보: src/serializer/control.rs
```

우선순위는 다음과 같다.

```text
1. GenShape tuple의 CTRL_DATA/CTRL_HEADER payload contract를 먼저 추적한다.
2. DocInfo MEMO_SHAPE는 HWPX 원본에 의미 정보가 있는지 별도 확인 후 처리한다.
3. SectionDef tail은 h01 성공 케이스에도 차이가 남았던 축이므로 직접 원인으로 단정하지 않는다.
```

## 8. 검증

```text
cargo check: 통과
hwp5-contract-analyze: 보고서 4개 생성 통과
HWP probe 생성: 없음
작업지시자 시각 판정 요청: 없음
```
