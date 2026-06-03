# Task M100-949 Stage 18 계획서: table-axis 성공 후보 확장 검증

## 1. 목적

Stage 17에서 `hwpx-h-01` sentinel이 성공했다. 이제 같은 adapter를 `hwpx-h-02`,
`hwpx-h-03`에 적용해 다음 문제 축을 분리한다.

## 2. 원칙

```text
1. Stage 17 성공 후보를 그대로 사용한다.
2. hwpx-h-01은 guard로 다시 생성한다.
3. hwpx-h-02/hwpx-h-03에서 실패가 발생하면 그 실패를 새 probe로 넓히지 않는다.
4. 실패 샘플의 oracle HWP record bundle과 HWPX source contract를 직접 비교한다.
```

## 3. 산출물

```text
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/
```

생성 대상:

```text
samples/hwpx/hwpx-h-01.hwpx
samples/hwpx/hwpx-h-02.hwpx
samples/hwpx/hwpx-h-03.hwpx
```

