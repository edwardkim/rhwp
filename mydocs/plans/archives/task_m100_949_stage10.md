# Task m100 #949 Stage 10 계획: hwpx-h-03 성공 축 재검증

## 1. 배경

Stage 9에서 `hwpx-h-01`에 대해 TABLE/CTRL_HEADER(Table) 축별 graft probe를 만들었다.
작업지시자 판정 결과 `04_ctrl_common_attr_only`와 `08_all_table_axes`가 한컴 에디터와
rhwp-studio 양쪽에서 성공했다.

이번 단계는 같은 성공 축을 `hwpx-h-03` 계열 baseline에 적용했을 때, 기존에 반복되던
한컴 파일손상 판정이 사라지는지 확인한다.

## 2. 입력

```text
oracle:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated baseline:
output/poc/hwpx2hwp/task903/stage58_table_ctrl_attr_margin_probe/01_current_baseline_hwpx-h-03.hwp
```

주의:

```text
generated baseline은 Stage 58의 현 production 산출물이다.
파일 크기가 약 38KB로 작으므로, 이미지/BinData 리소스 판정은 보조로 보고
이번 핵심은 한컴 파일손상/읽기오류 판정 변화다.
```

## 3. 작업

```bash
./target/debug/rhwp hwp5-table-probe \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task903/stage58_table_ctrl_attr_margin_probe/01_current_baseline_hwpx-h-03.hwp \
  --out-dir output/poc/hwpx2hwp/task949/stage10/hwpx-h-03
```

추가로 `table-probe-plan`을 생성해 h-03의 TABLE 축 차이를 문서화한다.

## 4. 판정 우선순위

우선 확인 대상:

```text
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/04_ctrl_common_attr_only.hwp
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp
```

전체 비교 대상:

```text
01_ctrl_outer_margin_only.hwp
02_table_attr_only.hwp
03_table_tail_only.hwp
04_ctrl_common_attr_only.hwp
05_outer_margin_table_attr.hwp
06_outer_margin_table_tail.hwp
07_table_attr_tail.hwp
08_all_table_axes.hwp
```

## 5. 판정 항목

```text
- 한컴 에디터에서 파일손상 또는 파일 읽기 오류가 발생하는지
- 한컴 에디터에서 몇 페이지까지 출력되는지
- 이미지 출력이 유지되는지
- 표/셀 배치가 개선되는지
- rhwp-studio reload와 렌더링이 정상인지
```
