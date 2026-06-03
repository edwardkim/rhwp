# Task m100 #949 Stage 8 계획: TABLE probe manifest

## 1. 목적

Stage 7 `table-fields` 결과를 판정용 HWP 생성 전에 사용할 수 있는 probe manifest로 정리한다.

이번 단계의 목표는 HWP를 직접 패치하는 것이 아니다. 먼저 다음 단계에서 생성할 판정 파일의
축을 기계적으로 고정한다.

```text
1. 어떤 TABLE 관련 필드군이 반복적으로 다른지 분리한다.
2. 각 필드군이 어떤 record index에 적용되어야 하는지 나열한다.
3. 단독/조합 probe matrix를 자동 생성한다.
```

## 2. 구현 범위

CLI 확장:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report table-probe-plan \
  --out <path>
```

추가 report:

```text
table-probe-plan
```

출력 내용:

```text
- probe axis summary
- recommended probe matrix
- axis별 affected record 목록
- oracle/generated field value
```

## 3. Probe 축

Stage 7 결과를 기준으로 다음 축을 분리한다.

```text
ctrl_outer_margin
ctrl_common_attr
table_attr
table_tail
```

주의:

```text
이 축 이름은 P0 관찰명이다.
HWP5 최종 contract 이름으로 확정하지 않는다.
```

## 4. 검증

```text
cargo build
table-probe-plan 산출물 생성
기존 diff/hints/bundles/table-fields smoke 유지
```

