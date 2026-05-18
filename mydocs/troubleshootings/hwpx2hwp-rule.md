# HWPX to HWP Hancom Compatibility Rules

## 1. 목적

이 문서는 HWPX를 HWP로 저장할 때 한컴 에디터가 요구하는 암묵적 record contract를
추적하고 유지하기 위한 규칙 문서다.

대상은 다음 종류의 현상이다.

```text
HWPX 원본은 rhwp-studio에서 정상 렌더링됨
HWPX -> IR -> HWP 저장 결과도 rhwp-studio에서는 열림
하지만 한컴 에디터에서는 파일 읽기 오류, 파일 손상, 조판 실패가 발생함
```

따라서 이 문서는 렌더링 규칙 문서가 아니라 저장 호환성 규칙 문서다.
한컴에서 HWPX를 HWP로 저장한 정답 HWP를 oracle로 삼고, rhwp가 생성한 HWP와
record/control 단위로 비교하여 HWPX construct가 어떤 HWP5 record tuple로 lowering되는지
정확한 contract를 확정한다.

## 2. 기본 원칙

### 2.1 포맷 거버넌스 차이

현재까지의 관찰은 HWP와 HWPX의 포맷 거버넌스가 다르다는 가설을 강하게 지지한다.

HWP는 DTP와 워드프로세서의 결합에 가깝다. 문서의 의미 구조뿐 아니라 이미 계산된 조판 상태와
한컴 편집 엔진의 내부 record contract가 파일에 함께 저장된다.

예:

```text
PARA_LINE_SEG
lineSegArray에 대응되는 line height / baseline / text_start
TAC 표의 위치와 크기
표/그림 control의 binary payload
LIST_HEADER / TABLE / PARA_HEADER tail
```

반면 HWPX는 워드프로세서 문서 모델에 더 가까운 선언적 포맷으로 보인다. 문서 구조, 스타일,
객체 관계, 조판 힌트를 저장하지만, HWP에서 엄격하게 저장되던 일부 조판 결과와 binary contract
값은 생략되거나 선택적으로만 존재한다.

따라서 HWPX에서 어떤 값이 없다는 것은 곧바로 파싱 결함을 의미하지 않는다. 그 값이 HWPX 포맷
책임 밖에 있고, 한컴 엔진이 열기/렌더링/저장 시점에 기본값 또는 계산값으로 채우는 값일 수 있다.

이 차이를 다음처럼 표현한다.

```text
HWP:
  문서 의미 + 스타일 + 조판 결과 + HWP5 binary record contract를 저장한다.

HWPX:
  문서 의미 + 스타일 + 객체 관계 + 일부 조판 힌트를 저장한다.
  HWP5 binary record contract는 저장 포맷의 직접 책임이 아닐 수 있다.
```

결론:

```text
HWPX -> HWP 저장은 XML 필드 복사가 아니다.
HWPX construct를 한컴 HWP oracle이 보여주는 HWP5 record/control contract로 lowering하는 작업이다.
```

### 2.2 어려운 target backend를 먼저 푼다

제품 기능만 놓고 보면 HWPX 원본을 편집한 뒤 다시 HWPX로 저장하는 경로가 더 자연스러워 보일 수 있다.
같은 포맷 거버넌스 안에서 의미 구조, 스타일, 객체 관계를 보존하면 되기 때문이다.

하지만 본 프로젝트의 우선순위는 쉬운 저장 경로를 먼저 구현하는 것이 아니다. HWPX -> HWP 변환은
한컴이 HWPX에 명시하지 않은 값들을 어떤 HWP5 record/control contract로 채우는지 밝히는 연구 장치다.

따라서 작업 순서는 다음으로 고정한다.

```text
1. 먼저 어려운 target backend를 푼다.
   HWPX -> IR -> HWP
   HWPX construct -> HWP5 record/control lowering contract를 확정한다.

2. 그 다음 쉬운 same-format backend를 푼다.
   HWPX -> IR -> HWPX
   이미 확정한 의미론과 contract 지식을 바탕으로 round-trip 저장 경계를 정한다.
```

이 순서의 이유:

```text
HWPX -> HWP를 먼저 풀면:
  한컴이 생략 값을 어떻게 보완하는지 알 수 있다.
  HWPX construct의 실제 의미 경계가 선명해진다.
  HWPX 저장에서 무엇을 보존하고 무엇을 canonicalize해야 하는지 판단할 수 있다.

HWPX -> HWPX를 먼저 풀면:
  한컴 내부 lowering contract는 여전히 알 수 없다.
  렌더링은 맞아도 교환성과 한컴 호환성 기준이 흐릴 수 있다.
```

