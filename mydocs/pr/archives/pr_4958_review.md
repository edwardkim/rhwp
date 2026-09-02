---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4958 검토 - HWPX Scripts 패키지 원문 보존

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4958](https://github.com/edwardkim/rhwp/pull/4958) |
| 작성자 / source | @planet6897 / `fix/3557-scripts-preserve` |
| 원 source head | `f04dc03b8a5bca6934eaa511ab08e80748d8e284` |
| 기준 devel | `418e5b191d23cf0618ce99f0cfec332c19ac1bc2` |
| 통합 branch / local 적용 | `review/non-draft-20260816` / `84ad0e5b5` |
| 관련 issue | #3557 |
| 작성 시점 원 PR 상태 | `OPEN` / `MERGEABLE` / `CLEAN`; merge 전 재확인 필요 |

IR로 모델링되지 않는 `Scripts/` ZIP 엔트리의 원문 바이트와 content.hpf manifest·spine 참조를 parser와
serializer 사이에 보존한다. OLE와 embedded font의 기존 보존 회귀도 함께 고정한다.

## 검증과 판단

`issue_3557_package_preservation`을 포함한 최종 release-test nextest 전체가 6,519건 통과했고, fmt, clippy,
diff 검사도 통과했다. parser·serializer 패키지 보존 변경이며 렌더 출력 계약을 바꾸지 않으므로 별도 시각
sweep은 적용하지 않았다. **통합 수용 권고.**
