# #7069 CI audit의 비동기 증거 불일치 개선

- Issue: [#7069](https://github.com/edwardkim/rhwp/issues/7069)
- 기준: `3f34869b9c4d15a27b181dd22c63cd0a3730d46a`
- 실제 응답: [최소 보존 fixture](../../scripts/tests/fixtures/ci-impact-policy/issue7069-lint-snapshot.json)

성공 workflow 아래 known nonterminal job/step을 failure와 구분해 pending으로 처리했다.
세부 단계의 성공으로 job 성공을 합성하지 않는다. 완료 실패·취소·잘못된 skip·누락·중복에 대한
기존 검사를 유지했다. 실제 API 응답을 재생한 테스트와 수렴/비수렴 반례를 실행했다.

새 수집기는 동일 run/head/저장소/branch/workflow를 확인하고 Jobs API 전후 run snapshot을
대조한다. attempt 또는 상태가 바뀌면 자료를 버리고 재수집한다. job은 같은 run/head에 속해야
하며 현재 attempt보다 미래이면 거부한다. 부분 재실행의 이전 attempt 성공 job은 latest 응답에
남을 수 있으므로 허용하되 현재 attempt의 job이 적어도 하나 있어야 한다.

최대 4회, 재조회 간격 5초, 경과 예산 45초로 추가 재시도를 제한했다. 각 HTTP 요청 timeout은
10초이며 pagination/HTTP 요청 자체 시간과 controller의 10분 상한은 별개다. 예산 소진 시
pending을 유지하고 완료 이벤트가 더 없으면 해당 policy audit run을 재실행하도록 경고한다.
자동 재실행 루프나 추가 권한은 도입하지 않았다.

2026-09-13 로컬 Node classifier/policy/collector/post-merge 회귀 **333 PASS**, Python workflow
계약 **176 PASS**. Rust·renderer·HWP/PDF 입력 변경이 없으므로 Cargo/시각 검증은 비해당이다.

policy 모듈의 판정 변경은 live devel base에서 로드된다. collector workflow 배선은 기본 branch
main에 정상 release로 반영된 뒤 활성화된다. 이 PR의 devel 병합만으로 controller 배선까지
운영 중이라고 주장하지 않는다. 별도 post-merge 실행 제거는 #7070으로 추적한다.
