# 단 너비 정수화와 저장 줄

`header.hwpx`는 rhwp의 빈 문서에서 만든 독립 합성 입력이다. 사용자 서식에서 추출한 데이터는 없다.
문구는 `Left 10`, 공백 95개, `Right`이며, 단 너비와 저장 줄 너비는 36002 HWPUNIT이다.
정수화된 단 너비 36000과의 차이만으로 저장 줄을 다시 나누면 `Right`가 다음 줄로 이동한다.

이 입력은 저장 줄 재사용의 계약 검사다. 한컴에서 작성하거나 출력한 문서로 주장하지 않는다.
회귀는 0~3 HWPUNIT 잔여값, 실제 1 HWPUNIT 폭 축소, 편집 입력 무효화를 구분한다.
합성 배치 계약은 왼쪽 여백에서 시작하는 `Left 10`이 단 너비의 왼쪽 1/4 안에 모이고,
`Right` 전체가 같은 줄의 오른쪽 1/4 안에 놓이는 것이다. 기대 영역은 단의 물리적 경계로
정하며, 구현의 출력 좌표를 golden 값으로 복사하지 않는다. 공개 `charX`로 각 문구의
실제 범위를 검사하여 두 문구가 하나의 run으로 반환되어도 가로 배치 오류를 검출한다.

## 2026-09-15 메인터너 독립 PDF 검토

기존 HWPX 바이트를 변경하지 않고 hwp2024-mcp-convert의 `start` → `status` → `download`로
한컴 PDF를 만들었다. `status=succeeded`, engine 2020, Hancom 12.0.0.4605, 32bit direct DLL host,
전처리 없음, one-up 출력, 1쪽, 서버 경과 8초다. job ID는 `5ba21e32-ca5c-43c2-88a8-ab8f2e7c1776`다.
입력의 `lastSavedWith`는 2020(11.0.0.3524)이지만 rhwp 합성 입력에서 상속된 메타데이터이며,
입력을 한컴이 작성했다는 뜻은 아니다. PDF 출력만 한컴으로 독립 확인했다.

한컴 PDF는 `Left`와 `10`을 첫 줄 양쪽에, `Right`를 다음 줄에 배치한다. 현재 합성 계약은 `Left 10`을 왼쪽에 모으고 `Right`를 같은 줄 오른쪽으로 보낸다. 따라서 이 PDF는 기존 기대값을 승인하는 golden이 아니라, 실제 출력과 계약의 불일치를 기록하는 독립 검토 자료다.

- 입력: `tests/fixtures/stored_column_width_quantization/header.hwpx`
- 입력 SHA-256: `b0ac06d1333b9c8ea263ec184e782d58d043916b7e6b9a6f39dfdeb31102437b`
- 기준 출력: [pdf/stored-column-header-2020.pdf](../../../pdf/stored-column-header-2020.pdf)
- PDF SHA-256: `bb8d858c102d2ffd858c182ea721b36e3225fbc9d3a2ea289fdb411679a88368`

원 PR이 설명한 실제 사용자 문서는 이 fixture에 포함되어 있지 않으며, 그 원본의 동일성이나 개선은
위 합성 문서 검증으로 대신 증명하지 않는다. 기준 PDF·원본 파일은 이름을 바꿔 중복 추가하지 않았다.
