# #7751 Print 물리 종이와 논리 조판 경계 — 11단계

시작 HEAD `be2bc0e67`.10단계 RGB 보정·작은 실제 한컴 저장본·독립 Print PDF와
정식 공개 backend 검사를 커밋한 뒤 시작한다. 편람 전쪽의 devel 대비 하락을
실제 출력 계약으로 보정하고 최종 PR 검증을 계속한다.

## 독립 근거와 원인

원본 `samples/2025 행정업무운영 편람(최종).hwp`의 종이는55559×75402HU다.
독립 KoPub2024 Print PDF383쪽의 MediaBox는 세로555×754pt309쪽,
가로754×555pt74쪽이다. 한컴이 실제 인쇄한 물리 종이는 원본 HU/100을 정수
point로 내린 크기이며, 글줄·표·그림의 논리 배치 좌표는 원본 종이를 사용한다.
기존 Print layer tree는 논리740.7867px를 그대로 MediaBox/PNG 크기로 넘겼다.
Canvas/Skia bitmap의 올림 결과는741px지만 독립 Print의 너비는740px다.

공개 DocumentCore/LayerBuilder로 실패를 재현했다. 수정 전 Print가555pt 대신
555.59pt를 노출해 회귀 후보가 실제 assertion으로 실패했다. 초기 진단 코드의
컴파일 오류는 RED로 세지 않았다. `print-paper-red-result-r3.json`과 대응 실행
로그에 compile0/test101 및 실제555.59 관측을 보존했다.

## 변경할 생산 결과와 소비 경로

`LayerBuilder::build_with_embedded_fonts`에서 Print의 출력 종이 크기만 독립
MediaBox 계약에 맞춘다. source PageNode, root bounds, 본문·표·그림·글줄 좌표,
pagination은 그대로 논리 종이를 사용한다. 화면/fast preview/high quality의
논리 viewport도 유지한다. 축소하거나 객체 원점을 clamp하지 않는다.

생산한 `PageLayerTree.page_width/page_height`는 실제로 다음이 함께 소비한다.

| 경로 | 실제 소비 위치 |
| --- | --- |
| Native PNG | Skia `render_raster_with_options`의 surface 크기 |
| fresh WASM Canvas | `render_page_to_canvas_filtered_with_profile_impl`의 bitmap 크기 |
| Print SVG | SvgLayerRenderer의 `begin_layer_page` |
| SVG PDF | DocumentCore가 같은 layer tree로 SVG를 만들고 PDF 변환 |
| direct PDF | `layer_trees_to_pdf`의 page dimension → MediaBox |

따라서 비교 PNG를 잘라 점수를 바꾸는 진단을 제출 결과로 쓰지 않는다.
Visual Sweep/DPI/허용치/마스크는 바꾸지 않고 제품의 Print 출력을 재생성한다.
완료 페이지의 독립 종이 크기와 화면의 논리 bounds 보존을 정식 검사에 넣는다.

## 검증 순서

1. 기존 편람 경계 focused 검사로 Native producer를 준비하고 공용 target 산출물을
   ignored output에 동결한다. Cargo가 만든 실제 CLI의 바이트와 source SHA를 기록한다.
2. Native 전383쪽, fresh WASM 빌드 후 전383쪽을 같은 독립 PDF·글꼴·96dpi·Print·2024로
   비교한다.90% 미만·누락·쪽수 차이와 fixed devel 대비0.01%p 하락을 따로 판정한다.
3. 새 입력5개도 최종 출력으로 다시 확인한다. RGB 회귀와 실제 Canvas의 조건·색 계약을 유지한다.
4. 양쪽 검증·확대 판독을 마친 뒤 종이 계약의 정식 회귀를 추가한다. 기존 px 핀을
   바꿔 통과시키지 않으며 실제 출력 종이와 논리 배치의 구분을 검사한다.
5. 최종 code head의 전체 Rust·Native Skia3종·lint·fresh WASM·Studio·시각 증적과
   archive review/오늘할일·PR/CI/merge 후속 처리를 canonical workflow에 따라 마친다.

