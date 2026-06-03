# Task m100 #949 Stage 20 작업 기록

## 1. 목적

Stage 19에서 `hwpx-h-03`의 `memoProperties -> MEMO_SHAPE` 및 `ID_MAPPINGS` 계약은
정적으로 닫혔지만, 한컴 판정은 Stage 18과 동일하게 파일손상이었다.

Stage 20의 목적은 다음 차이를 후보 구현 전에 정확히 해석하는 것이다.

```text
BodyText.Section0#824
oracle:    CTRL_HEADER GenShape size=60
generated: CTRL_HEADER GenShape size=46
```

## 2. 산출물

```text
output/poc/hwpx2hwp/task949/stage20_genshape_contract_trace/
```

작성한 분석 문서:

```text
genshape_824_payload_diff.md
genshape_824_field_decode.md
genshape_source_trace.md
success_control_comparison.md
stage20_findings.md
```

근거 데이터:

```text
oracle_section0.jsonl
generated_section0.jsonl
shape_bundles_w8.md
oracle_dump_records.txt
generated_dump_records.txt
```

## 3. 핵심 발견

`#824`의 60B/46B 차이는 알 수 없는 tail이 아니다.

추가 14바이트는 HWP 문자열이다.

```text
07 00 ac c0 01 ac 15 d6 85 c7 c8 b2 e4 b2 2e 00
```

해석:

```text
length = 7
text   = "사각형입니다."
```

HWPX 원본에도 대응 값이 있다.

```xml
<hp:shapeComment>사각형입니다.</hp:shapeComment>
```

따라서 이 차이는 다음처럼 정리한다.

```text
GenShape #824 unknown tail 14B 누락이 아니라,
hp:rect의 shapeComment를 CommonObjAttr.description으로 보존하지 않은 문제다.
```

## 4. 주변 tuple 차이

`#824` 하나만 보면 설명문 누락이 보이지만, 같은 control tuple에는 더 중요한 차이가 있다.

| record | oracle | generated | 해석 |
|---|---:|---:|---|
| `#824 CTRL_HEADER` | 60B | 46B | `shapeComment` 설명문 누락, attr bit 차이 |
| `#825 SHAPE_COMPONENT` | 252B | 252B | size는 같지만 rendering/shadow/tail payload 차이 |
| `#826 LIST_HEADER` | 33B | 20B | drawText/subList list_attr와 13B tail 차이 |
| `#827 PARA_HEADER` | 24B | 22B | drawText 내부 paragraph header tail 차이 |
| `#831 CTRL_HEADER` | 176B | 176B | size는 같지만 attr bit 차이 |
| `#833 CTRL_DATA` | 76B | 76B | Stage 19에서 oracle과 payload 일치 |
| `#835 CTRL_HEADER` | 46B | 46B | size는 같지만 attr bit 차이 |
| `#838 SC_RECT` | 33B | 33B | size 동일 |

## 5. HWPX source 구조

`hwpx-h-03`은 명시적인 `hp:container`가 없다.

```text
hp:container = 0
hp:pic       = 3
```

문제가 되는 2페이지 이미지 묶음은 다음 구조다.

```text
hp:rect
  hp:drawText
    hp:subList vertAlign="CENTER"
      hp:p
        hp:run
          hp:pic href="http://www.korea.kr;1;0;0;"
          hp:pic href=""
          hp:t
  hp:shapeComment
```

반면 성공 대조군 `hwpx-h-01`에는 명시적인 `hp:container`가 있다.

```text
hp:container = 1
hp:pic       = 5
```

따라서 `hp:container` 경로와 `hp:rect/drawText` 경로를 같은 구현으로 취급하면 안 된다.

## 6. 코드상 확인한 누락

```text
src/parser/hwpx/section.rs
```

확인 내용:

```text
1. parse_picture는 hp:shapeComment를 CommonObjAttr.description에 저장한다.
2. parse_shape_object는 hp:shapeComment를 처리하지 않는다.
3. parse_draw_text는 subList@vertAlign을 TextBox.vertical_align에 저장한다.
4. serializer는 TextBox.vertical_align이 아니라 TextBox.list_attr 원값을 쓴다.
```

따라서 `hp:drawText/hp:subList vertAlign="CENTER"`는 현재 HWP `LIST_HEADER list_attr=0x20`으로
materialize되지 않는다.

## 7. Stage 20 판정

Stage 20에서 직접 원인을 다음처럼 재정의한다.

```text
아님:
  GenShape #824의 정체불명 14B tail 누락

맞음:
  hp:rect/drawText를 HWP5의 GenShape + text-box LIST_HEADER + embedded paragraph +
  child picture GenShape + SC_RECT tuple로 낮추는 contract 미완성
```

## 8. 다음 단계

Stage 21 후보는 `hp:rect/drawText` 경로에 한정한다.

우선순위:

```text
1. 일반 shape의 hp:shapeComment를 CommonObjAttr.description으로 보존한다.
2. drawText subList vertAlign=CENTER를 TextBox.list_attr bit 5로 materialize한다.
3. #826 LIST_HEADER 13B tail과 #827 PARA_HEADER 24B/22B 차이를 추가로 확인한다.
```

판정 순서:

```text
1. hwpx-h-01 성공 유지
2. hwpx-h-02 성공 유지
3. hwpx-h-03 파일손상 개선 여부 확인
```

`hwpx-h-01` 또는 `hwpx-h-02`에서 회귀가 생기면 후보는 폐기한다.

## 9. 실행한 검증

Stage 20은 분석 단계라 한컴 판정용 HWP를 새로 만들지 않았다.

실행한 정적 분석:

```text
target/debug/rhwp hwp5-inventory ... --section 0
target/debug/rhwp hwp5-inventory-diff ... --focus shape --window 8
target/debug/rhwp dump-records ...
```
