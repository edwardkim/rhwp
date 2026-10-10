# 단계 396: 무암호 HWP5 본문 넘침 회귀의 기준 확인

## 사전 분석

- 전체 회귀 세 번째 실패는 `body_overflow_does_not_grow_partition_7`이다. 신규 입력은 `samples/HWP5-nopassword-123456.hwp`이며 기대값을 바꾸기 전에 독립 기준을 확보한다.
- 원본 저장 정보는 한컴 2024 `13.0.0.3901`이다. 현재 Native와 exact devel은 모두 24쪽이며, 기존 PDF는 HWPX 원본용이므로 HWP 기준으로 대신하지 않는다.
- 현재 Native CLI는 3쪽 마지막 본문 줄의 바닥 넘침을 보고한다. 전체 nextest 로그의 1쪽 조각 넘침과 위치가 다르므로 CLI·검사 실행 환경 차이를 먼저 확인한다.
- 공차·baseline을 완화하거나 새 회귀 검사를 추가하지 않는다. 같은 원문을 Native/fresh WASM으로 비교하고 문단 소속과 실제 한컴 출력으로 회귀의 타당성을 판단한다.

## 독립 Print 기준

- HWP 2024 client의 engine 2024, 인쇄 방식 0, 한 쪽씩 출력으로 24쪽 PDF를 생성했다.
- 서버 결과는 `hwp-managed-direct-dll-host`, worker 32bit, Hancom `13.0.0.3901`, 전처리 없음, font scope 검증 성공이다.
- 새 기준은 `pdf/HWP5-nopassword-123456-hwp-2024.pdf`이다. 기존 HWPX 기준은 보존한다.
- 입력·PDF 해시와 인쇄 결과는 `mydocs/pr/assets/issue7445/completed-20261009/nopassword-print-validation.json`에 기록한다. 새 PDF와 증적을 커밋에 포함한다.

## 비교 계획

exact devel 및 코드 후보의 Native/fresh WASM 전체 24쪽 TSV를 같은 PDF·DPI 96·로컬 글꼴로 비교한다. 캡처는 8개씩 수행하고 현재 로드맵 검증 종료 후 시작하여 캡처가 16개로 겹치지 않게 한다. Rust는 공용 target에서 순차 실행하며 기존 partition 7을 threads 8로 재현한다.
