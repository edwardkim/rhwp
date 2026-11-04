# 공통 인라인 흐름의 본문 글자 주소

## 선행 분석과 독립 계약

현재 45개 PR 범위와 행정업무편람 추가 개선 제외를 유지한다. #4961의 원 해결은
#5122, merge `0bc05ef81107ac61ec38d622f71b44a44d1b4821`이며 진단의 원문 소유를
추정으로 보충하지 않는 계약을 확인했다. 현재 실패 검사는 `de6516ffc0`의 실제
원문·렌더 런 대조와 연결된다. HWPX 본문은 `9460078238`에서 들어온 NO_LS 그림의
공통 inline flow를 사용하고, 같은 문서의 중첩 셀과 HWP 본문은 기존 composer를
사용한다. source trace에서 문자열을 맞춰 소유를 추측하는 방식으로 고치지 않는다.

공개 API 생성 원본의 일반 텍스트는 `가🦦나나끝`이고 raw scalar 1에 그림 두 개가
있다. 공개 모델의 논리 제어 주소는 각각 1과 2이며 바로 뒤 🦦는 논리 주소 3이다.
기존 HWP·중첩 셀의 런은 0/3/4/6이지만 HWPX 본문은 0/1/2/4로 잘못 발행한다.
보조평면 문자는 scalar 한 칸이다. 구조 제어와 부동 개체는 추가 칸을 차지하지 않는다.

## 생산과 소비 경로

`inline_flow::plan`의 Text.range는 원문 scalar 범위이며 측정·물리 줄의 계약이다.
`layout_inline_flow_plan`이 이를 그대로 TextRun.char_start로 저장하면서 논리
주소 소비자인 W2의 소유 판정과 커서가 다른 문자를 가리킨다. 같은 함수의 런
병합 검사도 원문과 논리 축을 섞는다. 두 소비자를 같은 문단 주소 맵에 연결한다.
물리 상자·줄 나눔·폭·높이·실제 그림 원점과 원문 range는 바꾸지 않는다.

주소 맵은 원문 scalar 앞에 놓인 실제 is_logical_inline 제어의 수를 한 번 계산한다.
같은 source 위치의 그림 두 개도 모두 센다. 삽입 캐럿은 개체 앞 경계일 수 있으나
여기서는 실제로 개체 뒤에 그리는 일반 글자의 시작을 구한다. 표시 폭이나 문자
모양으로 위치를 추측하거나 HWPX 버전에 따라 상수를 더하지 않는다.

## 검증과 남은 시각 결함

기존 #4961 검사 전체와 그림·표의 주소 대조군을 같은 동결 Native에서 실행한다.
기존 assertion의 source offset·내용 순서는 유지한다. 이전 Native117의 같은 검사
실패와 새 제품의 실제 결과를 연결하며 새 fixture·골든·배치 기대값은 추가하지 않는다.

원본 HWP/HWPX의 독립 한컴 2020 Print는 별도 job으로 확보했다. Native117은
47.68752%/50.20350%이며 중첩 셀의 잘림·표 높이가 다르다. 이 단계의 주소 보정만으로
시각 검증을 통과했다고 판정하지 않는다. 공개 API를 읽기 전용으로 호출하기 전후
모델·SVG·저장 바이트 보존과 원문 소유를 먼저 확인한 뒤, 남은 실제 배치 결함도
공통 경로에서 보정하고 Native/fresh 전쪽 90% 이상과 확대 판독을 완료한다.


## 주소 보정의 실제 검증

Native119는 시작 head `51a6e3414d`의 작업 트리에서 241.16초, exit 0으로 빌드했다.
공유 target·컴파일 잡 8개를 유지했고 동결 소스 해시는 빌드 전후 일치했다.
`char-overlap-native-build-119-result.json`에 전체 SHA·명령·실행 파일/rlib 해시가 있다.

| 실제 검사 모듈 | 결과 |
| --- | --- |
| issue_4961_font_decision_trace | 13 PASS |
| issue_4962_font_metric_coverage | 8 PASS |
| issue_2724_passthrough_invalidation_guard | 5 PASS |
| issue_7642_missing_lineseg_picture_flow | 4 PASS |
| text_font_coverage_registration | 3 PASS |
| nested_inline_table_source_ownership | 2 PASS |

35건 전부 실제 실행됐으며 누락은 0건이다. `current-previous-failures-1833-result.json`과
`logs/inline-flow-address-contracts-1833-driver.log`에 검사별 명령과 결과를 남겼다.
새로운 배치 검사나 fixture를 추가하지 않았고 기존 source assertion도 바꾸지 않았다.
실제 Native118의 같은 원문 소유 검사는 1832 계측에서 source unavailable로 실패하고
Native119는 동일 기대값으로 통과한다. 1817의 실제 Native117 실패도 보존한다.

HWP/HWPX 두 원본을 각각 Native118/119에서 Print SVG로 내보냈고 둘 다 바이트가
완전히 같다. `inline-flow-address-svg-parity-1834/result.json`에 해시를 기록했다.
이 결과는 새 PNG의 Native/fresh 일치율 판정과 구별한다. 기존 미달을 통과로 바꾸어
보고하지 않으며 fresh WASM·최종 전체 검증은 완료 전이다.
workspace fmt와 diff check도 통과했다.

## 남은 잘림의 생산 지점

1832에서 그림 속성 토글 직후와 중첩 셀로 복사하기 직전을 계측했다. 두 그림 토글
직후 모두 줄 높이/텍스트 높이가 400HU, 기준선 340HU다. 원문의 18pt 글자가 있어도
`migrate_picture_floating_to_inline`이 그림 높이로 첫 줄을 덮는다.
`two-tac-toggle-cache-probe-1832-result.json`과 실행 로그가 실제 생산 경로를 입증한다.

원 #1151의 독립 한컴 근거는 텍스트가 없는 그림 전용 문단의 토글이다. 이 원문과
`hancom_picture_tac_toggle.md`를 함께 대조해 빈 개체 문단의 계약을 텍스트 혼합
줄에 그대로 일반화했는지 다음 단계에서 보정한다. 저장 줄 수용 조건을 먼저
완화하거나 중첩 셀 clip만 키우는 방식으로 캐시 생산 결함을 숨기지 않는다.
