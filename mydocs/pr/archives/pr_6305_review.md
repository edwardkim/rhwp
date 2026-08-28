---
kind: review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6305 검토 - 한양견고딕 alias 잠금

- PR: [#6305](https://github.com/edwardkim/rhwp/pull/6305)
- 작성자: `@planet6897`
- 원 source head: `8f21e36b3992d5ba0ec9e538574446d954f1b5e5`
- 누적 검토 적용: `b489f8efc` (`git cherry-pick -x`)

## 변경 검토

`tests/cases/issue_6171_hanyang_gyeongothic_alias.rs`에 한양견고딕 alias와 family chain의
우선순위를 잠그는 회귀를 추가한다. 제품 코드나 fixture의 의미를 바꾸지 않는 테스트 전용 PR이다.

## 검증 상태

- 원 source head의 GitHub required CI는 성공했다.
- 누적 체리픽은 충돌 없이 적용됐다.
- 로컬 focused 회귀는 이번 누적 정적 검토에서 다시 실행하지 않았다. 이미 green인 정확한 source head의
  중복 전체 회귀는 실행하지 않는 경로를 적용한다.

## 최종 판정 - 수용 가능

`#6305`는 수용 가능하다. 같은 누적 후보의 보류 사유가 해소되고 통합 head CI가 성공하면 함께
merge 대상으로 확정한다.
