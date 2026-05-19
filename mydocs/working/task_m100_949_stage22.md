# task_m100_949 Stage 22 작업 기록

## 1. 목적

`hwpx-h-03`의 한컴 에디터 파일손상 원인을 `hp:rect > hp:drawText` 내부 HWP5 record contract
관점에서 정적으로 추적한다.

이번 단계는 구현 후보를 만들기 전 단계다. 목표는 다음을 분리하는 것이다.

```text
1. HWPX source에 명시되어 있고 이미 반영된 계약
2. HWPX source에는 직접 없지만 한컴 HWP 저장 결과에는 materialize되는 record tail/field
3. 아직 원인으로 단정하면 안 되는 주변 GenShape/ShapeComponent payload 차이
```

## 2. 기준 파일

Oracle HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
```

Generated HWP:

```text
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-03.hwp
```

출력 경로:

```text
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/
```

## 3. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/oracle_section0.jsonl
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/generated_section0.jsonl
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/oracle_inventory.md
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/generated_inventory.md
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/shape_bundles_w8.md
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/hwpx-h-03_source_tree.md
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/record_window_824_838_oracle.md
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/record_window_824_838_generated.md
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/list_header_826_decode.md
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/para_header_827_decode.md
output/poc/hwpx2hwp/task949/stage22_drawtext_record_contract/drawtext_contract_findings.md
```

## 4. Source tree 대응

`hwpx-h-03`의 파일손상 후보 source tree:

```text
hp:rect
  hp:shapeComment
  hp:drawText
    hp:subList
      hp:p
        hp:run
          hp:pic
          hp:pic
          hp:t
        hp:linesegarray
```

이 source tree는 Hancom oracle HWP에서 `BodyText.Section0#820..#838` record window로
materialize된다.

```text
#820 PARA_HEADER       outer paragraph
#821 PARA_TEXT
#822 PARA_CHAR_SHAPE
#823 PARA_LINE_SEG
#824 CTRL_HEADER       GenShape, hp:rect
#825 SHAPE_COMPONENT
#826 LIST_HEADER       hp:drawText/hp:subList text-box list
#827 PARA_HEADER       hp:drawText inner paragraph
#828 PARA_TEXT
#829 PARA_CHAR_SHAPE
#830 PARA_LINE_SEG
#831 CTRL_HEADER       first hp:pic
#832 SHAPE_COMPONENT
#833 CTRL_DATA         first hp:pic href data
#834 SHAPE_PICTURE
#835 CTRL_HEADER       second hp:pic
#836 SHAPE_COMPONENT
#837 SHAPE_PICTURE
#838 SHAPE_RECTANGLE
```

## 5. 이미 닫힌 계약

Stage 15/21에서 다음 계약은 닫혔다.

```text
hp:pic@href -> CTRL_DATA #833
hp:shapeComment -> CommonObjAttr.description
hp:subList@vertAlign=CENTER -> LIST_HEADER list_attr bit 5
```

정적 확인:

```text
oracle    CTRL_DATA #833 size=76 hash=024e873ad9c2bd92...
generated CTRL_DATA #833 size=76 hash=024e873ad9c2bd92...
```

따라서 현재 남은 파일손상 원인을 `hp:pic@href`의 `CTRL_DATA` 누락으로 다시 돌리면 안 된다.

## 6. Stage 22 핵심 차이

### 6.1 LIST_HEADER #826

Oracle:

```text
size=33
01 00 00 00 20 00 00 00 00 00 00 00 00 00 00 00
62 64 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00
```

Generated:

```text
size=20
01 00 00 00 20 00 00 00 00 00 00 00 00 00 00 00
62 64 00 00
```

해석:

```text
1. para_count, list_attr, margin, max_width의 20B base payload는 일치한다.
2. Stage 21에서 vertAlign=CENTER -> list_attr=0x20 반영은 성공했다.
3. 남은 차이는 oracle의 13B zero tail이다.
```

