# Task M100-949 Stage 35 계획

## 1. 목적

Stage 34에서 남은 차이가 텍스트박스 및 내부 그림의 `SHAPE_COMPONENT` payload로 좁혀졌다.
Stage 35는 HWPX source에 존재하지만 HWP5 저장기에 아직 충분히 반영되지 않은 shape storage,
line, shadow 계열 값을 구현 후보로 반영한다.

## 2. 입력

```text
samples/hwpx/hwpx-h-01.hwpx
samples/hwpx/hwpx-h-02.hwpx
samples/hwpx/hwpx-h-03.hwpx
samples/hwpx/hy-001.hwpx
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
samples/hwpx/hancom-hwp/hy-001.hwp
```

## 3. 구현 후보

```text
1. hp:rotationInfo@rotateimage -> SHAPE_COMPONENT storage bit 0x00080000
2. hp:drawText를 가진 외곽 rect -> SHAPE_COMPONENT storage bit 0x01000000
3. hp:lineShape color/style/headfill/tailfill -> ShapeBorderLine
4. hp:shadow type/color/offset/alpha -> DrawingObjAttr shadow fields
```

## 4. 판정 기준

```text
- h03/hy 문제 구간의 SHAPE_COMPONENT diff가 줄어드는지 확인한다.
- guard 샘플 h01/h02의 기존 성공 축이 깨지면 후보에서 제외한다.
- byte-equal이 되지 않은 축은 다음 stage에서 exact byte diff로 다시 좁힌다.
```

## 5. 산출물

```text
output/poc/hwpx2hwp/task949/stage35_shape_storage_payload_candidate/
```
