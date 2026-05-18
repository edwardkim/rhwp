# Task M100-949 Stage 36 계획

## 1. 목적

Stage 35에서 남은 `SHAPE_COMPONENT` 차이는 필드 표시상 모두 같은 값으로 보였지만 payload
hash가 달랐다. Stage 36은 표시 정밀도에 가려진 실제 byte 차이를 찾아 HWP5 저장 규칙으로
반영한다.

## 2. 입력

```text
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
samples/hwpx/hancom-hwp/hy-001.hwp
output/poc/hwpx2hwp/task949/stage35_shape_storage_payload_candidate/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage35_shape_storage_payload_candidate/hy-001.hwp
```

## 3. 절차

```text
1. 남은 SHAPE_COMPONENT의 byte-level diff를 추출한다.
2. 차이가 발생하는 offset을 HWP5 matrix block에 매핑한다.
3. HWPX renderingInfo 값을 HWP5 raw_rendering으로 낮출 때의 정밀도 규칙을 구현한다.
4. h03/hy 문제 구간이 정답 HWP와 byte-equal인지 확인한다.
5. h01/h02/h03/hy 산출물을 생성해 작업지시자 시각 판정을 요청한다.
```

## 4. 판정 기준

```text
- h03 문제 구간 #824~#838 shape/textbox bundle이 정답 HWP와 byte-equal
- hy 문제 구간 #222~#235 shape/textbox bundle이 정답 HWP와 byte-equal
- rhwp 재로드가 h01/h02/h03/hy에서 실패하지 않음
- 최종 판정은 한컴 에디터 시각 확인으로 확정
```

## 5. 산출물

```text
output/poc/hwpx2hwp/task949/stage36_rendering_f32_quantize/
```
