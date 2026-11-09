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
5. 동일 CodeQL 2.27.4의 전체 Rust snapshot과 기본 security suite로 전후 분석한다. 소스 집합·추출 상태·query·양성 대조를 확인하고 탐지 제외·넓은 sanitizer로 완료를 대신하지 않는다.
6. GitHub의 8건은 아직 OPEN을 유지한다. 로컬 검증을 완료해도 원격 분석·protected branch fixed 확인을 수행한 것으로 쓰지 않는다.

## 증적 보관 경계

원문 SARIF와 로그·snapshot은 기본 작업공간의 ignored `output/security/code-scanning-20261109/`에 둔다. 공개 문서에 실제 문서 본문·비밀 값·민감한 상세 재현을 넣지 않는다. 정식 결과는 검증 종료 후 이 문서와 정규화 증적에 기록한다.
