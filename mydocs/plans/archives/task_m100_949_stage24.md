# Task M100 #949 Stage 24 계획서 — hwpx-h-03 파일손상 원인 재분리

## 1. 배경

Stage 23 후보는 reject한다.

판정 결과:

```text
hwpx-h-01: 성공
hwpx-h-02: 성공
hwpx-h-03: 파일손상, 2페이지 글상자 전까지만 출력
hy-001: 파일손상, #974 guard 회귀
```

Stage 23에서 확인한 사실:

```text
1. hwpx-h-03의 #826 LIST_HEADER, #827 PARA_HEADER는 정답지와 size/hash가 일치했다.
2. 그런데 hwpx-h-03 파일손상은 남았다.
3. TextBox 전체에 HWP5 envelope 보강을 generic하게 적용하면 hy-001 guard가 깨진다.
```

따라서 다음 단계는 TextBox 전체 보강이 아니라, `hwpx-h-03`에서 한컴 에디터가 중단하는
2페이지 글상자 주변의 실제 HWP5 record contract를 정답지와 대조한다.

## 2. 목표

`hwpx-h-03` 파일손상 원인을 다음 후보군 중 하나로 좁힌다.

```text
1. 2페이지 문제 문단의 control ordering 계약
2. 2페이지 글상자 host shape의 CTRL_HEADER / SHAPE_COMPONENT / GEN_SHAPE_OBJECT 계약
3. 글상자 내부 picture/group child record 계약
4. drawText 주변 PARA_HEADER/PARA_TEXT/PARA_LINE_SEG record level 또는 sibling ordering 계약
5. Stage 23에서 닫은 #826/#827 이후에 남은 tail/reference/count 계약
```

이번 단계의 성공 기준은 구현이 아니라 "정답지와 generated 사이에서 한컴 파일손상과 직접 연결되는
최소 record bundle 차이를 설명 가능한 형태로 식별"하는 것이다.

## 3. 작업 원칙

```text
1. hwpx-h-01에서 이미 성공한 table-axis 계약은 건드리지 않는다.
2. hy-001 guard를 깨는 generic TextBox 보강은 다시 사용하지 않는다.
3. static test 통과만으로 성공 판정하지 않는다.
4. 한컴 시각 판정이 필요한 파일은 최소화한다.
5. 후보 구현 전 정답지 record bundle과 generated record bundle의 차이를 먼저 문서화한다.
```

## 4. 비교 대상

정답지:

```text
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
```

generated baseline:

```text
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-03.hwp
```

필요 시 재생성 baseline:

```text
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/hwpx-h-03-current.hwp
```

guard 파일:

```text
samples/hwpx/hwpx-h-01.hwpx
samples/hwpx/hwpx-h-02.hwpx
samples/hwpx/hy-001.hwpx
```

## 5. 절차

### 5.0 문제 문단의 구조 관찰을 anchor로 둔다

작업지시자 관찰:

```text
hwpx-h-03에서 문제가 발생하는 지점은 문단 시작 시 [표]가 먼저 선언된다.
이후 텍스트가 2줄로 구성되고, 그 뒤에 글상자가 만들어지며, 글상자 안에 이미지가 들어간다.
```

따라서 Stage 24의 1차 비교 단위는 TextBox 단독이 아니라 "한 문단 안의 control stream"이다.

확인할 항목:

```text
1. 문단 PARA_TEXT 안의 control char 순서와 개수
2. 문단 controls 배열의 순서: table -> text lines -> shape/textBox 계열인지
3. TABLE control tuple과 SHAPE control tuple 사이의 record level/order
4. 2줄 텍스트를 구성하는 PARA_LINE_SEG / line break / char shape range
5. 글상자 내부 picture가 host paragraph의 table control과 같은 paragraph scope에 묶이는지
6. oracle HWP와 generated HWP에서 control id, control index, record parent scope가 일치하는지
```

이 구조는 `drawText` 내부 envelope만 맞춰서는 해결되지 않는다. host paragraph의 table/text/shape
control ordering contract가 맞지 않으면 한컴 로더가 글상자 직전 또는 내부 image에서 파일손상으로
판정할 수 있다.

### 5.1 h03 source/HWP record anchor 재확인

`hwpx-h-03`에서 한컴이 중단하는 2페이지 글상자 주변 source node와 HWP record index를 다시
고정한다.

산출물:

```text
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/h03_anchor.md
```

포함 내용:

```text
1. source XPath 또는 source summary
2. oracle record index range
3. generated record index range
4. parent/child level tree
```

### 5.2 oracle/generated record bundle diff

비교 범위는 파일 전체가 아니라 한컴 중단 지점 주변으로 제한한다.

우선 비교 후보:

```text
#824 CTRL_HEADER
#825 SHAPE_COMPONENT
#826 LIST_HEADER
#827 PARA_HEADER
#828 PARA_TEXT
#829 PARA_CHAR_SHAPE
#830 PARA_LINE_SEG
#831 CTRL_HEADER
#832 SHAPE_COMPONENT
#833 CTRL_DATA
#834 SHAPE_COMPONENT
#835 GEN_SHAPE_OBJECT
```

단, Stage 23에서 #826/#827은 이미 닫았으므로 이번 단계에서는 나머지 record의
payload/level/order/count 차이를 우선 본다.

산출물:

```text
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/h03_record_bundle_diff.md
```

### 5.3 hy-001 guard와 충돌 여부 분리

`hy-001`은 #974 guard다. `hwpx-h-03` 후보 contract가 `hy-001`의 어떤 TextBox에는 적용되면
안 되는지 조건을 분리한다.

비교 항목:

```text
1. text_box.paragraphs 개수
2. 내부 picture control 존재 여부
3. host shape kind
4. shape description/comment 존재 여부
5. control nesting level
6. raw_list_header_extra / raw_header_extra 원본 상태
```

산출물:

```text
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/h03_vs_hy001_textbox_guard.md
```

## 6. 구현 판단 기준

이번 단계에서 바로 코드를 고치지 않는다. 다음 조건을 만족할 때만 Stage 25에서 구현한다.

```text
1. h03 정답지와 generated의 최소 차이가 record 단위로 확인된다.
2. 그 차이가 h01/h02 성공 guard와 충돌하지 않는 이유를 설명할 수 있다.
3. 그 차이가 hy-001 guard와 충돌하지 않는 적용 조건을 설명할 수 있다.
```

## 7. 완료 조건

```text
1. h03_anchor.md 작성
2. h03_record_bundle_diff.md 작성
3. h03_vs_hy001_textbox_guard.md 작성
4. mydocs/working/task_m100_949_stage24.md 보고서 작성
5. Stage 25 구현 후보가 있으면 명확히 기술, 없으면 추가 관찰 항목을 명시
```
