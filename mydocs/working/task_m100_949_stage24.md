# Task M100-949 Stage 24 - `hwpx-h-03` 파일손상 계약 재고정

## 1. 목적

Stage 23에서 TextBox HWP5 envelope 보강 후보가 `hy-001` guard를 깨뜨렸다.
따라서 Stage 24는 구현을 추가하지 않고, `hwpx-h-03` 파일손상 지점을 정답 HWP와 현재 생성 HWP의
record contract 차이로 다시 고정한다.

## 2. 입력

```text
source    : samples/hwpx/hwpx-h-03.hwpx
oracle    : samples/hwpx/hancom-hwp/hwpx-h-03.hwp
generated : output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/hwpx-h-03-current.hwp
guard     : samples/hwpx/hy-001.hwpx
guard hwp : samples/hwpx/hancom-hwp/hy-001.hwp
```

## 3. 산출물

```text
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/h03_anchor.md
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/h03_record_bundle_diff.md
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/h03_vs_hy001_textbox_guard.md
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/h03_inventory_diff_index.md
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/h03_inventory_diff_lcs.md
output/poc/hwpx2hwp/task949/stage24_h03_damage_contract/hy001_inventory_diff_index.md
```

## 4. 작업지시자 관찰 반영

작업지시자 관찰:

```text
hwpx-h-03.hwp에서 문제가 발생되는 지점의 특이한 점은
문단 시작할 때 [표]가 먼저 선언된다.
이후 텍스트가 2줄로 구성된 후, 글상자가 만들어지고 그 안에 이미지가 들어가는 구조다.
```

Stage 24에서는 이 관찰을 record inventory 기준으로 다음 경계에 고정했다.

```text
drawText/image group tuple : #820..#838
following table paragraph : #839..#858
```

한컴 에디터가 시각적으로 2페이지 글상자 전후에서 멈추는 현상은 이 경계의 contract mismatch로 본다.

## 5. 핵심 diff

### 정상으로 확인된 항목

```text
- drawText 내부 LIST_HEADER/PARA_HEADER는 현재 generated와 oracle이 일치한다.
- hp:pic@href -> CTRL_DATA payload/위치도 #833에서 oracle과 일치한다.
- following table paragraph의 PARA_TEXT #840은 oracle과 일치한다.
- following table의 CTRL_HEADER #843과 TABLE #844도 oracle과 일치한다.
```

### 여전히 다른 항목

```text
#824 CTRL_HEADER outer rect GenShape
#831 CTRL_HEADER first child picture GenShape
#835 CTRL_HEADER second child picture GenShape
#825/#832/#836 SHAPE_COMPONENT payload
#834/#837 SHAPE_PICTURE payload
following table cell LIST_HEADER/PARA_HEADER tail
```

가장 강한 후보는 `#824/#831/#835`의 GenShape CommonObjAttr/placement 차이다.

## 6. 해석

Stage 23의 generic TextBox LIST/PARA header tail 보강은 폐기한다.

이유:

```text
1. hwpx-h-03의 drawText 내부 LIST/PARA header는 이미 oracle과 일치한다.
2. Stage 23 후보는 hy-001 guard를 파일손상으로 만들었다.
3. 따라서 TextBox envelope 전체를 일반 규칙으로 보강하는 방식은 위험하다.
```

Stage 24 기준의 다음 후보는 다음이다.

```text
source XML의 textWrap/textFlow/TAC picture placement 정보를
HWP5 GenShape CTRL_HEADER CommonObjAttr와 child picture placement 값으로 정확히 매핑한다.
```

## 7. 다음 작업

Stage 25에서는 구현 전에 다음 중 하나를 먼저 수행한다.

```text
1. #824/#831/#835 CTRL_HEADER payload field decode를 더 세분화한다.
2. oracle의 CommonObjAttr high bits가 source XML의 어떤 속성에서 유도되는지 규칙화한다.
3. `hwpx-h-01`, `hwpx-h-02`, `hwpx-h-03`, `hy-001` 4개를 동시에 guard하는 최소 후보를 만든다.
```

검증 기준:

```text
hwpx-h-01: 성공 유지
hwpx-h-02: 성공 유지
hwpx-h-03: 파일손상 해소
hy-001: #974 visual guard 유지
```

