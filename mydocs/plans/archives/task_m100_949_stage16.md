# Task M100-949 Stage 16 Plan

## 목적

Stage 15에서 구현한 `hp:pic@href -> HWP CTRL_DATA` 계약 반영이 실제 저장 산출물에서 유지되는지
`hwpx-h-01`, `hwpx-h-02`, `hwpx-h-03` 샘플로 검증한다.

## 원칙

- 새 추론 probe를 만들지 않는다.
- Stage 15 구현 결과를 그대로 사용해 저장 산출물을 만든다.
- 한컴 에디터 시각 판정은 작업지시자가 수행할 수 있도록 `output/` 아래에 파일을 둔다.
- CLI/진단 도구로는 record contract와 rhwp reload 가능 여부만 확인한다.

## 산출물

```text
output/poc/hwpx2hwp/task949/stage16_adapter_regression/
  hwpx-h-01.hwp
  hwpx-h-02.hwp
  hwpx-h-03.hwp
  generation.md
  ctrl_data_trace_hwpx-h-03.md
```

## 판정 요청 항목

```text
- 한컴 에디터 파일 열기 성공 여부
- 이미지 출력 여부
- 표/셀 배치 여부
- 셀 텍스트 클리핑 여부
- 마지막 페이지 출력 여부
- rhwp-studio 재열기/rendering 여부
```

