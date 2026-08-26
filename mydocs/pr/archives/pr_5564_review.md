# PR #5564 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5564>
- 작성자: `planet6897`
- 관련 이슈: #5558
- base / 원 head: `devel` / `b91a458b06ecf404f335c3fc8eda6881b595f909`
- 누적 적용: `d99dd75a9` (`review/planet6897-20260819`)
- 선행 의존: #5552
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

원 head에는 #5552의 선행 commit이 포함되어 있으므로 누적 branch에는 선행분을 중복 적용하지 않고,
글상자 정보의 내부 문단 리스트를 회수하는 follow-up만 적용했다. common header의 잉여 구간이
표 78 구조와 정합할 때만 text box 문단으로 파싱하고, 정합하지 않으면 기존 skip 경로를 유지한다.
`issue_5558_hwp3_textbox_recovery`가 이 보수적 조건을 검사한다.

## 판정

#5552 뒤에 적용한다는 전제를 만족하므로 누적 수용을 권고한다. 도형 자식과 text box 복구는
시각 영향이 있으므로 최신 CI와 관련 fixture 검증을 최종 조건으로 둔다.