결론:

```text
HWPX round-trip 저장이 제품 기능으로는 더 자연스러울 수 있다.
그러나 포맷 의미론과 한컴 호환성을 확보하기 위해 HWPX -> HWP lowering contract를 먼저 규명한다.
어려운 target backend를 먼저 풀어야 쉬운 same-format 저장의 경계도 정확해진다.
```

### 2.3 장기 프로젝트 운영 원칙

HWPX -> HWP lowering은 단기 버그픽스가 아니라 최소 1년 이상의 장기 프로젝트로 다룬다.
목표는 특정 샘플 하나를 우연히 통과시키는 것이 아니라, 한컴 HWP oracle에서 관찰되는
HWP5 lowering contract map을 축적하는 것이다.

접근 방식은 의도적으로 잘게 쪼갠다.

```text
HWPX -> IR -> HWP 성공에 이르는 경로를 가령, 최소 1000개의 contract unit으로 나눈다.
각 unit은 HWPX construct 하나와 그에 대응되는 HWP5 record/control tuple 하나를 다룬다.
```

각 unit은 다음 형식을 가져야 한다.

```text
construct
oracle tuple
generated tuple
lowering contract
contract violation
Hancom judgement
rule
regression sample
```

운영 원칙:

```text
1. 모든 시도는 문서화한다.
2. 모든 실패는 contract 지식으로 남긴다.
3. 모든 성공은 oracle-derived rule로만 승격한다.
4. 모든 샘플은 regression corpus가 된다.
5. 모든 도구는 다음 사람이 같은 판단을 반복하지 않게 만든다.
```

가장 중요한 원칙:

```text
빠르게 통과시키는 것보다, 다시 틀리지 않게 지식을 남기는 것이 우선이다.
```

따라서 한컴에서 열리는 산출물을 만들었더라도, 그 산출물이 어떤 HWP5 lowering contract를
만족해서 열렸는지 설명하지 못하면 구현 규칙으로 채택하지 않는다.

### 2.4 HWPX는 HWP5 바이너리의 XML 직렬화가 아니다

HWPX는 HWP5 바이너리를 XML로 그대로 직렬화한 형식이 아니다.

HWPX에는 렌더링 엔진이 기본값과 조판 계산을 보완해 화면을 구성할 수 있는 의미/스타일/객체
정보가 들어 있다. 그러나 HWP5 저장에 필요한 binary record payload, tail, count, size,
layout-computed value가 그대로 존재하지 않는 경우가 있다.

대표적으로 다음 값들은 HWPX에 없거나, 있어도 HWP5 record의 정답으로 바로 사용할 수 없다.

```text
CTRL_HEADER payload
LIST_HEADER tail
TABLE record attr/tail
PARA_HEADER extra
PARA_LINE_SEG / lineSegArray
DocInfo ID_MAPPINGS와 각 ID 참조 관계
BinData record와 CFB BinData stream 관계
shape/picture control의 HWP5 payload
section_count, page/section stream 관계
```

한컴 에디터의 정확한 내부 구현은 알 수 없다. 다만 한컴이 변환한 HWP oracle과 HWPX 원본의
차이를 보면, HWPX를 열고 HWP로 저장하는 과정은 단순 필드 복사가 아니라 다음 단계로 이루어진
lowering pipeline에 가깝다고 보아야 한다.

또한 한컴 에디터는 역사적으로 Windows GDI/GDI+ 기반의 Visual Studio C++ 응용 프로그램이라는
구현 맥락을 고려해야 한다. 따라서 이 과정을 순수 함수형 변환기나 XML serializer처럼 가정하면
안 된다. 실제로는 편집기 내부의 상태ful 문서 모델, 화면 조판 캐시, GDI 계측값, legacy HWP5
record writer가 결합된 경로일 가능성이 높다.

이 관점에서는 다음 값들이 XML 파싱 직후가 아니라 편집/조판/저장 경로 중간에서 확정될 수 있다.

```text
폰트 계측 기반 line height / baseline
문단 line segment
TAC 표의 실제 x/y/폭/높이
그림/도형 control의 배치와 clip 값
페이지/섹션 경계와 section_count
LIST_HEADER / TABLE / PARA_HEADER count와 tail
DocInfo ID mapping과 BinData stream 참조
```

