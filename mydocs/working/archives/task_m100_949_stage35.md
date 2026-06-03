# Task M100-949 Stage 35 작업 기록

## 1. 목적

Stage 34에서 남은 차이는 텍스트박스 외곽 `SHAPE_COMPONENT`와 내부 그림
`SHAPE_COMPONENT`에 집중되어 있었다. Stage 35는 HWPX source의 shape storage, line,
shadow 값을 HWP5 record payload로 낮추는 후보를 적용했다.

## 2. 구현 내용

변경된 축:

```text
- hp:rotationInfo@rotateimage 보존 필드 추가
- drawText를 가진 rect의 storage bit 0x01000000 적용
- picture/group storage bit 기존 규칙 유지
- rotateimage=true인 shape에 0x00080000 bit 적용
- lineShape color/style/headfill/tailfill 반영
- shadow type/color/offset/alpha 반영
```

주요 소스:

```text
src/model/shape.rs
src/parser/hwpx/section.rs
```

## 3. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage35_shape_storage_payload_candidate/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage35_shape_storage_payload_candidate/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage35_shape_storage_payload_candidate/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage35_shape_storage_payload_candidate/hy-001.hwp
```

## 4. 정답 HWP 비교 결과

Stage 35에서 `hy-001` 내부 그림 `SHAPE_COMPONENT`는 정답 HWP와 byte-equal로 닫혔다.

```text
hy-001 #230 SHAPE_COMPONENT: equal
hy-001 #233 SHAPE_COMPONENT: equal
```

그러나 다음 record는 아직 hash가 달랐다.

```text
hwpx-h-03 #825 outer text-box rect SHAPE_COMPONENT
hwpx-h-03 #832 first inner picture SHAPE_COMPONENT
hwpx-h-03 #836 second inner picture SHAPE_COMPONENT
hy-001 #223 outer text-box rect SHAPE_COMPONENT
```

## 5. 추가 관찰

남은 차이는 구조체 필드 단위 출력에서는 모두 `same`으로 보였지만, SHA-256은 달랐다.
따라서 line/shadow/storage 계약은 대부분 닫혔고, 남은 문제는 사람이 읽는 값으로는 같아
보이는 binary precision 계열일 가능성이 높다.

다음 단계는 남은 `SHAPE_COMPONENT`에 대해 byte offset 단위 차이를 확인한다.
