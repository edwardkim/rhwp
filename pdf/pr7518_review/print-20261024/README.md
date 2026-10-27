# PR #7518 보정의 독립 한컴 Print PDF

2026-10-24 검증에 사용하는 원본 입력을 변경하지 않고 공식 MCP의 한컴 2020
엔진 11.0.0.9136에서 Print method 0(1-up)으로 전체 출력했다. HWPX 변환본이나
RHWP 재저장본의 출력으로 대체하지 않았다. 이전 PDF는 그대로 보존한다.

입력·PDF 경로와 SHA-256, 엔진·인쇄 방식·전체 쪽수·job ID는
[provenance.json](provenance.json)에 기록했다. 실행 로그와 임시 중간 파일은
ignored output에 남기며 이 경로에 포함하지 않는다. 이 PDF 산출 자체가
RHWP 시각 통과나 회귀 기대값의 적합성을 입증하는 것은 아니다.