```text
1. HWPX XML construct를 한컴 내부 편집 모델로 정규화한다.
2. HWPX에 명시된 의미/스타일/객체 속성을 HWP5 record 값으로 변환한다.
3. HWPX에 없거나 선택적인 값은 내부 기본값 또는 문맥별 기본값으로 보완한다.
4. 조판 과정에서 필요한 layout-computed value를 계산하거나 갱신한다.
5. HWP5 record tree가 요구하는 count, size, level, tail, reference를 재구성한다.
6. DocInfo, BinData, BodyText stream 사이의 ID mapping과 CFB stream 관계를 일관되게 닫는다.
```

따라서 rhwp의 HWPX -> HWP 저장기는 HWPX에 있는 XML 필드를 HWP5 record로 옮기는 수준을
넘어, 한컴 HWP oracle이 보여주는 target record contract를 재현해야 한다.

이 작업은 단순히 기존 구현을 흉내 내는 저장기를 하나 더 만드는 일이 아니다. 2026년 현재의
현대적인 구현 방법론으로 HWPX 의미 모델을 분석하고, 한컴 oracle에서 확인되는 legacy HWP5
binary contract를 명시적인 lowering backend로 재구성하는 작업이다.

rhwp의 목표는 암묵적인 편집기 상태와 legacy writer 동작을 그대로 복제하는 것이 아니라,
그 결과 contract를 관찰 가능한 규칙으로 분해하고, 테스트 가능하고 유지 가능한 형태로 새로
구현하는 것이다.

컴파일러 관점으로 표현하면 다음과 같다.

```text
HWPX parser = frontend
rhwp IR = 렌더링 가능한 중간표현
rhwp renderer = rendering backend
HWP serializer = HWP5 binary backend
한컴 HWP = target ABI / binary contract
```

즉 `HWPX -> IR -> render`가 성공했다는 것은 frontend와 rendering backend가 맞다는 뜻이다.
`HWPX -> HWP`는 별도의 HWP5 target lowering 문제이며, 한컴 HWP oracle은 이 target ABI의
reference로 취급한다.

## 3. HWP 저장이 성공하는 이유와 HWPX 저장이 실패하는 이유

HWP 원본을 열고 IR을 구성한 뒤 편집 후 다시 저장하는 경로가 성공하는 이유는,
IR만으로 HWP를 완전히 새로 만드는 것이 아니기 때문이다.

HWP 파서는 원본 HWP record stream과 raw payload를 보존하고, 저장기는 가능한 한
이를 그대로 재사용한다.

```text
FileHeader.raw_data
DocInfo.raw_stream
Section.raw_stream
Paragraph.raw_header_extra
Table.raw_ctrl_data
Table.raw_table_record_attr
Table.raw_table_record_extra
Cell.raw_list_extra
DocInfo item raw_data
```

반면 HWPX 파서는 XML/ZIP에서 의미 IR을 구성하므로 HWP5 원본 stream이 없다.
따라서 HWPX 출처 IR을 HWP serializer에 넣는 것은 원본 HWP stream을 보존하는 경로와
본질적으로 다르다.

결론:

```text
HWP -> IR -> HWP 저장 성공은 raw contract 보존의 결과다.
HWPX -> IR -> HWP 저장은 HWP5 control/record를 새로 합성해야 하는 문제다.
```

## 4. 한컴 판정 용어

작업지시자 판정은 다음 용어로 구분해서 기록한다.

### 파일 읽기 오류

한컴 에디터가 파일을 읽거나 저장하는 데 오류가 있다고 판정하는 경우다.

다음 contract 위반 여부를 우선 확인한다.

```text
CFB stream 구성 오류
FileHeader / DocInfo / BodyText stream contract 오류
record size/count 불일치
DocInfo ID_MAPPINGS 불일치
BinData record와 실제 BinData stream 불일치
초기 record tree 진입 전의 치명적 구조 오류
```

### 파일 손상

한컴 에디터가 일부 내용을 출력한 뒤 파일 손상을 판정하는 경우다.

다음 contract 위반 여부를 우선 확인한다.

```text
출력 중단 지점 직후 record/control contract 오류
CTRL_HEADER 다음 record type 불일치
LIST_HEADER paragraph count / child record 범위 불일치
TABLE row/cell count, span, tail 불일치
shape/picture control payload 누락
문단 char_count와 PARA_TEXT code unit 수 불일치
```

