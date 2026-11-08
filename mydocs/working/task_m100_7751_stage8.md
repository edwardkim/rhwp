# #7751 SVG 원본 크기와 Canvas crop — 8단계

시작 HEAD `51139a44f`, 기준 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`.
7단계 변경은 커밋했으며 해당 소스의 전체 Rust 검증은 계속 실행 중이다.

## 독립 근거와 원인

편람130쪽 그림376은 원본 HWP에 crop `(0,391,57840,50640)`가 있고,
그림 기준 크기는 `(57840,50640)`HU다. 생산 SVG의 viewBox는771×675이며
그 안의 raster는770×674다. 원본·기존 독립 KoPub 한컴 Print는 바꾸지 않는다.

Native는 SVG를 목적 크기로 raster화한 뒤 `compute_image_crop_src`를 사용한다.
Canvas는 `parse_image_dimensions_canvas`가 PNG/JPEG/GIF/BMP만 받기 때문에
SVG의 crop 분기를 건너뛴다. `draw_image_cropped`도 알 수 있는 원본 픽셀 좌표가
필요하다. 크기가 없는 SVG를 Chrome에서 읽으면 natural size가171×150으로
나오고, 그 값만 source rect에 넣으면 그림 일부가 확대된다. viewBox와 일치하는
명시적 width/height와 동일 좌표계의 crop 크기가 함께 필요하다.

공개 layer JSON의 crop·originalSizeHu를 읽는 진단에서 두 조건을 함께 적용했다.
실제 적용 source rect는 `(0,5.2117891,771,669.7882109)`이고, 기존 WASM의
원본130쪽 raw는93.34522%에서97.94843%로 개선됐다. 제품 코드가 아닌
브라우저 진단 결과이며 최종 fresh WASM 검증으로 대신하지 않는다.
`p130-svg-crop-diagnostic-scores-r2/silhouette.tsv`,
`logs/p130-svg-crop-diagnostic-r2.log`에 입력·적용 좌표·실행을 보존했다.

## 생산·소비 경로와 수정 계획

1. 색 처리한 단일 raster WMF payload의 공통 producer가 최종 viewBox와 같은
   명시적 원본 픽셀 width/height를 출력하게 한다. raster의 위치·크기·viewBox와
   색 처리는 유지한다.
2. 공통 SVG 도구에서 명시적 정수 px 크기를 판독하고 Canvas의 원본 크기
   판독이 이를 소비하게 한다. 퍼센트·정의되지 않은 크기를 추측하지 않는다.
3. 기존 공통 `compute_image_crop_src` → `draw_image_cropped` 경로를 사용한다.
   문서 ID·391HU 등의 사례 수치를 제품 분기에 추가하지 않는다.
4. 크기 판독 계약과 실제 WMF producer를 검사하고, fresh WASM130쪽을 먼저
   비교한다. 전체 페이지와 다른 영향 문서의 Native/fresh WASM 무회귀도 확인한다.
   130쪽은 진단에서도98% 미만이므로 px 회귀를 추가하지 않는다.

이전 intrinsic-only 진단은 크기를 실제로 바꿨지만 crop 분기가 없어서 효과가
없었다. 후속 계측으로 실제 SVG src·771×675 변경을 확인했다. 정규식 이스케이프가
원인이라는 중간 추정은 바이트 재확인에서 기각했으며 제품 수정 근거로 쓰지 않는다.

224·226쪽의 devel 대비 하락은 별도 보정 대상으로 유지한다.130쪽의 개선으로
다른 쪽의 하락이나 남은 필수 검증을 상쇄하지 않는다.

## 구현과 정식 소비 경로 회귀

`wmf_picture.rs`에서 효과를 적용한 SVG에 원본 창의 width/height를 명시한다.
`svg_fragment::svg_intrinsic_pixel_dimensions`는 루트의 명시적 정수 px 크기만
판독한다. Canvas의 기존 크기 판독 마지막 경로에서 이를 소비하므로,
`compute_image_crop_src`와 실제9인자 `drawImage`가 같은 좌표계를 사용한다.
퍼센트·불완전한 크기·자식 image의 크기를 원본 픽셀 크기로 추측하지 않는다.

새 `scripts/tests/wmf-picture-crop.browser.mjs`는 실제 WASM·편람130쪽을 실행하고
실제 Canvas 호출을 관찰한다. source rect를 저장 IR의 crop·기준 크기와 원본
SVG 창에 대조한다. 기존 CI의 `visual-sweep-canvas-fonts.browser.mjs` 진입점에서
import하며 ci.yml에 개별 호출을 추가하지 않는다. 이 검사는 이전 v2 WASM에서
crop 없는5인자 호출을 관측해 의도한 assertion으로 FAIL했다(15.26초, exit1).
`logs/wasm-wmf-crop-regression-red.log`에 보존했다.

Native의 기존 정식 WMF payload 검사도 원본 SVG 창과 효과 payload의 명시적
픽셀 크기가 같음을 확인한다. 여기서 원본 창 크기는 배치 px 핀이 아니라
실제 저장 WMF의 원본 그림 좌표계 계약이다.

7단계 전체 Rust 검사는51139a44f의 컴파일이 끝난 바이너리에서 계속 실행 중이다.
이번 편집 이후의 전체 검증 통과로 재표기하지 않는다. 새 생산 소스의 fresh WASM,
브라우저 회귀와 시각 검증 결과는 완료 후 별도로 기록한다.

## 2026-10-17 실행 결과와 남은 검증

- R2 fresh WASM 빌드 exit0,342.02초. `stage8-source-provenance-r2.json`과
  `stage8-fresh-wasm-provenance.json`에51139a44f+변경4파일 SHA를 고정했다.
  첫 빌드의 quick_xml0.42 API 사용 오류는 수정했고 실패 로그도 보존했다.
- 같은4파일의 SHA를 확인한 Native CLI 빌드 exit0,7분25초.
  `stage8-native-provenance.json`에 실행 파일·소스 해시를 보존했다.
- 실제 fresh WASM130쪽 raw97.94843%, 진단과 동일하다.
  `stage8-wasm-review/handbook/overlay/overlay_130.png`를 직접 판독했다.
  명시적 원본 크기와 crop을 함께 적용한 후 이전 그림의 세로 어긋남은 사라졌다.
  미세 차이는 남으며98% 미만이므로 이 쪽의 절대 배치 좌표는 고정하지 않았다.
- 정식 브라우저 진입점은 HFT·WMF crop·글꼴 PNG3검사 모두 PASS,
  61.56초(`logs/stage8-browser-font-crop-tests.log`).
- Native Skia lib는 본체3861PASS/12ignored, 나머지 workspace182PASS,
  합계4043PASS/0FAIL(`logs/stage8-native-skia-lib-tests.log`).
  기존 PageBackground2검사는 이전 filter ID 대신 실제2×2 PNG의 독립 RGB·alpha와
  중복 효과 부재를 확인해 통과했다. WMF 검사의 검정 픽셀 수 임의 하한100은
  원본에 맞지 않았다. 원본의 모든 검정 픽셀 보존과 비어 있지 않음을 검사한다.
- WASM383쪽 TSV 완료: 최저90쪽92.45371%,90% 미만·누락0쪽.
  기준552514f42와 비교한226쪽99.20548→99.17155%(−0.03393%p)는 남는다.
  `stage8-wasm-base-head-comparison.json`에 Decimal 비교를 보존했다.
- Native383쪽은 PNG 출력이 완료됐고 이 기록 시점에 PDF raster/TSV를 산출 중이다.
  변경된 통합 검사·Skia placeholder/direct PDF 검사와 최종 lint도 진행/대기 중이다.
  현재 단계의 통과로 전체 PR 준비 완료를 선언하지 않는다.

앞서51139a44f 전체 Rust는10467실행/10458PASS/9FAIL/45SKIP로 종료됐다.
PNG 색 처리 표현을 요구하는 기존 검사, 원본 bright/contrast 교환 기대값,
WMF 임의 검정 픽셀 수와 실제 배치 회귀를 분리해 처리한다. HWPX image354의
원본 XML bright=8/contrast=0을 확인해 저장 후 속성 검사도 원본 의미로 정정했다.
`hwpspec.hwp`의 신규 text-overlap은 기준에서0건/현재1건이므로 이관하지 않는다.
제목 pi346과 뒤 표 pi347의 공통 흐름 원점을 다음 단계에서 보정한다.

600dpi device grid와 정수 PDF 용지 투영은 독립 괘선에서 관측했으나, 선 endpoint만
적용한6쪽 진단은226쪽을0.00087%p만 개선했다. `print-grid-line-diagnostic-decision.json`
참조. 현재 보류 해소 근거가 아니며 제품에 전역 좌표 보정은 추가하지 않았다.
