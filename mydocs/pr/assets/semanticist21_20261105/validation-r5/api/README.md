# 실제 공개 API의 base·후보 대조

고정 base `170b886f167b705b6e630c5b34bb638d92fae0a9`와 제품
`71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 최적화 WASM을 같은 드라이버로 실행했다.
[결과](api-before-after.json)는 12개 API 그룹 중 11개가 수정 전 실패·수정 후 통과한다.
PR #8602의 RHWP 재열기는 양쪽 통과하므로 원래 손상의 검출 근거로 인정하지 않는다.
[독립 CFB 레코드 대조](pr8602-independent-record-comparison.json)로 기존 빈 문단의
`char_count=1`·종료 CR만 있는 PARA_TEXT가 수정 후 없어지고 줄 시작 0이 보존됨을 확인했다.
현재 최소 글꼴 회귀의 `emptied_paragraph_para_text`도 이 계약을 검사한다.

[원시 실행 결과](actual-results.json)의 출력 경로는 당시 실행 위치다. 인쇄 입력과 정식 보존 경로는
[입력 매핑](../inputs.json)에 기록한다. 이 API 검사는 독립 Print·전쪽 시각 검증을 대체하지 않는다.

재현은 저장소 루트에서 `node mydocs/pr/assets/semanticist21_20261105/validation-r5/api/claim-probe.mjs
<최적화 WASM pkg 절대 경로> <output 아래 결과 절대 경로> <실제 제품 SHA>`로 실행한다.
[fixture 매핑](fixture-map.json)은 기존 정식 입력과 SHA를 고정한다. 수정 전 실행의 exit 1은
11개 결함을 검출한 대조군의 결과이며, 후보 실행은 12개 모두 통과했다.

독립 원시 레코드 판독은 `venv/bin/python mydocs/pr/assets/semanticist21_20261105/validation-r5/api/read-hwp-paragraph-records.py <수정 전 HWP> <수정 후 HWP>`로 재현한다. 이 드라이버를 실제 두 저장본에 실행해 보존한 레코드 표와 같은 결과임을 확인했다.