이 경우 "마지막으로 정상 출력된 위치"가 매우 중요하다. 다음 probe는 그 위치 바로 뒤의
control/record를 정답 HWP와 비교해야 한다.

### 열림 + 조판 실패

한컴이 파일은 열지만 표 배치, 이미지, 셀 텍스트 baseline, 페이지 수가 틀리는 경우다.

다음 contract 위반 여부를 우선 확인한다.

```text
HWPX에 생략된 기본값 미합성
layout-computed value 누락
ParaShape / CharShape / Table attr 일부 누락
object positioning 속성 누락
lineSegArray를 HWP PARA_LINE_SEG로 잘못 사용
```

### rhwp-studio 정상 + 한컴 실패

rhwp-studio 재로드 성공은 필요 조건일 뿐, 한컴 호환의 충분 조건이 아니다.
rhwp 파서는 자체 출력에 관대할 수 있으므로, 최종 판정은 반드시 한컴 에디터 기준으로 한다.

## 5. 실패 유형 분류

### A. Container / Stream Contract

파일 전체 구조와 stream 구성의 문제다.

점검 대상:

```text
FileHeader
DocInfo stream
BodyText/Section stream
BinData stream
compression flag
stream size
section_count
```

관찰 신호:

```text
파일 읽기 오류
파일 크기가 비정상적으로 작거나 큼
rhwp-studio는 열지만 한컴이 초기 로딩에서 실패
```

### B. Record Tree Contract

HWP5 record의 순서, level, parent-child scope 문제다.

점검 대상:

```text
CTRL_HEADER -> concrete control record
LIST_HEADER -> child paragraph records
PARA_HEADER -> PARA_TEXT / PARA_CHAR_SHAPE / PARA_LINE_SEG
TABLE -> CELL/LIST_HEADER/TEXT subtree
SHAPE_COMPONENT / SHAPE_PICTURE subtree
```

관찰 신호:

```text
일부 출력 후 파일 손상
특정 표, 그림, 문단 직후 중단
rhwp-studio는 정상 렌더링
```

### C. Count / Size / Reference Contract

record 내부 count, byte size, ID reference가 실제 payload와 맞지 않는 문제다.

점검 대상:

```text
PARA_HEADER char_count
control mask
LIST_HEADER paragraph count
TABLE row_count / col_count / cell_count / span
DocInfo ID_MAPPINGS
CharShape / ParaShape / BorderFill / BinData reference id
```

관찰 신호:

```text
파일 손상
일부 셀이나 일부 문단 이후 중단
특정 object 삽입 후 다음 record가 밀린 듯 보임
```

### D. DocInfo / BinData Contract

DocInfo record와 실제 CFB stream이 맞지 않는 문제다.

점검 대상:

```text
BIN_DATA record count
BIN_DATA type/path/storage id
CFB BinData/BINxxxx stream 존재 여부
picture control의 bin_data_id
image format 변환 여부
```

관찰 신호:

```text
그림 경로 찾기 대화상자
이미지 미출력
rhwp-studio 이미지 출력 실패 또는 일부 출력
한컴은 열지만 이미지가 빠짐
```

주의:

HWP -> HWP 저장은 원본 BinData stream을 보존한다. HWPX -> HWP 저장은 HWPX package의
binary item을 HWP CFB BinData stream과 DocInfo BIN_DATA record로 새로 매핑해야 한다.

### E. Missing HWP Defaults

HWPX에는 명시되지 않지만 HWP5 record에는 필요한 기본값 문제다.

점검 대상:

```text
CTRL_HEADER attr
TABLE attr / margin / tail
LIST_HEADER extra
ParaShape attr bits
SectionDef 기본 필드
PageBorderFill / PageDef / ColumnDef 기본값
```

관찰 신호:

```text
한컴에서 열리지만 조판이 다름
셀 텍스트가 위로 올라가거나 클리핑됨
표가 종이 왼쪽에 붙음
페이지 수가 다름
```

### F. Layout-computed Values

렌더링이나 조판 이후 계산되어 HWP에 저장되는 값의 문제다.

점검 대상:

```text
PARA_LINE_SEG
line height / baseline
object vpos / hpos
table row height
page break result
```

관찰 신호:

