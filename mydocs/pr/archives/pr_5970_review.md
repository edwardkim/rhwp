---
kind: pr-review
status: held-excluded
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5970 review - HWPX 저장 lineseg 축 보정 (#5961)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#5970](https://github.com/edwardkim/rhwp/pull/5970) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| head | `b4b2bedccfa0342e49ebbe60f0dc82ffd801f608` |
| GitHub 상태 | non-draft, `MERGEABLE`, `BLOCKED` |
| 판정 | **보류 / 이번 통합에서 제외** |

## 검토 결과

이 PR은 처음 통합 후보에 포함했으나, GitHub CI의 `test-archive-b-shard-1 / Default-feature tests
(Archive B)`가 실패했고 aggregate `Build & Test`도 실패했다.

로컬에서도 #5970의 새 #5961 계열 commit을 포함한 후보에서 기존 회귀
`issue_1880_convert_hwp_roundtrip_render_is_self_consistent`가 실패했다. 비교 확인 결과 최신
`upstream/devel`에서는 해당 테스트가 통과했지만, #5970의 첫 새 code commit 적용 직후부터 같은 실패가
재현됐다.

따라서 작업지시자의 최신 지시에 따라 #5970은 이번 planet6897 통합 후보에서 제외했다. #5943 관련 앞선
중복 commit은 이미 `upstream/devel`에 포함된 범위라 별도로 다시 적용하지 않았다.

## 권고

현재 head 기준으로는 수용하지 않는다. #5970은 원 PR에서 CI 실패와 `issue_1880` 회귀 원인을 보정한 새
head가 올라온 뒤 별도 검토한다.
