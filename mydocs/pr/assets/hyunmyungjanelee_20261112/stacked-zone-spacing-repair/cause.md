# 단일 단 구역 경계의 마지막 줄간격 중복 소비

원 PR #8897의 공개 재현물 `whitespace-host-topbottom-stacked-zones.hwp`를 동일 입력의
Windows 한컴 2020 11.0.0.9136 MCP Print와 대조했다. 원 PR이 제시한 웹한글기안기 PDF는
판정에 사용하지 않았다. 공식 기준은 `pdf/issue8573/whitespace-host-topbottom-stacked-zones-hwp-2020.pdf`이며,
입력·PDF 해시와 작업 ID는 [Print 영수증](../mcp-print-reference/receipts.json)에 있다.

`3532e09de2`의 Native/fresh WASM 점수는 모두 99.06%였지만, 다음 단 구역의 Z·Y가 약 8px
아래로 밀렸다. Q·표·A의 위치는 유지됐다. 원본 구역의 단 사이 간격은 1134HU(96 DPI에서
15.12px)다. 다음 구역은 A의 글자 상자 하단과 이 간격을 한 번씩 소비해야 한다. 실제 배치는
직전 줄의 후행 간격 600HU(8px)를 추가했다.

선행 보정 `4d296dd1d26525ebe25fd981ad32589aa35ec3ff`와
`bacab82ca657c0351a801c3744b782e55c8f7000`는 저장 줄의 내용 하단을 확인하여 중복되는 꼬리
간격을 제외한다. 저장 줄이 없는 문단은 원본 줄 수가 0이므로 실제 구성의 마지막 줄 범위를
조회하지 못했다. 재조판 결과 전달용 필드와 메서드도 있었지만 실제 기록 호출이 없어 배치는
같은 확정 구성을 읽을 수 없었다.

보정은 원본 저장 줄 또는 수용한 재조판 줄에서 끝 줄과 간격을 읽는 공통 질의를 사용한다.
단일 텍스트 구역을 실제로 닫는 문단의 검증된 구성을 메모리 한도 계약에 따라 배치에 전달한다.
측정·배치는 내용 하단과 실제 흐름 끝의 차이가 해당 꼬리 간격인지 확인한 뒤 구역 간격을
소비한다. 저장 줄·개체·배분/평행 단·편집 중인 줄의 기존 소유 조건을 유지한다. 특정 문서나
페이지 식별자, PDF 좌표, 8px 상수에 따른 위치 이동은 추가하지 않는다.

새 관계형 검사는 A의 실제 줄 상자와 원본 `SectionDef.column_spacing`으로 Z의 위치 관계를
계산하며 A·Z·Y의 단일 소속과 순서를 확인한다. 수정 전 실제 4건 중 3 PASS / 1 FAIL이었다.
조회 경로만 연결한 중간 후보도 같은 이유로 3 PASS / 1 FAIL이었으며 수용 근거로 쓰지 않는다.
확정 구성 전달까지 보완한 후보는 최소 글꼴 집중 검사 4 PASS / 0 FAIL이며, Native 재출력은
같은 독립 Print와 100.00%다. [수정 후 review](after-native/review-001.png)와
[overlay](after-native/overlay-001.png)를 열어 Q·표·A의 위치 보존과 Z·Y의 간격 복원을 직접 확인했다.
[생산 SHA·파일 해시·실행 결과](after-native-validation.json)를 보존한다. 같은 소스의 fresh WASM도 같은 Print 기준 100.00%이며 [직접 판독 기록](../final-print-8fe7d1fb/wasm-direct-reading.json)에 Q·표·A 보존과 Z·Y 간격을 확인했다. 전체 회귀·최종 게이트는 별도 결과로 연결한다.
