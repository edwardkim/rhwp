# CodeQL 잔여 경고 정리 — 2026-11-09

## 승인 범위와 기준

- 2026-11-09 작업지시자가 기존 #8745 완료 상태를 유지하면서 114건의 개별 분류와 나머지 8건의 로컬 보정·검증을 승인했다. PR·push·merge·새 원격 분석 실행은 포함하지 않는다.
- 기준 devel: `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`. 별도 follow-up branch이며 완료된 #8745를 재오픈하지 않는다.
- 최초/최신 full scan은 analysis `2120100961`/`2124551567`, CodeQL 2.27.4다. 정규화된 결과 144개와 flow topology는 같다. 경고 대상 파일 20개는 main/devel Git blob이 같다.

## 원격 분류 완료

공개 테스트 벡터 67건은 `used in tests`, 빈 암호·RNG 작업 배열·PDF zero IV 11건과 PyO3 진단 35건은 `false positive`, legacy RC4 #615는 호환 한정 `won't fix`로 분류했다. 각 항목의 rule·경로·열 위치·source SHA를 다시 확인하고 PATCH 뒤 GET으로 분류와 근거를 검증했다.

결과는 OPEN 8건, dismissed 183건, fixed 10건이다. 이전 69건의 dismissed 분류와 fixed 10건은 바꾸지 않았다. 남은 OPEN은 #597·#616–#622다. 이는 실제 취약점 114개를 코드 수정한 결과가 아니라 메인테이너가 승인한 개별 운영 분류다.

## 로컬 보정·검증 계획

1. PDF backend의 decoded content와 인접 direct /Contents Debug 로그에서 payload를 없앤다. 쪽 번호·길이·종류 진단과 기존 오류·자원 한도·출력 계약은 보존한다.
2. qcms의 C bindings 테스트에서 nullable transform을 checked reference로 변환한 뒤 사용한다. 기존 assertion만으로 분석이 해소되지 않은 지점도 같은 경계에서 보강한다. 생산용 변환 연산은 바꾸지 않는다.
3. 공개 합성 PDF의 실제 로그 캡처와 qcms null 반환 대조를 실행한다. 수정 전 FAIL/수정 후 PASS와 정상 암호·오암호·재시도·저장 계약을 확인한다. 테스트는 기존 수집 경로에 등록한다.
4. 관련 집중 회귀·전체 release-test·필수 Rust lint를 완료한다. 조판·paint 계산을 바꾸지 않으므로 시각 좌표·쪽수 baseline은 수정하지 않는다.
5. 동일 CodeQL 2.27.4의 전체 Rust snapshot에서 기본 suite의 경고 규칙 17개를 모두 전후 분석한다. 비경고 진단·통계는 별도로 확인한다. 소스 집합·추출 상태·query·양성 대조를 확인하고 탐지 제외·넓은 sanitizer로 완료를 대신하지 않는다.
6. GitHub의 8건은 아직 OPEN을 유지한다. 로컬 검증을 완료해도 원격 분석·protected branch fixed 확인을 수행한 것으로 쓰지 않는다.

## 증적 보관 경계

검증 중 원문 SARIF와 로그·snapshot은 기본 작업공간의 ignored `output/security/code-scanning-20261109/`를 사용했다. 정식 결과는 이 문서와 정규화 증적에 반영했으며, 종료 후 원문 임시 산출물은 삭제했다. 실제 문서 본문·비밀 값·민감한 상세 재현은 공개 증적에 넣지 않는다.

## 구현과 런타임 검증

로컬 보정 commit은 `612b4053db0a09cd714aa85e9fefa045f7c4d193`이다. PDF 본문 로그 두 곳과
qcms 테스트 포인터 일곱 곳을 보정했고, 실제 선택 기능 실행에 필요했던 기존 C LUT adapter의
반환형 불일치도 연결했다. 기본 feature·생산용 색 변환·인증 정책·조판 계산은 바꾸지 않았다.

전체 release-test 13,015건, PDF 로그 비노출 2건, qcms `c_bindings` 30건,
기존 #8745 경계 6건과 PDF frontend 17건이 통과했다. 정식 Rust lint·workspace build·
suite 정책 검사도 통과했다. 전체 실행의 기존 skipped 45건은 그대로 기록하며 새 검사는
정식 suite에서 실행됐다. 실제 수정 전 실패/수정 후 성공과 추가 vendor lint의 기존 차이는
[검증 원장](../tech/investigations/code-scanning-20261109/README.md)에 구분했다.

CodeQL의 전체 소스 경고 평가도 완료했다. 동일한 17개 경고 규칙에서 SARIF는 144→137건,
포인터 경고는 7→0건이다. 대상 이외 136건의 rule·주요 위치·메시지 의미는 유지됐고,
분석 snapshot과 작업 소스의 hash 차이는 없다. #597의 4개 flow는 본문이 아닌
`page_index: usize` 수치로 이어져 추가 오탐 분류의 근거로 남겼다.

현재 승인 범위인 114건 분류와 8건의 로컬 보정·검증을 완료했다. 23:15 KST 재조회에서도
GitHub의 8건은 main·devel에 OPEN이다. 원격 코드 반영·새 분석과 #597의 추가 분류는
수행하지 않았으며, 보호 브랜치의 실제 해결 확인을 후속 조건으로 남긴다.
작업 branch `task/8745-code-scanning-followup`과 worktree는 미제출 보정 소스를 보존하기 위해
유지한다. 원문 로그 대신 위 검증 원장의 정규화 증적과 source commit으로 결과를 확인한다.
종료 정리로 저장소 루트 `output/` 전체와 `target/pr-review/debug`, 소유가 확인된 임시
snapshot 두 곳을 삭제해 약 28.16 GiB를 확보했다. [정리 원장](../tech/investigations/code-scanning-20261109/cleanup.json)에 실제 사용 종료·삭제 후 부재·유지 범위를 기록했다.
