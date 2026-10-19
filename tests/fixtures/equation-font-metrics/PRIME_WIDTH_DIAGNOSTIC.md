# 이슈 #7642 수식 폭 진단 입력과 한컴 Print PDF

사용자 요청에 따라 HWP와 그 파일을 직접 인쇄한 PDF를 함께 보존한다.
이 자료는 원인 진단과 실패 재현 자료이며, 원본 회귀 입력이나 통과한 golden을
대체하지 않는다. 실행 TSV·중간 JSON·MCP 응답은 Git에 포함하지 않는다.

## 수식 96개 진단

- 입력 생성: 기존 `equation-font-frame-matrix.hwpx`의 문단 구조를 사용하되,
  수식 상자 크기·기준선을 0으로 두고 저장 LineSeg 없이 생성했다.
  BaseUnit 600/700/900/1000/1200/1800별 16개 스크립트이며,
  소수 문자·괄호·관계 기호·미분 기호·삼각형·인용 문자열을 대조한다.
- 생성 입력: [HWPX](equation-prime-width-diagnostic.hwpx).
  SHA-256 `797e8fb9b49f827843e06054b376977823d9c3a3fa11f16cbac761191a3d63e6`.
- MCP 재생성: [HWP](equation-prime-width-diagnostic-2020.hwp).
  job `9f0280ee-6cc2-4429-9bc0-aac469821b95`, engine 2020,
  한컴 11.0.0.9136, 7초, SHA-256
  `03f38b458201832b4bab3cb2c040b6bb2640e125740c78277ed8be276afc1c44`.
- 위 HWP의 [Print PDF](../../../pdf/equation-font-metrics/equation-prime-width-diagnostic-print-2020.pdf):
  job `a22fb305-56a8-4349-ae25-f337a1860a64`, engine 2020,
  한컴 11.0.0.9136, `pdf_print_method=0`,
  `pdf_output_mode=hancom2020_pdf_driver_one_up`, 3쪽.
  SHA-256 `12057cf4571d018ea32db8546189712a2dce933fdbf3f4a7eaea92243b08d0d0`.

`rmP`와 `rmP'`의 새 상자 폭 차이는 위 여섯 크기에서 각각
150/225/300/300/375/450HU다. 한 크기의 차이를 고정 px나 일률적인
글자 크기 비율로 일반화하지 않는다. 새 HWP도 원본 문서의 통과 근거가 아니다.

## 원본 11월 문서의 별도 재저장 진단

- 원본은 [기존 HWP](../../../samples/3-11월_실전_통합_2022.hwp)를 유지한다.
- 별도 [재저장 HWP](../pr7600-equation-font-frame/nov-original-resaved-2020.hwp)는 MCP job
  `f633635e-aecc-41d9-94d4-08b8bdecf38e`, engine 2020,
  한컴 11.0.0.9136에서 생성했다. 원본 스크립트와 BaseUnit을 유지한
  1,680개 수식 중 69개의 상자 폭 또는 높이가 바뀌었다.
  입력의 저장 LineSeg는 실제 Print의 새 줄 구성과 다를 수 있다.
- 이 HWP의 [Print PDF](../../../pdf/pr7600-equation-font-frame-diagnostic/nov-original-resaved-print-2020.pdf):
  job `7c7c7945-708e-44ba-822c-897a3a8f860a`, engine 2020,
  한컴 11.0.0.9136, `pdf_print_method=0`,
  `pdf_output_mode=hancom2020_pdf_driver_one_up`, 21쪽.
  SHA-256 `9c737bf4a8700c5e31d0eb05bba16ea8e6bad4a3bfed7c3bf88334053abcbd79`.
- 원본 HWP에서 이전에 출력한 독립 Print PDF와 새 재저장 Print PDF의
  21쪽을 PDFium 96dpi로 비교했으며 모든 쪽의 RGB 픽셀이 같았다.
  파일 해시의 동일성이나 모든 출력 환경의 동일성을 주장하지 않는다.

## 판정과 실행 증거

[새 작업의 실행 기록](../../../mydocs/working/task_m100_7642_stage1.md)에
Native/fresh WASM source·패키지 해시, 전체 페이지 TSV, 대표 비교 PNG와
직접 판독 결과를 연결한다. 원본 입력과 재저장 입력의 결과는 구분한다.

수정 전 Native/fresh WASM은 96개 수식 문서 3쪽 모두 90% 미만이었다.
새 브랜치 `baea3726`의 Native는 최저 94.28874%로 개선됐다. 후속 미분 기호 face 연결 후보의 Native 최저는 94.00834%다.
fresh WASM의 검증은 실행 기록에서 별도로 확인한다. 재저장 문서의
21쪽 중 8쪽 미달은 별도 이슈 #7640에서 진행한다. 이 자료를 승인된 새 회귀 기대값으로 사용하지 않는다.
PR 생성·승인·통합 판정은 최종 head의 Native/fresh WASM 시각 보정 후에 한다.
