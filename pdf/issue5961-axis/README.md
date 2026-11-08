# #5961 HWPX 저장 축의 독립 2020 Print 기준

- 원본: 저장소의 `samples/hwpx/aift.hwpx` 그대로. HWP/HWPX로 재저장하지 않았다.
- 원본 SHA-256: `5e27ee51825d10f0f51bd04935f0f9f8fc24f930a010367d822638964fefd491`
- 원본 저장 제품: 한컴 2020 `11.0.0.8969` (`rhwp info --json`의 `lastSavedWith`).
- PDF: `aift-2020.pdf`, 2,311,161바이트, 전체 74쪽.
- PDF SHA-256: `cccf1955714f60faf17eefee2f75dad184cbb6861b2c2cdba4c8c44308ab8794`
- 실제 인쇄 제품: 한컴 `11.0.0.9136`, MCP `engine: 2020`, `target: pdf`.
- 인쇄 경로: `pdf_print_method: 0`, `pdf_output_mode: hancom2020_pdf_driver_one_up`.
- 원본 용지·방향 유지, 한 쪽씩 인쇄(1-up). 원본의 모아 찍기 설정을 PDF 장 수로 혼동하지 않는다.

공식 client `tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz`로 원본을
`start --target pdf --engine 2020 --output-filename aift-axis-original-2020.pdf
--timeout-seconds 1800`에 전달한 뒤 `status → download`로 받았다. 저장소 보관 시 이름만
`aift-2020.pdf`로 정리했으며 바이트는 바꾸지 않았다. 인증 정보는 포함하지 않는다.

저장 축의 분석·비공개 대조군과의 구분은 `mydocs/working/task_m100_7642_stage3885.md`를
따른다. 이 PDF 추가나 parser의 파일 값 보존 검사를 aift 전체 74쪽의 시각 개선 완료로
간주하지 않는다. 전쪽 Native/fresh WASM TSV와 직접 판독·내용 보존은 해당 검증에서 판정한다.
