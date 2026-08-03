---
kind: review
status: rework-required
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-04
---

# PR #3887 검토 - preflight 검사 범위 확대

- 원 head: `c5cfba6cda5754e477ddda55367f2b7e31b50f9b`
- 범위: `tools/agent_preflight.py`의 실패 경로·선언 밖 명령 검사 확대.
- 시각 검증: 불필요. 개발 도구만 변경한다.

## 차단 사항

`python3 tools/agent_preflight.py --bin target/review-kevin9327-20260804/release-test/rhwp`
가 실패했다. 새 `check_undeclared_commands_are_not_invisible`가 아래 명령이
`--rhwp-preflight-bogus-flag`를 exit 0으로 무시한다고 보고한다.

`gen-pua`, `gen-table`, `measure-width`, `test-caption`, `test-field`, `test-shape`

검사는 `tools/agent_preflight.py:622`~`665`에서 새로 추가됐고, 이 명령들은
`src/main.rs:326`~`335`, `2280`, `2311`~`2315`에서 dispatch·capabilities에 존재한다.
즉 새 가드가 실제 결함을 발견했으나, PR 자체는 green이 아니다.

## 요청 사항

각 명령이 미지 옵션을 exit 2와 빈 stdout으로 거부하게 하거나, 자기서술 밖 명령의
명시적 부류·제외 사유·회귀 테스트를 정의해야 한다. 보정 뒤 실제 release binary에
preflight를 실행해 통과시키는 로컬 검증이 필요하다.
