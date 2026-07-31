---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3620 리뷰 — k번째 치환·체크박스 세밀 수정

- 원 PR / 작성자: [#3620](https://github.com/edwardkim/rhwp/pull/3620) / `@kevin9327`
- 관련 이슈: [#3395](https://github.com/edwardkim/rhwp/issues/3395)
- 원 head / 통합 반영: `80cfdbb1e` / `79ed25417`, `99b3ab05f`

`--occurrence`로 지정한 한 번의 치환과 `hwp_set_checkbox`를 추가하고, #2724 invalidation guard의
delegation 예외를 계약에 등재한다. broad replace로 사용자의 실제 서식을 훼손하지 않는 조작 단위를 만든
점이 핵심이다. `replace_occurrence_contract`, guard 회귀와 전체 release-test exit 0을 확인했다. 전후
PNG는 해당 operation의 설명 증적이며 renderer 변경 검증은 아니다. **통합 PR full CI 조건부 수용**이다.

## 범위 확정

[#3647](https://github.com/edwardkim/rhwp/pull/3647)은
[`a54ff52a`](https://github.com/edwardkim/rhwp/commit/a54ff52aa8bb2ede5dfe06bb493090d74f9065ab)로 merge됐다.
이번 도구의 완료 범위는 지정 occurrence의 문자 `□`(U+25A1)를 `☑`로 바꾸는 경로다. 관련
[#3395](https://github.com/edwardkim/rhwp/issues/3395)가 최초로 지적한 문단 글머리표 `☐`(U+2610)는
현재 도구가 찾지 않으므로 해결됐다고 표현하지 않는다. 그 실물 양식 경로와 fixture·재독·렌더 회귀는
후속 범위로 issue를 open 유지한다.
