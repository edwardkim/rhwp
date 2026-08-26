---
kind: pr-review
status: completed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4462 검토 기록

## 결론

- 메인터너 보정 `2d2f42524`를 포함하면 수용 가능하다. HWPX caption의 vertical alignment를
  parser 모델로 보존한다.
- 최신 contributor head `9a8063d20`는 `devel` 대상, non-draft, `MERGEABLE`, required check 성공이었다.

## 누적 교차 보정

- `06a987f31`을 `-x` 체리픽했다.
- 이 PR은 공용 parser 이름을 `parse_caption`으로 정리했다. 뒤의 #4503이 새 `shape_children`
  분기에서 이전 이름을 호출해 누적 컴파일이 실패했다.
- 메인터너 보정은 #4503 호출 한 곳만 새 공용 이름으로 바꿨다. caption 데이터 모델·수직 정렬 의미·오류
  처리는 변경하지 않았다.

## 검증

- caption vertical alignment와 HWPX field parameters focused test 6건이 통과했다.
- 누적 `release-test` 전체 5,645건도 통과했다.
