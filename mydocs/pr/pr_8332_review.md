# PR #8332 검토 — perf(equation): 같은 수식 값의 흐름 메트릭을 다시 계산하지 않는다

## 최종 판정

**머지 보류 — 검증 진행 중.** 동일 입력의 최신 base/누적 후보 실행과 필수 검증을 완료한 뒤 판정을 갱신한다. 이 잠정 상태를 기여자 재작업 결론이나 새 회귀 검출로 사용하지 않는다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8332, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `c773e5060c0f186b4bf8281133004f5afe120fa4`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:49.714995+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 원 contributor의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례·필수 로컬 gate는 현재 검토 중이며 아직 완료로 판정하지 않는다.

## 선행 해결과 중복 보정

원 PR의 핵심 메트릭 캐시는 이미 [#8348](https://github.com/edwardkim/rhwp/pull/8348)의
`15ff366244`에서 flow/paint 공통 `equation/cache.rs`로 구현됐고 `c28d998091`에 편집 뒤 새 스레드 대조가 있다.
현재 base는 값 키·source frame seal·16,384개 flow/4,096개 paint의 유한 캐시를 유지한다.
원 head를 그대로 누적하면 기존 flow 캐시 안에 다른 flow 캐시가 생긴다. 메인터너는 기존 공통 캐시·source frame
판정을 유지하고 중첩 캐시와 불필요한 pass 초기화를 제거했다. 기여의 키워드 탐색 할당 제거·값 조건 뒤 토큰화와
공개 편집 계약 검사는 보존한다. 원 캐시 주장의 과거 15초→2.7초 수치를 최신 base의 개선으로 재인용하지 않는다.
현재 base/후보의 실제 성능과 출력 보존은 후속 측정에서 판정한다.
