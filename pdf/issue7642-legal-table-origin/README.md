# 법령 표 원점 보정의 현재 기준 PDF

Issue: #7642

2026-10-18 작업지시자의 지시에 따라 새 MCP Print 출력을 현재 비교 기준으로 사용한다.

- 원본: `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx`
- 실제 입력 형식: HWP5. 파일 확장자와 실제 형식이 다르다.
- 입력 SHA-256: `5090e9d20ce75814febeb4b79e106ea6abfd2546e511f229900d137d42623617`
- 저장 제품: 한컴 2018, `10.90.0.983` (`rhwp info`의 저장 메타데이터).
- MCP에는 바이트가 같은 `.hwp` 사본을 전송했다. HWPX로 변환하거나 내용을 수정하지 않았다.
- client: `tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz`
- 실행: `start → status → download`, `--target pdf --engine 2020`.
- job: `61fd60b2-40a7-40a9-bda8-dafb9145d785`
- 실제 한컴: `11.0.0.9136`
- 인쇄: `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`.
- 결과: 74쪽, 806,237바이트.
- PDF SHA-256: `674ef63b51adc926c7c89f530bd142a45bb64cb655a7117bc20285d7a6e5a8d2`

이 폴더의 `78494-virtual-convergence-industry-decree-2020.pdf`를 Native/fresh WASM과
고정 devel/후보의 같은 조건 대조에 지정한다. `--compat 2022`는 2020/2022 조판 세대의
공개 CLI 선택값이며 실제 출력 제품 연도나 MCP engine 이름을 바꾼 표기가 아니다.

기존 `pdf/78494-virtual-convergence-industry-decree-2020.pdf`는 이전 수동 Print 비교의
과거 자료로 보존한다. 새 MCP PDF와 SHA가 다르며 이전 TSV를 새 기준의 점수로 재사용하지 않는다.
위여백만 바꾼 Print 대조군은 원점 규칙 진단용이며 이 원본 기준 PDF를 대체하지 않는다.
전쪽 측정·devel 대비 하락·수식/간격 직접 판독과 기존 회귀는 별도로 검증한다.