```text
rhwp-studio 렌더링과 저장 HWP 재로드가 다름
한컴 저장 정답 HWP와 generated HWP의 lineSegArray가 다름
HWPX lineSegArray를 그대로 쓰면 표/문단 혼합 케이스가 깨짐
```

## 6. 현재까지 확인한 규칙

### HWPX lineSegArray는 HWP PARA_LINE_SEG 정답이 아니다

#### line segment

line segment는 문단을 실제 종이 위에 놓기 위해 한 줄 단위로 계산한 조판 정보다.
일반적으로 문단은 하나의 텍스트 덩어리처럼 보이지만, 편집기는 이를 여러 줄로 나누고
각 줄마다 다음 정보를 계산해야 한다.

```text
이 줄이 페이지/셀/문단 안에서 시작되는 위치
이 줄의 높이
글자의 baseline 위치
줄 안에서 텍스트가 시작되는 x 좌표
줄 주변에 떠 있는 표/그림/control 때문에 비워야 하는 영역
다음 줄 또는 다음 문단으로 넘어갈 위치
```

즉 line segment는 "문단 텍스트를 어느 줄에 어떤 높이와 기준선으로 배치할 것인가"를
기록한 조판 결과다. 한컴 에디터가 HWP를 열 때 이 값이 맞지 않으면, 글자가 셀 위쪽에
붙거나 잘리고, 표가 문단과 겹치고, 페이지 나눔이 달라질 수 있다.

이 조판 정보를 HWPX와 HWP는 서로 다른 형태로 저장한다.

```text
HWPX lineSegArray:
  XML 문서 안에 들어 있는 줄 단위 조판 정보 배열이다.
  HWPX 문서 모델에서 문단의 줄 배치 결과 또는 힌트를 표현한다.

HWP PARA_LINE_SEG:
  HWP5 BodyText stream 안에 들어 있는 binary record다.
  한컴 HWP 로더가 문단을 해석할 때 사용하는 HWP5 record tree의 일부다.
```

둘 다 "문단을 줄로 나누어 배치한 정보"를 다루지만, 같은 파일 포맷의 같은 필드는 아니다.
`lineSegArray`는 HWPX XML 세계의 표현이고, `PARA_LINE_SEG`는 HWP5 binary 세계의 표현이다.
따라서 이름과 목적이 비슷하다는 이유만으로 `lineSegArray`를 `PARA_LINE_SEG`로 직접
복사하면 안 된다.

`hwpx_lineseg_reflow_trap.md`에서 확인한 것처럼 HWPX의 lineSegArray는 HWP5
`PARA_LINE_SEG`와 1:1 대응하지 않는다.

#### lineSegArray

HWPX lineSegArray는 HWPX 문서 안에서 조판 결과나 렌더링 힌트에 가까운 정보다. 반면 HWP5
`PARA_LINE_SEG`는 HWP BodyText record tree 안에서 문단, 컨트롤, TAC 표, 글자 계측값,
페이지/섹션 상태와 함께 해석되는 binary record contract다. 두 값은 비슷한 이름과 역할을
갖지만 같은 target contract가 아니다.

특히 다음 케이스에서 그대로 대응시키면 위험하다.

```text
텍스트와 TAC 표가 같은 문단에 섞인 경우
문단 안에 그림/도형 control이 포함된 경우
셀 내부 문단 baseline과 table row height가 함께 작동하는 경우
페이지 나눔 또는 section 경계가 line segment 계산에 영향을 주는 경우
HWPX에는 lineSegArray가 있으나 한컴 HWP oracle의 PARA_LINE_SEG 값이 다른 경우
```

따라서 lineSegArray는 "있으면 참고할 수 있는 입력"이지 "HWP 저장 정답"이 아니다.
HWP 저장용 `PARA_LINE_SEG`는 한컴 HWP oracle의 `PARA_HEADER`, `PARA_TEXT`,
`PARA_CHAR_SHAPE`, `PARA_LINE_SEG`, 주변 control tuple을 함께 비교해서 결정한다.

규칙:

```text
lineSegArray를 HWP PARA_LINE_SEG로 직접 복사하지 않는다.
lineSegArray는 HWP 저장용 line segment 합성의 참고 입력으로만 사용한다.
한컴 HWP oracle의 PARA_LINE_SEG record와 값/개수/범위를 비교하기 전에는 규칙으로 승격하지 않는다.
텍스트+TAC 표 혼합 문단에서는 lineSegArray보다 oracle record tuple을 우선한다.
rhwp reflow 결과가 한컴 oracle과 다르면 reflow 값을 HWP5 contract로 사용하지 않는다.
```