`.936em`만 적용한 KoPub 실험은 철회된 상태다. 동일 글꼴 SHA의 Windows GDI
장평 양자화 관측은 별도 조사 증거이며 이 종이 보정에 폭 상수 변경을 섞지 않는다.

## Native 후보의 선행 확인

편람의 기존 경계 focused5검사가 Native에서5PASS/0FAIL로 완료됐다.
세로·가로를 함께 확인한 독립 종이 계약 진단도 compile0/test0이다.
수정 전555.59pt로 실패한 검사가 수정 후555pt와 가로754pt를 확인했고,
화면의 원본 HU 크기와 root bounds는 그대로임을 검사했다. 진단 로그는
`print-paper-green-run.log`에 보존했다.

이는 전383쪽 gate나 최종 완료 판정이 아니다. 이 생산 후보를 먼저 고정한 뒤
동일 코드의 Native/fresh WASM 전383쪽 TSV와 devel 대비 하락을 확인한다.
Cargo의 집중 검사 실행이 함께 만든 실제 CLI를 source 파일 SHA와 바이트 SHA로
동결했다. 실행 파일만 새 경로에 복사했으며 공용 target은 그대로 재사용한다.

## 전383쪽 결과와 재검토

- Native 최저93.07066%, fresh WASM 최저92.98523%, 양쪽383쪽/누락0/90% 미만0.
- 그러나 devel 대비0.01%p 하락은 Native8쪽, WASM8쪽이다. 공통7쪽과
  Native177/WASM205를 포함한9쪽을 별도로 분석했다. PR 생성·승인·통합은 보류한다.
- 이9쪽의 기존9단계 PNG와 현재 PNG에서 물리 종이 안쪽의 변경 픽셀은0개다.
  예를 들어57쪽의 content union은105017→104167로850 줄었다.
  종이 밖1열의 제거가 매칭되는 내용 픽셀 분모를 줄였음을 확인했다. 기준을 완화하지 않는다.
- Native 전체 PDF raster383쪽을 SHA·96dpi·thinlinemode=shape로 고정해 WASM에서
  재사용했다. 글꼴 catalog492faces의 실제 프로그램 hash를 모두 재검증한 뒤 재사용하고,
  WASM PNG383쪽은 새2d08313b6 package로 전부 다시 출력했다. 과거 RHWP PNG는 재사용하지 않았다.
  실제 명령·PDF PNG별 SHA·입력과 package SHA는 source_provenance.json에 보존했다.
- 고정 devel 참조는fe5725dc3, 기존 기준 producer는552514f42다. 두 commit의
  renderer/paint/parser/model/document_core source diff가 없음을 다시 확인했다.
  실제 기준 producer SHA를fe로 바꾸어 표시하지 않았다.

## 다음 원인 계층의 독립 관측

한컴 PDF130쪽의 바탕쪽 JPEG 전체 폭은740px이고 Native 바탕쪽 그림은740.7867px다.
같은 원본 그림을 자른 제목 부분은 PDF `(158.708,74.552,511.4467,74.0733)`px,
Native `(158.84,74.6267,512.0267,74.0533)`px다. 물리/논리 종이 비율로 raster
그림의 목적 사각형을 투영한 진단은 위치·크기를 독립 출력에 가깝게 했다.
동적 제목 텍스트는 별도 GDI 원점을 사용하므로 함께 축소하지 않는다.

이 진단의 비투영 대조군15쪽은 공식 Native PNG와 픽셀이 완전히 같았다.
그림 목적 사각형만 바꾼 진단에서는 위9쪽의 devel 대비 하락이0.003%p 미만으로
줄었고 다른 대표 경계를 악화하지 않았다. 이는 제품 검증을 대신하지 않는다.
바탕쪽 raster 그림의 인쇄 기준 프레임을 공통 producer에서 명확히 하고,
실제 Native/fresh WASM 전체 재검증을 다음 단계에서 수행한다.
