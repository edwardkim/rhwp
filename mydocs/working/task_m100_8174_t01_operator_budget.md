# #8174 T01 — operator 버퍼 선행 제한과 standard 800만

- 승인: 2026-10-25 권고 수용과 「순서대로 진행하세요.」.
- 시작 source: `cf5d33d8ec`. [잔여 작업](task_m100_8174_pr_remaining_todo.md)의 T01이다.
- 상태: 수정 전 정식 자원 계약 실행과 구현 진행 중. 완료·PR 제출 판정이 아니다.
- 증적: ignored `output/pr-review/issue8174/v2-s08-20261025/operator-assessment/`.

## 선행 분석과 적용 경계

bounded content 수집은 #7913의 `4662c069e5`, 제품 400만은 #7970의 `515bd72dcd`에서
이어졌다. `parse_content_with_budget`는 callback으로 실제 명령을 Vec에 모으고,
`PdfDocument::parse_page_import_content`가 페이지 캐시와 interpreter에 전달한다.
`operator_copy::slots`는 소비자 복사 전에 단일 stream 32 MiB와 decoded 예산을 검사하지만
생산자의 최초 Vec 수집은 끝난 뒤였다. [상한 조사](task_m100_8174_operator_ceiling_review.md)에서
집중 입력의 Native 329 MiB·WASM 선형 메모리 457 MiB까지 커진 뒤 거절하는 현상을 확인했다.

공통 수집기에서 `stream_bytes / size_of::<Operator>()`로 저장 가능한 슬롯 수를 정하고,
capacity 증가 목표를 그 슬롯 수 이하로 제한해 fallible exact reserve한다. 새 예약은
실제 할당 전에 같은 요청 decoded 예산에 계상한다. 이 카운터는 누적 예약 작업이며 allocator의
부가 비용이나 live heap 전체의 상한이 아니다. 점진적인 재예약도 누적 작업에 포함한다.
이 생산 결과를 Native/WASM의 기존 cache·interpreter 경로가 그대로 소비한다.

standard operator만 8,000,000으로 바꾼다. constrained 250,000·선택 쪽당 250,000과
낮춘 명시 cap·참조/XObject/그림/문자 등 다른 한도는 유지한다. actual counter를 줄이거나
실패 상태를 초기화하지 않는다. 토큰 준비 사실은 기록했으며 비밀 값이나 인증 파일은 열지 않았다.

## 검증 계약

정식 `issue_8174_pdf_operator_limits`는 저장/조판 golden이 아닌 합성 자원 계약이다.
Native와 wasm32의 enum 크기 차이를 실제 `size_of`로 반영하며 임의 화면 좌표를 고정하지 않는다.

1. 슬롯 하나보다 작은 한도, 비정수 슬롯 바이트 cap, 기하급수 증가의 끝을 실행해 실제 반환 Vec
   capacity가 단일 버퍼 한도 이내인지 확인한다. 초과 뒤 sticky 상태와 새 요청을 대조한다.
2. 첫 슬롯의 decoded 정확/직전 한도에서 예약 전 거절과 올바른 실패 진단을 확인한다.
3. 40쪽 800만 정확 경계·+1, constrained·낮은 host cap과 선택 1쪽의 유효값을 실제 frontend에서
   실행한다. 압축 입력의 명령 개수는 생성식으로 독립 확정한다.
4. 집중 입력의 거절 전 메모리와 정상 800만을 실제 Native/최적화 제품 WASM·Studio에서
   재검증한다. 새 제품 source로 실행하기 전에는 기존 소비자 결과로 대체하지 않는다.

실행 source·결과와 다음 작업은 이어 기록한다.

수정 전 제품 source에서 새 정식 계약 3개를 실제 실행해 **3 FAIL**을 확인했다. 첫 검사는
한 슬롯 한도에서 Vec의 과예약, 둘째는 최초 공간의 무계상, 셋째는 400만 소진을 검출했다.
원문은 `t01-before-tests.log`다. 같은 계약을 보정 후 다시 실행한다.

보정 후 같은 정식 계약 **3 PASS**를 확인했다(`t01-after-tests.log`). 실제 공통 frontend의
800만 정확 경계·+1 거절, constrained/400만 host cap과 선택 1쪽의 250,000 배정을 실행했다.
기존 resource/frontend/CLI 정책 계약과 제품 실행·메모리 검증은 이어 진행한다.

