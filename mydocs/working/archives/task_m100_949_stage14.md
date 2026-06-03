# Task m100 #949 Stage 14 작업 보고서: CTRL_DATA ParameterSet trace

## 1. 목적

Stage 13에서 oracle-only로 확인된 `CTRL_DATA#833`의 의미를 해석했다.

이번 단계에서는 HWP probe 파일을 만들지 않았다. 작업지시자 시각 판정도 요청하지 않았다.

## 2. 추가한 진단 명령

```text
rhwp hwp5-ctrl-data-trace <oracle.hwp> <generated.hwp> --out <path> [--section N] [--record-index N]
```

소스:

```text
src/diagnostics/hwp5_ctrl_data_trace.rs
src/diagnostics/mod.rs
src/main.rs
```

역할:

```text
- oracle/generated HWP의 CTRL_DATA record 수와 payload hash 비교
- CTRL_DATA payload를 HWP ParameterSet 구조로 재귀 decode
- String, nested ParameterSet, BINDataID 등 기본 ParameterItem type 표시
```

## 3. 실행 명령

```bash
cargo run --quiet --bin rhwp -- hwp5-ctrl-data-trace \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage14/hwpx-h-03/ctrl_data_trace.md
```

## 4. 산출물

```text
output/poc/hwpx2hwp/task949/stage14/hwpx-h-03/ctrl_data_trace.md
```

## 5. 결과

### 5.1 CTRL_DATA count

```text
oracle:
  CTRL_DATA records = 1
  total bytes = 76

generated:
  CTRL_DATA records = 0
```

### 5.2 oracle CTRL_DATA#833 구조

위치:

```text
BodyText/Section0/PARA_HEADER#820
  /CTRL_HEADER#824
  /SHAPE_COMPONENT#825
  /PARA_HEADER#827
  /CTRL_HEADER#831
  /SHAPE_COMPONENT#832
  /CTRL_DATA#833
```

ParameterSet decode:

```text
ParameterSet ps_id=0x021b count=1 dummy=0x0000
  item#0 id=0x026f type=0x8000(ParameterSet)
    ParameterSet ps_id=0x026f count=1 dummy=0x0000
      item#0 id=0x0265 type=0x0001(String)
        string len=27 value="http\://www.korea.kr;1;0;0;"
```

해석:

```text
이 CTRL_DATA는 그림/묶음의 임의 바이너리 blob이 아니라 HWPX picture href를
HWP5 ParameterSet 형태로 저장한 record다.
```

## 6. HWPX 원본 대응 정보

`samples/hwpx/hwpx-h-03.hwpx`의 `Contents/section0.xml`에서 같은 의미 정보가 확인된다.

```xml
<hp:pic
  id="1875692960"
  zOrder="28"
  numberingType="PICTURE"
  textWrap="TOP_AND_BOTTOM"
  href="http://www.korea.kr;1;0;0;"
  instid="801951137"
  reverse="0">
```

이 `hp:pic`은 다음 문맥에 있다.

```text
hp:rect
  hp:drawText
    hp:subList
      hp:p
        hp:run
          hp:pic href="http://www.korea.kr;1;0;0;"
```

따라서 Stage 13에서 말한 nested GenShape tuple은 HWPX의 글상자/도형 내부 그림에 대응한다.

## 7. 소스 누락 지점

현재 HWPX parser의 `parse_picture`는 `<hp:pic href="...">` 속성을 읽지 않는다.

```text
src/parser/hwpx/section.rs
```

현재 `Picture` 모델에도 HWPX `href`를 보존할 필드가 없다.

```text
src/model/image.rs
```

현재 HWPX -> HWP adapter는 table contract만 materialize한다.

```text
src/document_core/converters/hwpx_to_hwp.rs
```

## 8. 구현 결론

다음 구현은 다음 순서가 맞다.

```text
1. Picture IR에 HWPX href 보존 필드 추가
2. HWPX parser에서 hp:pic@href를 Picture IR에 저장
3. HWPX -> HWP adapter에서 href가 있는 Picture에 대해 CTRL_DATA ParameterSet을 materialize
4. serializer는 기존 경로 그대로 사용
```

중요한 점:

```text
CTRL_DATA를 모든 picture에 붙이는 것이 아니다.
href가 있는 picture에 한해, 한컴 oracle과 같은 ParameterSet 형태로 합성해야 한다.
```

## 9. 검증

```text
cargo check: 통과
hwp5-ctrl-data-trace: 보고서 생성 통과
HWP probe 생성: 없음
작업지시자 시각 판정 요청: 없음
```

## 10. 다음 단계

Stage 15에서 위 구현 결론을 코드에 반영한다.

검증 순서:

```text
1. generated HWP에 CTRL_DATA count=1이 생기는지 확인
2. CTRL_DATA payload가 oracle과 같은 ParameterSet 구조인지 확인
3. 기존 h01/h02에 불필요한 CTRL_DATA가 생기지 않는지 확인
4. 그 다음에만 한컴 에디터 판정을 요청한다
```
