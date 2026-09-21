# #6656 — 저장 줄 전진의 측정·배치 정합

## 분석

- 재현 입력은 `samples/hwpctl_ParameterSetID_Item_v1.2.hwp`이며, 독립 기준은
  `pdf/hwpctl_ParameterSetID_Item_v1.2-2022.pdf` 3쪽이다.
- 문단 0.7의 저장 사다리는 `ls[2] vpos=0, lh=1560, th=1000, ls=600`과
  `ls[3] vpos=1600`이다. 즉 다음 줄 전진은 1600HU(21.333px)지만, 줄 상자와
  간격의 합은 2160HU(28.8px)다.
- #6691은 layout의 실제 배치에 저장 사다리를 적용했지만, `TypesetEngine`과
  `HeightMeasurer`는 여전히 `lh + ls`를 합산했다. 이 차이는 page budget과
  render cursor를 서로 다르게 만들어 페이지 나눔의 잔여 회귀가 됐다.

## 수정

- `renderer::stored_line_flow_height`가 저장 사다리 사용 조건을 한 곳에서 판정한다.
  현재 줄 상자가 저장 `lh`와 같고, 사다리가 증가하며, 다음 줄 전진이 글자 또는
  글자처럼 취급되는 개체의 최소 높이를 침범하지 않을 때만 저장값을 쓴다.
- `LayoutEngine`, `TypesetEngine`, `HeightMeasurer`가 이 공통 판별자를 사용한다.
  재조판·마지막 줄·역행 사다리·너무 작은 advance는 종전 흐름 높이를 보존한다.
- 실제 HWP를 읽는 회귀 계약은 문단 0.7의 측정 advance가 저장값 21.333px와 같음을
  확인한다. 수정 전에는 `measured=28.8`, `stored=21.333`으로 실패했다.

## 검증 결과

- `cargo fmt --all -- --check` 통과.
- `cargo clippy --locked -p rhwp --lib -- -D warnings` 통과.
- `regression_suite_014` 210개 통과. 기존 3쪽 SVG 위치 계약과 새 실제 HWP 측정
  계약이 모두 통과했다.
- Native Visual Sweep: 3쪽 완료, 구조 후보 0건,
  `visual_accuracy_proxy_percent=81.36943`.
  - review: `/tmp/issue6656-native-sweep-20260921/issue6656/review/review_003.png`
  - overlay: `/tmp/issue6656-native-sweep-20260921/issue6656/overlay/overlay_003.png`
- fresh WASM Visual Sweep: 3쪽 완료, 구조 후보 0건,
  `visual_accuracy_proxy_percent=81.36943`.
  - review: `/tmp/issue6656-wasm-sweep-20260921/issue6656/review/review_003.png`
  - overlay: `/tmp/issue6656-wasm-sweep-20260921/issue6656/overlay/overlay_003.png`

기준 PDF와의 기존 glyph/그림 raster 차이는 overlay에 남아 있으나, 이번 변경으로
새 위치 drift 또는 overflow 후보는 관찰되지 않았다.