기존 `issue_7911_pdf_resource_limits`·`issue_7911_pdf_frontend`·
`issue_8174_pdf_resource_policy`도 Native Skia `release-test`로 실제 실행해 **155 PASS**를
확인했다(`t01-existing-contracts.log`). 정상 여정·요청 격리·낮은 host cap·자식 감독과
원자적 출력 보존이 유지됐다. 정식 자원 검사는 합계 158 PASS다.

현재 제품 Native source의 standard 기본값은 800만이다. 최적화 제품 WASM·Studio와 집중
입력의 메모리 재측정은 아직 완료하지 않았다. 현재 코드 변경을 커밋하고 T02 저장 오류의
원인을 이어 추적하며, 남은 구현을 반영한 제품 source에서 T01의 실제 환경 검증과 T07–T08
통합 게이트를 모두 완료한다. 이번 focused 결과를 전체 회귀·최종 제품 검증으로 확대하지 않는다.

## Native 실제 제품과 예약 비용

고정 Native 제품 source `fd4386a5cc`에서 40쪽 800만 정확 경계의 실제 CLI 가져오기·HWPX
저장에 성공했다. +1·집중 입력·낮춘 400만 host cap·constrained 거절은 기존 출력 sentinel을
유지했고 이후 정상 2쪽 요청도 성공했다. `t01-native-contracts/results.json`에 기록했다.
이 동작 검사 중에는 소비자 빌드가 겹쳤으므로 시간·RSS 비교 표본으로 사용하지 않는다.

별도 진단 소비자는 원래 상한 조사와 같은 `pdf-import` 전용·opt-level 1·codegen-units 64로
빌드했다. 빌드가 끝난 뒤 같은 생성 입력·감독·3표본을 재실행했다. 정상 800만의 중앙값은
1,231.86 → 1,220.92 ms, 관측 RSS 최대는 55.70 → 51.05 MiB였다. 유한한 관측이며
일반적인 성능 개선·무영향을 주장하지 않는다. 23회 경계/집중 요청과 각각의 정상 후속 요청을
통과했다. 해시·원문은 `operator-assessment/t01-consumer/`에 별도로 고정했다.

집중 입력은 기존처럼 약 800만을 모으지 않고 838,861번째 예약에서 `content operator buffer`로
거절했다. 관측 RSS는 **329.43 → 57.16 MiB**, 시간은 519.74 → 80.75 ms였다.
새 예약 카운터는 재할당 작업까지 계상하며 실제 RSS와 구별한다. 최적화 제품 WASM·Studio의
용량·종료·복구와 최종 코드의 전체 검증은 계속 남아 있다.

## 최적화 제품 WASM·Studio — source `9cabbff39d`

동일 source의 최적화 제품 WASM에서 40쪽 800만 정확 경계의 가져오기·저장과 +1·집중 입력의
거절을 실제 Chrome으로 실행했다. 한 요청에서 실제 사용한 linear memory capacity는
85,262,336 bytes였지만, 앞선 정상 문서의 할당을 재사용한 단회 관찰이므로 peak RSS나 통제한
전후 성능으로 발표하지 않는다. 집중 입력의 거절 이유는 `content operator buffer`다.

실제 Studio의 PDF 가져오기 창으로 +1, 명시 400만 cap, 집중 입력을 거절하고 800만 요청을
취소했다. 각 실패·취소 후 활성 문서 generation·페이지 수·파일 handle·draft identity·저장된
draft 목록·최근 문서 목록이 이전과 같았다. 이어 정상 15쪽을 열어 HWP·HWPX 저장에 성공했다.
실제 로드한 WASM SHA-256은 `eabd668c471a2d32c96f4a8ae72956030c500901c15f3f32ecd9031d8b57a680`으로
최적화 producer와 일치한다. Worker도 실제 비사각 clip 거절 후 이전 5쪽을 유지하고 새 15쪽을
저장했다. T07–T08의 최종 변경 source에서 통합 검증·비용을 다시 확정한다.

증거는 [S08 공개 요약](../pr/assets/issue8174/v2-s08/README.md)과 ignored
`t02-operators-candidate.json`·`t02-worker-clip.json`·`t02-studio.json`에 있다.
