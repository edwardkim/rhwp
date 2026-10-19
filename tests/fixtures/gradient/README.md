# 원형 채우기 독립 Print 대조군

Issue: #7642

`radial-corner-print-contract.hwpx`는 한컴이 저장한 정상 HWPX 구조를 바탕으로 만든
6쪽 통제 입력이다. 글자/용지 속성을 보존하고 저장 줄 캐시를 제거했으며, 각 쪽의
단일 셀에 `RADIAL`, `step=50`, `stepCenter=50`, `#C8CCF8 → #FFFFFF`를 명시했다.

| 쪽 | 셀 너비 × 높이 (HWPUNIT) | 중심 x/y (%) |
| --- | --- | --- |
| 1 | 18000 × 18000 | 50 / 50 |
| 2 | 24000 × 12000 | 50 / 50 |
| 3 | 12000 × 24000 | 50 / 50 |
| 4 | 24000 × 12000 | 0 / 50 |
| 5 | 24000 × 12000 | 100 / 50 |
| 6 | 24000 × 12000 | 50 / 0 |

기준 `radial-corner-print-contract-2020.pdf`는 같은 바이트를 MCP `engine=2020`으로
한컴 `11.0.0.9136`에서 Print한 6쪽 PDF다. 실제 결과는 `pdf_print_method=0`,
`pdf_output_mode=hancom2020_pdf_driver_one_up`이며 job은
`08972b0f-ccf0-4ae8-8bbc-682250eb06e1`이다. PDF 저장/내보내기나 RHWP의 출력이 아니다.

- HWPX: 8,749바이트, SHA-256 `01884d8ea148e21fdb68194dd23c353c635903e4fc19d8b242e1efbe1d0aaf0d`
- PDF: 35,457바이트, SHA-256 `1f0a7707198e81b8bd9a93efe9e5e6a35fe3ffd9e0131ebb4851c71e9ec2d1a8`

진단 원본 `radial-gradient-contract-r3.hwpx` 및 그 Print PDF와 각각 바이트가 같다.
전쪽 Native/fresh WASM/실제 SVG 브라우저 출력의 실루엣은 모두 100%다.
96dpi 독립 PDF의 타원 path에서 얻은 중심·반경을 ±2 CSS px로 보호하고 셀 소유·내용
순서·누락·중복·포함·비겹침 검사도 유지한다. 테스트는
`tests/cases/verified_radial_print_geometry.rs`, 세부 검증은
`mydocs/working/task_m100_7642_stage1414.md`에 있다.