### section_count는 마지막 페이지 출력에 영향을 준다

Task #903 Stage 30에서 `DocProperties.section_count` 보정만으로 마지막 페이지 미출력 문제가
회복되는 사례가 확인되었다.

규칙:

```text
HWPX -> HWP 저장 시 DocProperties.section_count는 실제 section 수와 맞아야 한다.
마지막 페이지 출력 contract에는 section_count / section stream 관계가 포함된다.
```

### ParaShape attr1 vertical align bits는 셀 텍스트 클리핑에 영향을 준다

Task #903 Stage 50-51에서 ParaShape `attr1` 중 vertical align 관련 bits가 셀 내부 텍스트
baseline/클리핑에 영향을 주는 사례가 확인되었다.

규칙:

```text
셀 텍스트가 위로 올라가거나 클리핑되면 ParaShape attr1 vertical align bits를 우선 확인한다.
margin fields만으로는 이 문제를 해결하지 못한다.
```

### BinData와 CTRL_HEADER는 독립 축이 아니다

Task #903 Stage 53에서 `BIN_DATA + CTRL_HEADER` 조합이 `hwpx-h-01`의 이미지 출력과
표 배치 축을 동시에 회복하는 사례가 확인되었다. Stage 54에서는 이 관찰을 실제 구현 경로에서
검증하여 `hwpx-h-01` 한컴/rhwp-studio 판정이 모두 성공했다.

규칙:

```text
이미지 출력 문제와 표 배치 문제를 완전히 분리해서 보지 않는다.
picture/table control의 CTRL_HEADER와 DocInfo BinData mapping은 함께 검증한다.
```

단, 이 규칙은 `hwpx-h-01`에서 확인된 contract checkpoint다. 다른 샘플로 일반화하려면
각 HWPX picture/table construct와 한컴 HWP oracle의 BIN_DATA / CTRL_HEADER tuple 대응을
별도로 확정해야 한다.

### renderingInfo 소수 matrix는 f32 정밀도로 양자화한 뒤 HWP5 double slot에 저장한다

Task #949 Stage 36에서 한컴 파일손상 해소의 마지막 원인은 `SHAPE_COMPONENT` rendering
matrix precision이었다.

HWP5 `SHAPE_COMPONENT`의 matrix 원소는 double slot에 저장되지만, 한컴 HWPX -> HWP oracle은
HWPX XML의 소수 matrix 값을 `f64` 그대로 기록하지 않았다.

확인된 규칙:

```text
정수 matrix 값: 그대로 f64 저장
소수 matrix 값: f32로 양자화한 뒤 f64로 승격해 저장
```

예:

```text
HWPX XML value: 0.723629
rhwp 기존 저장: f64(0.723629)
한컴 정답 저장: f64(f32(0.723629))
```

이 차이는 필드 디코더의 사람이 읽는 출력에서는 같은 값처럼 보일 수 있다. 그러나 payload hash와
byte-level diff에서는 `SHAPE_COMPONENT` rendering matrix block의 하위 바이트가 달라진다.

규칙:

```text
HWPX renderingInfo를 HWP5 raw_rendering으로 materialize할 때 소수 matrix 값은 f32 -> f64로 저장한다.
필드 표시상 값이 같아도 payload hash가 다르면 byte-level diff를 확인한다.
이 규칙은 HWP5 저장용 raw_rendering payload에만 적용한다.
렌더러 내부 계산이나 IR 의미값 전체에 무차별 적용하지 않는다.
```

상세 기록:

```text
mydocs/troubleshootings/hwpx2hwp_shape_rendering_matrix_precision.md
mydocs/working/task_m100_949_stage36.md
```

### Table attr는 관찰 근거가 있으나 oracle contract 대조가 필요하다

Task #903 Stage 58의 작업지시자 판정표는 `mydocs/working/task_m100_903_stage58.md`에
복원해 저장소에서 참조 가능하게 남겼다. 이 기록에 따르면 table attr 계열 변경은 `hwpx-h-01`의
표 배치를 회복시키는 사례가 있었지만, `hwpx-h-02`의 이미지 출력 문제와 `hwpx-h-03`의 파일 손상
문제를 함께 해결하지는 못했다.

