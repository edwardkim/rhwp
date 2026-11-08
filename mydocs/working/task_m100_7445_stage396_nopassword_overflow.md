# 단계 396: 무암호 HWP5 본문 넘침 회귀의 기준 확인

## 사전 분석

- 전체 회귀 세 번째 실패는 `body_overflow_does_not_grow_partition_7`이다. 신규 입력은 `samples/HWP5-nopassword-123456.hwp`이며 기대값을 바꾸기 전에 독립 기준을 확보한다.
- 원본 저장 정보는 한컴 2024 `13.0.0.3901`이다. 현재 Native와 exact devel은 모두 24쪽이며, 기존 PDF는 HWPX 원본용이므로 HWP 기준으로 대신하지 않는다.
- 현재 Native CLI의 default/Skia 두 실행 결과는 모두 3쪽 마지막 `\WORK` 문단의 바닥 넘침 14.6133px을 보고한다. 여러 입력을 순회하는 전체 nextest의 라벨 없는 1쪽 진단을 이 파일에 연결한 것은 근거가 부족했다. 실행 환경 차이라고 단정하지 않는다.
- 공차·baseline을 완화하거나 새 회귀 검사를 추가하지 않는다. 같은 원문을 Native/fresh WASM으로 비교하고 문단 소속과 실제 한컴 출력으로 회귀의 타당성을 판단한다.

## 독립 Print 기준

- HWP 2024 client의 engine 2024, 인쇄 방식 0, 한 쪽씩 출력으로 24쪽 PDF를 생성했다.
- 서버 결과는 `hwp-managed-direct-dll-host`, worker 32bit, Hancom `13.0.0.3901`, 전처리 없음, font scope 검증 성공이다.
- 새 기준은 `pdf/HWP5-nopassword-123456-hwp-2024.pdf`이다. 기존 HWPX 기준은 보존한다.
- 입력·PDF 해시와 인쇄 결과는 `mydocs/pr/assets/issue7445/completed-20261009/nopassword-print-validation.json`에 기록한다. 새 PDF와 증적을 커밋에 포함한다.

## 비교 계획

exact devel 및 코드 후보의 Native/fresh WASM 전체 24쪽 TSV를 같은 PDF·DPI 96·로컬 글꼴로 비교한다. 캡처는 8개씩 수행하고 현재 로드맵 검증 종료 후 시작하여 캡처가 16개로 겹치지 않게 한다. Rust는 공용 target에서 순차 실행하며 기존 partition 7을 threads 8로 재현한다.

## 재현과 원인 경로 확인

- 코드 후보 `0c08289ffbe19c4c60f3ec31cabd4ea76238f125`에서 기존 partition 7은 0 PASS / 1 FAIL이다. 재빌드를 포함한 4.451초의 종료 코드는 100이며, 실패를 제외하거나 baseline에 추가하지 않았다.
- exact devel은 24쪽·본문 넘침 0건이다. 같은 코드 후보의 두 Native 실행은 모두 24쪽·본문 넘침 1건이다. 실제 마지막 줄 상자는 y=1007.7333px, 높이 16px이며 본문 바닥은 1009.12px이다.
- `1737aa751`은 HWP3 계보를 근거로 HWP5 원본 ParaShape 간격을 반감하고 저장 vpos를 차감하던 처리를 제거했다. 이 파일의 p25 이후 저장 원점 차이는 568HU이고, 현재 문단 뒤 간격은 원본 값이다. 원본 IR을 다시 훼손하여 과거 결과를 복원하지 않는다.
- `uses_hwp3_origin_flow_spacing_before`는 이미 원본 HWP5에서 앞 간격 사전 차감 생략을 사용하지 않는다. `vpos_corrected_end_y`의 오래된 설명만 보고 이 플래그가 켜져 있다고 판단하거나 다시 바꾸지 않는다.
- 전체 Print 비교가 끝나면 원본 저장 줄의 간격·인라인 도형 및 표 점유·실제 흐름 커서가 같은 물리 공간을 한 번 소비하는지 확인한다. 아직 완료·회귀 제거·PR 준비 완료로 판단할 근거는 없다.

## 전수 비교 결과

- exact devel `21ed64071`의 Native/fresh WASM 전24쪽은 최저 44.28514%, 90% 미만 12쪽이다. Native와 실제 Chrome WASM의 TSV 수치가 일치한다. 독립 기준은 이번에 재산출한 한컴 2024 Print PDF다.
- 이전 코드 후보 `b60dca726`의 Native/fresh WASM 전24쪽은 최저 51.57501%, 미달 2쪽(2쪽 88.73774%, 3쪽 51.57501%)이다. 새 글리프 폭·수직 정렬 후보의 최종 증거로 대신하지 않는다.
- baseline WASM에는 조판 세대 setter가 없어 최신 exporter가 거부했다. `21ed64071`의 생성자 기본값 `hangul2024_compat=false`를 소스에서 확인하고 기존 진단 exporter의 2022 전용 경로로 실행했다. setter를 가짜로 주입하거나 조판 세대 2024를 주장하지 않는다. package·비교 알고리즘은 변경하지 않았다.
- 전수 TSV 4종과 입력·PDF·package·exporter 출처를 `mydocs/pr/assets/issue7445/completed-20261009/nopassword-base-head-visual-validation.json`에 기록했다.
- base 자체의 시각 미달과 현재 수정이 새로 만든 3쪽 넘침은 구분한다. 기대값 완화나 원문 제외로 새 넘침을 숨기지 않고, p25 표 이후 문단 간격·표 바깥여백의 흐름 중복을 구현에서 해결한다. 완료·회귀 제거·PR 승인 가능한 문서로 판정하지 않았다.
