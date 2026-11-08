# #7642 Canvas 원 글꼴 이름과 축소 필터의 공통 보정

Issue: #7642.

## 독립 근거

- #7970 `image-count`의 base/head RenderTree 요약과 글꼴 catalogue는 같다.
  실제 그림 축소만 낮은 품질로 소비한 진단은 문제 5쪽에서 base와 픽셀이
  완전히 같았다. 형제 `operators`의 두 하락은 이 변경으로 해결되지 않았다.
- `operators`의 실제 Canvas 글자 호출을 기록했다. 계측 대조군은 원래 PNG와
  픽셀이 같다. 원점·글자는 같지만 family 요청과 가로 변환이 달라졌다.
  `HCRBatang`/`HCRDotum`을 같은 프로그램의 family로 연결한 진단에서 1쪽
  raw는 96.76983%에서 96.91725%로 바뀌었다. base는 96.83833%다.
- PDF subset과 설치 글꼴의 실제 비어 있지 않은 윤곽선·advance를 비교했다.
  HCRBatang 147/147, HCRDotum 33/33, HCRDotum-Bold 57/57,
  Windows Batang 410/410이 일치했다. 정확한 파일 SHA·name 레코드는
  ignored stage2829에 있다. Windows Batang은 private stage2828에만 둔다.

## 잘못된 가정과 보정 범위

원본 전체 그림을 조금이라도 줄이면 high 품질을 강제한 가정을 제거한다.
선형 필터의 두 source 표본 범위를 넘는 minification에서만 넓은 필터를
요청한다. 원 픽셀 수와 실제 CTM의 장치 크기로 판단하며 원점·원문·DPI·
기준 PDF·점수 계산식은 바꾸지 않는다. 두 표본은 필터의 source 단위이며
조판 좌표를 고정하는 px 핀이 아니다. 특정 문서·쪽·그림 크기를 조건으로
삼지 않는다. source 일부를 자르는 기존 경로의 정책도 임의로 바꾸지 않는다.

기존 `installed_render_font_aliases`에는 실제 SFNT name 6의 `HCRBatang`,
`HCRDotum`과 name 1의 `HCR Batang`, `HCR Dotum`을 연결한다. 외형이
비슷한 Haansoft 계열을 동일 프로그램의 별칭으로 취급하지 않는다. 굵기와
기울임, 위치·전진폭 소비 계약은 유지한다. 이 변경으로 모든 PostScript
이름이나 미설치 font program이 해결된다고 주장하지 않는다.

## 진단과 최종 검증의 구별

stage2817/2819의 단순 low/medium 강제는 최종 보정이 아니다. medium은
문제 출력과 같았고 low는 가는 선의 대조군에 불리했다. stage2827의 source
필터 범위 진단은 `image-count` 전쪽에서 base 대비 0.01%p 이상 하락 0쪽이다.
stage2830에서 가는 선을 포함한 10월 시험지 HWP/HWPX의 전쪽을 함께 확인한다.
계측 PNG·전쪽 TSV는 ignored output에 보존하며 제품 통과로 쓰지 않는다.

새 source에서 Native/fresh WASM 출력, 원 Print PDF와 확대 판독,
전체 102개 문서·유입 PR 기능 보존·기준 하락·필수 Rust/lint를 재검증한다.
공유 target의 실행은 순차 유지하고 새 source의 실물 회귀·기준값은 해당
전쪽 시각 선행 조건 이후에만 추가한다.
