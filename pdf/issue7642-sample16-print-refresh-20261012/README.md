# sample16 HWPX 독립 Print 기준 재산출

원본 `samples/hwp3-sample16-hwp5.hwpx`를 편집하지 않고 한컴 2024 MCP의 Print 0,
`frame_print_to_pdf_ex_one_up`으로 전64쪽 출력했다. 실행 제품은 13.0.0.3901이며 원본의
저장 제품은 13.0.0.3457이다. 입력·PDF 해시와 job/Print 실행 근거는 `print-provenance.json`에 있다.

#7722의 과거 정상 Print job은 조회할 수 없어 같은 원본을 다시 출력했다. 기존
`pdf/hwp3-sample16-hwp5-hwpx-2024.pdf`는 출처 대조용으로 유지하고 이 파일로 덮어쓰지 않았다.
새 Print 파일 확보와 현재 브랜치의 시각 통과는 별개다. 전체 Native/fresh WASM TSV와 직접
확대 판독은 진행 중이며 기존 기록의 완료를 현재 브랜치 완료로 대신하지 않는다.
