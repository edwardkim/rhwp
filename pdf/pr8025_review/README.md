# 기존 월간 수출입 원본의 독립 한컴 Print PDF

- 원본: `samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp`.
- 원본 SHA-256: `83fca0151c67e2adeb7a85203f5c75204d8e53af4e44902958c96b85adf6845e`.
- [기존 Print PDF](monthly-trade-original-2020.pdf): SHA-256 `cd3798dd11f32ae9ccb0e6f300005c9f6fd0bf141b5349920e0a5cd1a0340912`, 786,224바이트, 19쪽.
- 원본 저장 제품은 한컴 2020 `11.0.0.8362`다. 2026-10-19에 원본 그대로 Windows MCP의 engine 2020·파일 → 인쇄(Print) 경로로 출력한 기존 자료를 바이트 그대로 복사했다. 새 입력 생성·편집·PDF 재출력은 하지 않았다.
- 실제 완료 job은 `44009145-e7a6-445e-86f3-3bc1d695337c`다. 완료 응답의 `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`와 입력/PDF 해시를 재확인했다. PDF Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`은 출력 출처 표기이며 손상 판정 근거가 아니다.
- 이전 보관 위치는 ignored `pdf/semanticist21-20261019/font-trace-originals/`였으므로, PR에서 독립 기준에 접근할 수 있게 이 정식 경로에 등록했다. 다른 이름의 2020·2024 PDF와 바이트가 다르므로 서로 섞지 않는다.
- 기존 font trace 검증 대상 안에서 #8025의 실제 HY헤드라인M 제목(4·8·10쪽)을 확인한다. 새로운 HWP 문서를 검토 범위에 추가하지 않는다. 실제 Native/fresh WASM 비교 및 직접 판독은 별도 결과가 나오기 전까지 완료로 표시하지 않는다.

MCP 인증·접근 정보와 원격 내부 경로는 기록하지 않는다. 원본과 PDF의 대응·명령·producer·최종 TSV·review·overlay는 개별 PR 검토 기록에 연결한다.
