---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4851 검토 - 프롬프트 주입 방패 armor

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4851](https://github.com/edwardkim/rhwp/pull/4851) · @kevin9327 |
| 원 head | `7cbc51c8a19b2662542c38cd553029cd0b88af2f` |
| 누적 순서·적용 SHA | 15/27 · `b24cea3b7` → `3acd23363` → `45eefc36b` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · agent profile·field dictionary 보정 |

nonce 격벽, 주입 신호·출처 표지를 제공하는 armor를 추가한다. 메인터너 보정으로 profile 등록과 테스트
nonce 생성을 실제 생성기로 바꾸고, 지식지도에 필드를 등재했다. 전체 CI가 통과했다. **수용 가능**으로 판정한다.
