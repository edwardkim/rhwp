# #7032 HWPX 저장 줄의 독립 Print 확인

- 입력: `samples/hwpx/21761835_jeonjik_exemption_table.hwpx`, SHA-256
  `53b33bfca8d8a80534f27564ecee86403c6d1e3dc83cf405c7d6d158ae9625d2`.
- 입력의 `lastSavedWith`: 한컴오피스 2020, `11.0.0.9136`.
- 동일 HWPX의 한컴 2020 Print: `pdf/task2146/21761835_jeonjik_exemption_table-hwpx-2020.pdf`,
  SHA-256 `495fcddc4eed04918c776ae8b52931fde1923a59f8113fdfd9363b5311ad982f`.
  MCP 작업 `72b11ee2-5281-48ed-8fed-58fc0f72625f`, 실제 제품 `11.0.0.9136`,
  `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`, 6쪽이다.
- 독립 PDF의 임베디드 폰트는 `Haansoft Batang`이며, 원본의 1쪽 제목 글자모양은
  `나눔명조`를 요청한다. 로컬에서 한컴 Print의 출력 face를 재현하기 위해
  `hwpx-print-font-environment.json`을 사용한다. 파일은 face 이름만 기록하며 폰트를 배포하지 않는다.
- 제품 source `5fdccefdd8`과 같은 Native 실행 `7a982cead6b700a9516c70578405a7c95963dd6b2e654b33d5418cc60903bdcb`의
  96dpi PDFium 2px raw는 1쪽에서 기본 공급 96.42400%, 명시 공급 100.00000%다.
  이 결과는 1쪽 Native에 한정하며 다른 쪽이나 fresh WASM 성공을 뜻하지 않는다.

이전 `source8152-d2176`의 95~97% 비교는 HWPX 입력에 동일 원본 Print인지 입증되지 않은
기존 PDF를 사용했으므로 동일 HWPX Print와의 판정으로 사용하지 않는다. 기존 고정 좌표를
새 기준으로 재검토하려면 해당 1~6쪽의 Native와 fresh WASM 및 직접 판독이 모두 필요하다.
