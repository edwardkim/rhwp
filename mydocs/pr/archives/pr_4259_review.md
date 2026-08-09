---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4259 검토 - 텍스트 표 호스트 문단 캐럿 질의 페이지 좁히기

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4259](https://github.com/edwardkim/rhwp/pull/4259) / @humdrum00001010 |
| contributor 원 head | `2b19680d8c27ad10ea660b52a00c816ab6948dbc` |
| base / 규모 | `devel`, 11개 파일, +671/-2 |
| 관련 이슈 | [#4179](https://github.com/edwardkim/rhwp/issues/4179), [#4145](https://github.com/edwardkim/rhwp/issues/4145) |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

텍스트가 있는 표 호스트 문단의 캐럿 질의에서, pagination 메타데이터만으로 연속 중간 페이지를 후보에서
제외한다. 해석 불가능한 경우에는 기존 넓은 탐색 경로를 보존한다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- `issue_4179_cursor_rect_text_host_para_pages`와 전체 `release-test --tests`가 통과했다.
- #4248 fast path와 조합한 cursor query 경로를 통합 후보에서 검증했다.

**통합 수용 권고.** 통합 PR의 closing keyword에는 #4179와 #4145를 모두 포함한다.
