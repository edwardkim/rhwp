# #3587 Stage 14 — C 종료 통합 검증

- 승인: 메인테이너 「다음 절차 진행을 승인합니다」, 2026-09-13.
- 시작 기준: `011248b01`, `task_m100_3587`. Stage 13 기록은 이미 커밋되어 있다.
- 범위: 전체 nextest, Native Skia 3종, Rust lint, Docker WASM 실제 실행 및 C 채우기 1/10/100회 계측.
- 상태: 1차 전체 검사 후 계약 누락 보완·재검증 준비. D/Gym·push·PR·댓글은 이번 범위가 아니다.

기존 C1/C2 실물 성공 판정과 B 계측은 재사용하되 현재 C 코드의 전체 검증을 대체하지 않는다.
검증 source를 고정하고 고정 review target의 Cargo 명령은 순차 실행한다.

준비 중 신규 CLI 테스트의 실행 파일 선택을 정정했다. nextest archive가 전달하는 런타임
`CARGO_BIN_EXE_rhwp`를 먼저 읽고 컴파일 시점 경로는 fallback으로만 사용한다.
제품 동작과 기대값은 바꾸지 않는다. 이 테스트 보정도 새 검증 SHA에 포함한다.

## 1차 전체 검사와 보완

`8351b73f7`에서 nextest 전체 **9,620 PASS / 8 FAIL / 47 skipped**였다.
실행 361.226초, 준비·컴파일 포함 612.778초. `output/3587/c-integrated/nextest.log`를 보존한다.
기존 조판/IR/overflow/off-canvas/text-overlap 검사는 통과했으나 전체 통과는 아니다.

| 실패 | 원인과 보완 |
| --- | --- |
| 스키마 object 정책 1건 | typed 템플릿 주소/상한의 엄격한 키 정책과 문자열 record map이 기존 정책 목록에 빠졌다. 닫힌 사유를 명시하고 record 값 문자열 제약을 검사한다. 제품을 느슨하게 만들지 않는다. |
| 계획 생성기 1건 | 기존 4-action 시퀀스만 생성했다. 새 3-action은 별도의 단독 step 생성기로 JSON·typed 요청·스키마 대조를 추가한다. 기존 시퀀스 검사는 유지한다. |
| 출처 fixture 대조 2건 | C3 MAP에 추가한 preview·operationResult·검증 이유가 소비자 fixture에서 누락됐다. 실제 MAP와 origin을 동기화한다. |
| 출처 실물 대조 1건 | 레시피에 run dry-run 호출이 없어 preview 필드를 관측하지 못했다. 면제 대신 실제 dry-run 레시피를 추가한다. |
| 원본 바이트 무효화 가드 1건 | 준비/공통 commit으로 옮긴 네 래퍼가 미분류였다. 실제 raw_stream 무효화 호출 경로를 확인하고 DelegatesTo 사유를 추가한다. 무효화 자체를 생략하지 않는다. |
| 스키마 중앙화 가드 1건 | 새 WASM JSON 경계에 봉투 버전 문자열을 직접 썼다. 동일 값의 ENVELOPE_SCHEMA_VERSION을 참조한다. |
| 지식지도 필드 사전 1건 | steps[].operationResult·steps[].workload 설명 누락. 두 필드의 출처·입력량/실측 구분을 추가한다. |

첫 실행의 실패 로그와 보완 후 결과는 분리한다. 위 항목은 C3 연동에서 함께 갱신해야 했던 누락이며,
내가 집중 검사만으로 발견하지 못했다. baseline 완화나 제외로 통과시키지 않는다.
