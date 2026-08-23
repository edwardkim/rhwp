---
kind: pr-review
status: accepted-pending-integration-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5927 검토 - 쪽 하단 중첩 표 이송 여백 (#5920)

## 접수와 범위

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5927](https://github.com/edwardkim/rhwp/pull/5927) / [@kevin9327](https://github.com/kevin9327) |
| base / source head | `devel` / `6d5ad6a7fd77644d06d07ff3235d5aa2568a70c9` |
| 규모 | 4 files, +284 / -0, 1 commit |
| 접수 상태 | non-draft, reviewer `@jangster77` 지정, 작성 시점 `MERGEABLE/CLEAN` |

쪽 하단의 중첩 표가 보이지 않는 이송 여백 때문에 다음 쪽으로 밀리는 renderer 결함을 고친다. source
commit 1/1이 통합 후보에 적용됐다.

## 검증과 증적

- source head의 check는 21 success, 1 neutral, 3 skipped, failure 0이다.
- 통합 code candidate 전체 nextest 8,201 passed와 현 head merge-tree·공백·fmt·unit-tier 검사를 통과했다.
- [8쪽 전·후·한글 2020 대조](../../report/edit_demo_5920/page8_before_after.png)에서 수정 후 결론 상자가
  기준과 같은 페이지에 배치되고 수정 전의 하단 공백이 해소됨을 확인했다.

## 판정

**수용 권고.** renderer 회귀시험과 기준 대조가 같은 layout 개선을 보여 주며 차단 결함은 발견하지 못했다.
통합 PR의 최신 CI 성공과 작업지시자 승인 후 merge한다.
