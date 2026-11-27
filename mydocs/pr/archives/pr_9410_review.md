# PR #9410 자체 검토 — 한컴 2024 일반 인쇄 영수증

2026년 11월 27일, 작성자 postmelee가 [PR #9410](https://github.com/edwardkim/rhwp/pull/9410)의 코드 head `a360c5e10958a6e7035ce7ac4b66bc1374b08682`를 검토했다. 판단은 승인이다. collaborator self-merge 경로와 review-only trailing 절차를 적용했다.

## 확인한 동작

한컴 2024 일반 인쇄와 기존 PrintToPDFEx의 출력 모드를 영수증에 구분해 보존한다. 갱신기의 요청 옵션과 상태·다운로드 응답에서 같은 작업 ID, 엔진과 빌드, 일반 인쇄 모드, 정수 방식 0, 무전처리 표식 및 쪽수를 대조하는 경로를 확인했다. 다운로드 크기·SHA-256을 실제 PDF에 대조하고, 독립 감사에서 현재 원본과 PDF의 해시를 다시 확인한다. 불일치 시 기존 PDF를 교체하기 전에 중단한다. [서비스측 재현](https://github.com/edwardkim/rhwp/issues/9370#issuecomment-7106131512)

## 재현한 검증

- Python 3.14에서 `python3.14 -m unittest discover -s tools/oracle_public/tests` 51건이 통과했다. 새 일반 인쇄 경로는 모의 client로 검증했다. 작업 ID·크기·해시 불일치, 전처리, PDF 대체 저장, 실패 시 기존 파일 보존을 포함한다.
- 코드 head의 [기존 영수증](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)에 대해 `tools/oracle_public/audit_print_pairs.py`를 다시 실행했다. MCP Print 33쌍과 수동 Print 1쌍에서 원본과 PDF 총 68개의 실제 해시가 일치했다. 이 34쌍은 기존 형식의 호환성 근거다. 새 서비스 경로로 새로 출력한 실적은 이 검증에 포함하지 않았다.
- 코드 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/45564684488)에서 필수 Build & Test가 통과했다. 변경 범위는 PDF 갱신·감사 도구, 검사와 매뉴얼이다. 엔진 렌더링 결과나 충실도 수치를 갱신하지 않았다.

작업지시자가 이 PR의 병합을 승인했다. 이 기록을 포함하는 최종 head의 필수 검사 성공과 병합 가능 상태를 확인한 후 병합한다.
