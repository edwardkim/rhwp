# PR #5552 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5552>
- 작성자: `planet6897`
- 관련 이슈: #5141
- base / 원 head: `devel` / `a90e85003a31bd36814a966f6487d2d19b6c71ae`
- 누적 적용: `f4b18606c` (`review/planet6897-20260819`)
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

HWP3 묶음 개체 parser가 컨테이너 세부 길이 8바이트와 선언된 common header 길이를 모두 소비하도록
수정한다. 이를 통해 자식 drawing stream의 8바이트 시프트와 childless container 오판을 막는다.
`issue_5141_hwp3_group_children` 통합 회귀가 자식 도형 복구 계약을 고정한다.

## 판정

누적 수용을 권고한다. 후속 #5564가 이 parser 전진 뒤의 글상자 본문 회수를 확장하므로, 두 PR은
누적 순서를 유지해야 한다. 공통 검증 및 최신 CI 통과가 최종 조건이다.
