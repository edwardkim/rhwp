# Task m100 #949 Stage 12 계획: h03 contract graph 분석 전환

## 1. 목적

Stage 11의 graft 판정 접근을 중단하고, `hwpx-h-03`의 한컴 파일손상 원인을
정답 HWP와 generated HWP의 구조 차이로 설명한다.

이번 단계의 목표는 HWP 파일을 추가 생성해 시각 판정을 요청하는 것이 아니다.
먼저 다음 질문에 답한다.

```text
1. 한컴 정답 HWP에는 있지만 generated HWP에는 없는 record/control contract는 무엇인가?
2. 그 차이는 HWPX 원본, IR, HWP serializer 중 어느 단계에서 사라지는가?
3. 한컴 에디터가 거부할 수 있는 count, reference, parent/child, payload size 계약 위반은
   어디에서 발생하는가?
4. 구현해야 할 단위는 어떤 control/record graph인가?
```

## 2. 원칙

```text
- 작업지시자 판정은 이번 단계에서 요청하지 않는다.
- probe HWP를 먼저 만들지 않는다.
- 정답 HWP record를 generated HWP에 부분 graft하는 방식은 사용하지 않는다.
- 성공/실패 파일을 더 늘리지 않고, 기존 oracle/generated 쌍의 구조 차이를 설명한다.
- 구현 후보는 "정확한 contract 재구성 단위"로만 정의한다.
```

## 3. 입력

```text
HWPX source:
samples/hwpx/hwpx-h-03.hwpx

oracle HWP:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated HWP:
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp
```

## 4. 분석 축

### 4.1 DocInfo contract

확인 대상:

```text
- ID_MAPPINGS count/size/reference
- MEMO_SHAPE 존재 여부와 참조 관계
- BinData, FaceName, BorderFill, CharShape, ParaShape index 안정성
- DocInfo record 순서와 section/body record가 참조하는 id 범위
```

산출:

```text
output/poc/hwpx2hwp/task949/stage12/hwpx-h-03/docinfo_contract.md
```

### 4.2 BodyText control graph

확인 대상:

```text
- CTRL_HEADER 다음에 요구되는 payload record 묶음
- TABLE control tuple
- SHAPE_COMPONENT / SHAPE_PICTURE tuple
- CTRL_DATA 존재 조건
- LIST_HEADER / PARA_HEADER / PARA_TEXT / PARA_LINE_SEG 연결
- 문단 종료와 다음 control 시작 위치
```

산출:

```text
output/poc/hwpx2hwp/task949/stage12/hwpx-h-03/bodytext_control_graph.md
```

### 4.3 HWPX -> IR trace

확인 대상:

```text
- HWPX 원본에 해당 control의 의미 정보가 존재하는지
- rhwp IR에는 control이 보존되는지
- HWP serializer에서 record가 누락되는지
- rhwp-studio 렌더러가 계산값으로 보완해서 보이는 것인지
```

산출:

```text
output/poc/hwpx2hwp/task949/stage12/hwpx-h-03/hwpx_ir_serializer_trace.md
```

## 5. 도구 개선 방향

현재 필요한 것은 HWP 파일 생성기가 아니라 contract 설명기다.

도구가 출력해야 할 항목:

```text
- oracle/generated record alignment
- tag, size, payload hash
- parent control id
- child record span
- referenced DocInfo id
- missing record가 속한 control tuple
- HWPX source XPath 또는 IR node 경로
- serializer source path 후보
```

가능한 명령 형태:

```bash
cargo run --quiet --bin rhwp -- hwp5-contract-analyze \
  samples/hwpx/hwpx-h-03.hwpx \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp \
  --out-dir output/poc/hwpx2hwp/task949/stage12/hwpx-h-03
```

## 6. 완료 조건

Stage 12 완료 조건은 다음이다.

```text
1. hwpx-h-03 파일손상 후보를 "record 이름"이 아니라 "contract graph 위반"으로 설명한다.
2. 각 위반 후보에 대해 HWPX source, IR, serializer 중 어느 단계의 책임인지 표시한다.
3. 다음 구현 단계에서 수정할 소스 경로와 record/control 단위를 명시한다.
4. 작업지시자 판정이 필요한 경우, 그 판정이 어떤 구현 결정을 바꾸는지 먼저 문서화한다.
```

## 7. 금지 사항

```text
- 성공할 때까지 graft 조합을 늘리는 방식
- 독립적이지 않은 record를 독립 축처럼 판정하는 방식
- rhwp-studio reload 성공을 한컴 호환 성공으로 해석하는 방식
- 한컴 에디터 시각 판정을 원인 분석의 대체재로 사용하는 방식
```