따라서 table attr는 "근거 없는 추측"이 아니라 "시각 판정 근거가 있는 후보"다. 다만 이 후보를
production lowering rule로 승격하려면 한컴 HWP oracle의 TABLE / CTRL_HEADER / LIST_HEADER /
PARA_HEADER tuple과 generated HWP의 tuple을 inventory로 대조해야 한다.

규칙:

```text
table attr 적용 결과만으로 전체 HWPX -> HWP 저장 규칙을 확정하지 않는다.
정답 HWP의 table/control subtree와 1:1 대응되는 lowering contract가 확인된 경우에만 규칙으로 승격한다.
Stage 58 판정표는 후보 우선순위를 정하는 근거로 사용한다.
구현 근거는 Stage 58 판정표가 아니라 oracle-derived table/control lowering contract여야 한다.
```

## 7. Contract 추출 규칙

새 작업은 반드시 다음 순서로 진행한다.

```text
1. 대상 HWPX construct를 지정한다.
2. 한컴 HWP oracle에서 대응되는 HWP5 record tuple을 식별한다.
3. HWPX construct -> HWP5 record tuple lowering contract를 문서화한다.
4. rhwp generated HWP가 그 contract를 만족하는지 검증한다.
5. 만족하지 않으면 누락/추가/값 다름/순서 다름을 contract 위반으로 기록한다.
6. synthetic/probe HWP는 contract 확인용으로만 생성한다.
7. 한컴 통과 결과는 contract가 맞다는 검증이지, 추측의 근거가 아니다.
```

금지 규칙:

```text
rhwp-studio reload 성공만으로 한컴 호환으로 판단하지 않는다.
한컴 HWP oracle의 HWP5 record tuple과 정확히 대응시키지 않은 채 구현하지 않는다.
HWPX XML 필드를 HWP5 binary field에 직접 대응한다고 가정하지 않는다.
한 샘플의 한컴 통과를 전체 규칙으로 일반화하지 않는다.
contract가 확정되지 않은 한컴 통과 산출물을 구현 근거로 삼지 않는다.
구현 근거는 oracle-derived lowering contract여야 한다.
```

## 8. 필수 도구 체계

이 문서의 규칙은 사람이 수작업으로 hex dump를 비교하기 위한 것이 아니다.
장기 운영을 위해서는 HWP5 record/control contract를 기계적으로 추출하고 비교하는 도구가
먼저 필요하다.

우선순위는 다음으로 고정한다.

| priority | 도구 | 목적 | 비고 |
|---|---|---|---|
| P0 | `hwp5-inventory` | HWP5 record/control inventory 생성 | 한컴 oracle과 rhwp generated HWP를 같은 언어로 읽는다 |
| P0 | `hwp5-inventory-diff` | oracle/generated HWP5 inventory 대조 | 누락/추가/값 다름/순서 다름과 failure_class 힌트 생성 |
| P1 | `hwpx-control-inventory` | HWPX construct inventory 생성 | 3-way 대조의 HWPX 축 |
| P1 | `hwp5-probe-gen` | 단일 contract unit probe 생성 | synthetic/probe HWP는 contract 확인용 |
| P2 | contract corpus / schema | 1000+ contract unit 축적 | 각 unit을 기계 검증 가능한 형태로 보관 |
| P2 | `hwp5-contract-check` | 회귀 러너 | 샘플/contract unit 회귀 일괄 검증 |
| P2 | dashboard | 진행 상태 가시화 | satisfied / violated / unknown 분포 추적 |

도구 구축 순서:

```text
Phase 1:
  hwp5-inventory
  hwp5-inventory-diff

Phase 2:
  hwpx-control-inventory
  output/poc report 표준화

Phase 3:
  hwp5-probe-gen
  contract_unit schema

Phase 4:
  contract corpus registry
  hwp5-contract-check
  dashboard
```

핵심 원칙:

```text
HWPX -> HWP 저장기 구현은 P0 inventory/diff 없이 진행하지 않는다.
한컴 oracle HWP와 rhwp generated HWP의 HWP5 record/control 차이를 먼저 설명한다.
그 다음에 HWPX construct가 어느 HWP5 tuple로 lowering되어야 하는지 확정한다.
```

### 기존 도구의 한계

`ir-diff`는 IR 레벨 도구다. HWPX와 HWP가 같은 의미 IR을 만드는지 확인할 수 있지만,
HWP5 record/control target contract를 검증하지 못한다.

