# 공식 Print 기준 PDF

현재 검토 입력 34종의 기준 PDF를 원본 입력과 함께 보존한다. 33종은 Windows 한컴 MCP의 `start → status → download`로 재산출했으며, 실제 engine·제품 버전·Print method 0·한 쪽씩 출력 모드를 `receipts.json`에 기록했다. 나머지 1종(#8916)은 MCP 2020이 문서 열기를 거절하여 사용자가 한컴 2024 13.0.0.3901의 **인쇄(Print)** 로 출력한 파일을 제공했다. 해당 사용자 확인과 실패 작업 ID를 구분하며 MCP 2020 성공으로 표시하지 않는다.

입력 HWP/HWPX는 한 바이트도 바꾸지 않았다. 입력·이전 Cairo/기존 PDF·새 공식 Print PDF의 SHA-256과 경로를 각 항목에 남겼다. 기존 기준 파일은 같은 입력의 새 `-hwp-2020.pdf`·`-hwpx-2024.pdf` 등 형식·엔진을 명시한 PDF로 교체했으며, 이전 파일 내용은 Git 이력에서 확인할 수 있다. 새 PDF의 생성 정보만으로 Print를 추측하지 않고 서비스 실행 기록 또는 사용자 확인을 연결했다. 인증 정보와 글꼴 바이너리는 포함하지 않는다.

이 커밋은 기준 PDF 보관이며 Native/fresh WASM·전체 회귀 완료 또는 PR 수용 판정을 의미하지 않는다. Cairo PDF에서 얻은 이전 점수와 새 Print PDF의 점수를 섞지 않는다.

자동 정답지 선택에도 같은 파일을 사용하도록 `tools/oracle_pdf_selection.py`의 `canonical_pdf_path` 규칙을 적용했다. 처음 보관한 형식 미표기 경로와 현재 정본 경로는 각 영수증의 `formal_path_before_canonical_name`·`pdf`로 연결하며 바이트·SHA-256은 동일하다. 사용자 제공 #8916은 저장 제품이 지시하는 기본 2020 엔진과 구분되는 수동 2024 Print 기준이며, 기본 엔진 원장에 2020 출력처럼 등록하지 않는다.
