# Task M100 #949 Stage 29 계획서 - h03 outer GenShape 구조체 매핑 추적

## 1. 배경

Stage 28에서 HWPX `<hp:renderingInfo>`를 HWP5 `SHAPE_COMPONENT.raw_rendering`으로
materialize했다.

구조적으로 확인된 개선:

```text
hy-001 SHAPE_COMPONENT#21 size: 196 -> 292 bytes
hwpx-h-03 child picture SHAPE_COMPONENT#832/#836 size: 196 bytes 유지
hwpx-h-03 CTRL_DATA#833 payload: 정답지와 일치 유지
```

하지만 작업지시자 한컴 판정은 Stage 23/26과 동일했다.

```text
hwpx-h-01: 성공
hwpx-h-02: 성공
hwpx-h-03: 파일손상, 2페이지 글상자 전까지만 출력
hy-001   : 파일손상, 2페이지 마지막 표 전까지만 출력
```

따라서 Stage 28 변경은 필요한 축일 수 있지만 충분조건은 아니다.

## 2. 이번 단계의 문제 정의

이전 실패 패턴은 “후보 필드를 붙여서 한컴에 열어보는 방식”이었다. 이 방식은 다음 한계가 있다.

```text
1. 성공한 guard를 다시 깨뜨릴 수 있다.
2. 실패했을 때 원인 필드가 아니라 조합 실패만 남는다.
3. 한컴이 내부 C++ 구조체에 XML 요소를 매핑한 뒤 serialize한다는 관점과 맞지 않는다.
```

따라서 Stage 29는 구현 후보를 만들지 않는다. 먼저 `hwpx-h-03`에서 한컴이 멈추는 지점의
outer GenShape record bundle을 정답 HWP, 생성 HWP, HWPX source, IR 사이에서 field 단위로 연결한다.

## 3. 비교 기준점

주 기준점은 `hwpx-h-03` 2페이지 글상자 직전/주변 record다.

| idx | record | 현재 상태 |
|---:|---|---|
| 824 | `CTRL_HEADER GenShape` | size 동일, payload 다름 |
| 825 | `SHAPE_COMPONENT` | size 동일, payload 다름 |
| 826 | `LIST_HEADER` | Stage 23에서는 정답 size/hash 일치 가능했으나 파일손상 해소 실패 |
| 827 | `PARA_HEADER` | Stage 23에서는 정답 size/hash 일치 가능했으나 파일손상 해소 실패 |
| 831 | `CTRL_HEADER GenShape` | size 동일, payload 다름 |
| 832 | `SHAPE_COMPONENT` | size 동일, payload 다름 |
| 833 | `CTRL_DATA` | 정답 payload 일치 |
| 834 | `SHAPE_PICTURE` | size 동일, payload 다름 |
| 835 | `CTRL_HEADER GenShape` | size 동일, payload 다름 |
| 836 | `SHAPE_COMPONENT` | size 동일, payload 다름 |
| 837 | `SHAPE_PICTURE` | size 동일, payload 다름 |
| 838 | `SHAPE_RECTANGLE` | payload 일치 |

보조 기준점은 `hy-001` 글상자 안 그림이다.

| idx | record | 현재 상태 |
|---:|---|---|
| 20 | `CTRL_HEADER GenShape` | size 동일, payload 다름 |
| 21 | `SHAPE_COMPONENT` | size 292 회복, payload 다름 |
| 22 | `SHAPE_PICTURE` | size 동일, payload 다름 |
| 23 | `LIST_HEADER` | size 다름 |
| 24 | `PARA_HEADER` | size 다름 |

## 4. 산출물

출력 위치:

```text
output/poc/hwpx2hwp/task949/stage29_outer_genshape_field_map/
```

생성 문서:

```text
h03_outer_genshape_field_map.md
h03_outer_genshape_xml_trace.md
h03_outer_genshape_ir_trace.md
hy001_textbox_field_map.md
stage29_findings.md
```

작업 보고서:

```text
mydocs/working/task_m100_949_stage29.md
```

## 5. 작업 절차

### 5.1 HWP5 record field decode

정답 HWP와 Stage 28 생성 HWP에서 다음 record를 field 단위로 해독한다.

```text
CTRL_HEADER GenShape
SHAPE_COMPONENT
SHAPE_PICTURE
SHAPE_RECTANGLE
LIST_HEADER
PARA_HEADER
```

해독 단위:

```text
offset
field name
field size
oracle value
generated value
same/different
known meaning
source 후보
```

### 5.2 HWPX source trace

`samples/hwpx/hwpx-h-03.hwpx`에서 문제 지점의 XML subtree를 추적한다.

추적 대상:

```text
hp:tbl
hp:p / hp:run
hp:rect
hp:drawText
hp:pic
hp:pos
hp:sz
hp:outMargin
hp:renderingInfo
```

목표는 XML 요소가 존재하는지 여부가 아니라, XML 값이 한컴 HWP5 구조체 field에 어느 방식으로
들어가는지 후보를 세우는 것이다.

### 5.3 IR trace

HWPX parser 결과 IR에서 같은 객체가 어떤 필드로 보존되는지 확인한다.

분류:

```text
1. XML에는 있고 IR에도 있는 값
2. XML에는 있으나 IR에서 사라진 값
3. IR에는 있으나 HWP5 writer가 쓰지 않는 값
4. XML에는 없고 정답 HWP에는 존재하는 한컴 계산값
```

### 5.4 원인 후보 분류

각 payload 차이를 다음 범주로 분류한다.

```text
A. parser 누락
B. IR 모델 필드 누락
C. writer serialization 누락
D. 한컴 조판 계산값 미구현
E. 정답 HWP 출처 고유 raw payload 보존 필요
F. 현재 문서와 무관한 허용 차이
```

## 6. 성공 기준

Stage 29 성공 기준은 한컴 파일손상 해소가 아니다. 이번 단계는 구현 전 분석 단계다.

성공 기준:

```text
1. h03 #824/#825/#831/#832/#834/#835/#836/#837 payload diff가 field 단위 표로 분해됨
2. 각 diff가 XML/IR/writer/computed/raw 중 하나로 분류됨
3. 다음 구현 후보가 단일 contract 단위로 정의됨
4. h01/h02 성공 guard를 깨지 않을 적용 조건이 제시됨
```

실패 기준:

```text
1. payload hash 다름 수준에서 멈춤
2. offset 의미를 알 수 없는 상태에서 임의 후보 파일을 생성함
3. h03과 hy-001을 구분하지 않고 TextBox 전체에 generic 보정을 제안함
```

## 7. 다음 단계 조건

Stage 29 결과 후에만 Stage 30 구현 계획을 작성한다.

Stage 30 후보는 다음 형식이어야 한다.

```text
HWPX source 조건
IR 보존 필드
HWP5 target record
target offset/bit/size
guard 문서
expected visual result
rollback 조건
```

승인 후 Stage 29 진단을 진행한다.