따라서 다음을 금지한다.

```text
ir-diff가 같으므로 HWP5 저장 contract도 맞다고 판단하지 않는다.
rhwp-studio reload가 성공했으므로 한컴 HWP 로딩도 성공한다고 판단하지 않는다.
```

`examples/hwpx_roundtrip.rs`와 같은 HWPX self-roundtrip 도구도 한컴 oracle이 아니다.
같은 포맷 안에서 자기 보존성을 확인하는 용도로만 사용한다.

```text
HWPX self-roundtrip 성공 = HWPX 저장 경로의 자체 검증
HWPX self-roundtrip 성공 != HWPX -> HWP lowering contract 검증
```

## 9. Inventory 비교 표준 컬럼

정답 HWP와 generated HWP를 비교할 때는 최소한 다음 컬럼을 남긴다.

| column | 의미 |
|---|---|
| sample | 대상 HWPX 샘플 |
| hwp_oracle_path | 한컴 변환 HWP 정답지 |
| generated_path | rhwp 생성 HWP |
| section | BodyText section index |
| record_index | section 내 record index |
| level | HWP record level |
| tag | HWP record tag |
| size | payload byte size |
| owner | paragraph/table/cell/shape 등 소유 구조 |
| key_payload | attr/count/id/tail 등 핵심 payload |
| mismatch | 누락/추가/값 다름/순서 다름 |
| failure_class | A-F 분류 |
| lowering_contract | oracle에서 확정한 HWPX construct -> HWP5 tuple 규칙 |
| contract_status | satisfied / violated / unknown |

## 10. 작업지시자 판정 기록 규칙

시각 판정 파일은 항상 `output/poc/...` 아래 생성한다.

`output/poc/`는 다음 산출물을 모두 포함한다.

```text
작업지시자 시각 판정용 HWP
inventory JSONL/CSV/Markdown
oracle/generated diff report
contract violation hint report
probe generation metadata
```

보고서에는 다음 항목을 반드시 남긴다.

```text
한컴 판정 유형: 성공 / 파일 읽기 오류 / 파일 손상
한컴 마지막 출력 위치
이미지 출력 여부
표/셀 배치 여부
셀 텍스트 클리핑 여부
마지막 페이지 출력 여부
rhwp-studio 판정
정답 HWP 대비 주요 차이
```

특히 `파일 손상`은 마지막 출력 위치가 다음 probe의 시작점이다.

## 11. #944 이후 작업 원칙

#944의 접근 방식은 다음으로 고정한다.

```text
한컴 HWP oracle의 HWP5 record/control inventory를 만든다.
rhwp generated HWP inventory를 만든다.
oracle/generated HWP5 inventory를 먼저 대조한다.
HWPX control inventory를 3-way 대조의 세 번째 축으로 추가한다.
세 inventory를 대조하여 HWPX control -> HWP5 control/record lowering contract를 확정한다.
```

구현은 다음 조건을 모두 만족한 뒤 진행한다.

```text
1. HWPX construct와 한컴 HWP oracle record tuple의 대응이 명확하다.
2. 대응 tuple의 필수 record, level, order, size/count/reference contract가 문서화되어 있다.
3. rhwp generated HWP가 어떤 contract를 위반하는지 정확히 설명된다.
4. synthetic/probe HWP가 해당 contract를 만족할 때 한컴에서 통과한다.
5. 같은 contract가 hwpx-h-01, hwpx-h-02, hwpx-h-03 중 관련 샘플에서 회귀를 만들지 않는다.
6. 규칙을 이 문서에 추가한다.
7. 그 뒤 source adapter/serializer에 반영한다.
```

## 12. 관련 문서

- `mydocs/working/task_m100_944_stage0.md`
- `mydocs/troubleshootings/task178_hwpx_to_hwp_first_attempt_failure.md`
- `mydocs/troubleshootings/task178_second_attempt_hancom_rejection.md`
- `mydocs/troubleshootings/hwpx_lineseg_reflow_trap.md`
- `mydocs/troubleshootings/hwpx2hwp_shape_rendering_matrix_precision.md`
- `mydocs/tech/hwpx2hwp-01.md`
- `mydocs/working/task_m100_903_stage30.md`
- `mydocs/working/task_m100_903_stage53.md`
- `mydocs/working/task_m100_903_stage54.md`
- `mydocs/feedback/hwpx2hwp_rule_doc_review.md`
