# #1950 용지 고정 꼬리말과 마지막 본문의 독립 Print 기준

원본은 저장소의 `samples/issue1950_hwp3_tab_charoffset.hwp`이며 재저장·내용 변경 없이
한컴 2020 MCP에서 PDF로 인쇄했다. 기존 기준 PDF 두 파일은 그대로 보존한다.

| 항목 | 값 |
| --- | --- |
| 원본 SHA-256 | `bf004fec9d54e1d723282ab85a0c2b072f1efae6ba28e677a06d7aeb93638803` |
| 출력 PDF | `issue1950_hwp3_tab_charoffset-2020.pdf` |
| PDF SHA-256 | `827cd73fd393731a4b7ede66ab27809428b97d29ec0595d634b2da1b4b79cec5` |
| 바이트 수 / 전체 쪽수 | 56,022 / 1쪽 |
| engine / 실제 한컴 | `2020` / `11.0.0.9136` |
| Print 설정 | `pdf_print_method: 0`, `pdf_output_mode: hancom2020_pdf_driver_one_up` |
| MCP job | `f0c2dfa2-3620-4cd0-9f1f-ab7f7fa7b0df` |
| 용지·배율 | 원본 용지·방향 유지, 한 쪽씩 인쇄(1-up) |

공식 client `tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz`의
`hwp2024-mcp-convert start --input samples/issue1950_hwp3_tab_charoffset.hwp --target pdf
--engine 2020 --output-filename issue1950_hwp3_tab_charoffset-2020.pdf --timeout-seconds 1800`
이후 같은 job의 `status → download`로 산출했다. 인증은 Git 밖의 환경 파일로만 공급했다.

첫 쪽에 제목, 수신·날짜, 본문 표, 보고 문장, 서명, 마지막 본문 `2 - 48`, 용지 아래쪽 표
`- 581-35 -`가 각각 보존된다. PDF 텍스트 좌표를 기존 한컴 PDF와 대조했으며 주요 내용과
좌표가 같다. `2 - 48`을 다음 쪽에 놓는 출력은 이 독립 기준과 일치하지 않는다.

96dpi PDFium 및 Native/fresh WASM의 입력·기준 SHA, 명령·전쪽 TSV·대표 PNG와 판독은
`mydocs/working/task_m100_7642_stage3883.md`와 후속 검증 기록에서 연결한다.
PDF·쪽수 일치만으로 렌더링 전체의 완료를 주장하지 않는다.