이 tail은 HWPX XML에 직접 등장하는 속성이 아니라, 한컴이 HWP5 text-box `LIST_HEADER`를
저장할 때 materialize하는 record contract로 보는 것이 현재 가장 정밀하다.

### 6.2 PARA_HEADER #827

Oracle:

```text
size=24
16 00 00 80 00 08 00 00 55 00 00 00 01 00 00 00
01 00 00 00 00 80 00 00
```

Generated:

```text
size=22
16 00 00 80 00 08 00 00 55 00 00 00 01 00 00 00
01 00 00 00 00 00
```

해석:

```text
1. char_count, control_mask, para_shape_id, style_id, count 계열은 일치한다.
2. 차이는 drawText 내부 paragraph header의 tail 길이/형태다.
3. PARA_TEXT, PARA_CHAR_SHAPE, inner PARA_LINE_SEG #830은 oracle과 일치한다.
```

즉 텍스트 내용이나 line segment 자체가 이 위치의 주원인은 아니다. 문제는 HWPX-origin
drawText 내부 문단을 HWP5 `PARA_HEADER`로 낮출 때 필요한 tail contract다.

## 7. 아직 원인으로 단정하지 않는 차이

다음 record들은 payload hash가 다르다.

```text
#824 CTRL_HEADER GenShape
#825 SHAPE_COMPONENT
#831 CTRL_HEADER GenShape
#832 SHAPE_COMPONENT
#834 SHAPE_PICTURE
#835 CTRL_HEADER GenShape
#836 SHAPE_COMPONENT
#837 SHAPE_PICTURE
```

하지만 Stage 22에서는 이것들을 곧바로 1순위 구현 대상으로 두지 않는다.

이유:

```text
1. #826 LIST_HEADER와 #827 PARA_HEADER는 크기 자체가 oracle과 다르다.
2. #833 CTRL_DATA와 #838 SHAPE_RECTANGLE처럼 이미 일치하는 record도 같은 tuple 안에 있다.
3. 먼저 text-box list/paragraph contract를 닫아야 나머지 GenShape payload 차이가 독립 원인인지
   판단할 수 있다.
```

## 8. 결론

현재 가장 좁은 구현 후보:

```text
1. HWPX-origin hp:drawText text box에 대해 LIST_HEADER 13B tail을 materialize한다.
2. HWPX-origin hp:drawText 내부 paragraph에 대해 Hancom oracle과 같은 24B PARA_HEADER tail을
   materialize한다.
```

이것은 새 한컴 control을 찾는 문제가 아니라, 이미 파싱된 `hp:drawText`와 그 내부 문단을
HWP5 record contract에 맞게 lowering하는 문제다.

## 9. 다음 단계 제안

Stage 23에서는 구현을 바로 넓히지 말고 다음 순서로 진행한다.

```text
1. hwpx-h-01/hwpx-h-02 성공 guard를 유지한다.
2. #826 LIST_HEADER 13B tail만 materialize하는 후보를 만든다.
3. #827 PARA_HEADER 24B tail 후보를 분리한다.
4. 두 후보를 조합했을 때 hwpx-h-03 파일손상이 사라지는지 확인한다.
5. 그래도 남으면 #824/#831/#835 GenShape attr 및 SHAPE_COMPONENT payload 차이를 다음 계약으로
   추적한다.
```

guard 순서:

```text
1. hwpx-h-01 성공 유지
2. hwpx-h-02 성공 유지
3. hwpx-h-03 파일손상 해소
```

`hwpx-h-01`에서 회귀가 발생하는 후보는 `hwpx-h-03`에 변화가 있더라도 폐기한다.

## 10. 검증

실행:

```text
target/debug/rhwp hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-03.hwp --format jsonl --section 0
target/debug/rhwp hwp5-inventory output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-03.hwp --format jsonl --section 0
target/debug/rhwp hwp5-inventory-diff samples/hwpx/hancom-hwp/hwpx-h-03.hwp output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-03.hwp --align lcs --report bundles --focus shape --window 8 --section 0 --format md
```

문서 검증:

```text
git diff --check
```
