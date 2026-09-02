---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4809 검토 - Gym 판별력 감사

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4809](https://github.com/edwardkim/rhwp/pull/4809) · @kevin9327 |
| 원 head | `e8df000eb70e62c2aff02a4dbf68f541c5143e60` |
| 기준선 | `upstream/devel@4cf8a5898` |
| 누적 적용 | `e8df000eb` → `2f217e20e` |
| 메인터너 보정 | `f674ac7c5` |
| 원 CI | 작성 시점 참고값: CI·CodeQL 성공, mergeable `MERGEABLE` |
| 자동 검토 | [P1 #4809 경고](https://github.com/edwardkim/rhwp/pull/4809#discussion_r3789057609) 재현 후 보정 |

## 변경과 보정 이유

원 PR은 입력 무편집 복사와 틀린 answer를 음성 대조로 사용해 약한 오라클을 찾는다. 그러나 artifact
과제는 입력과 다른 임의 바이트가 `file_exists`와 `differs_from_input`만 통과하면 성공할 수 있었다.
경고와 같이 LR03, LR07, SD07, SR05, TB02, TB07에 1KiB garbage를 제출해 실제 false-pass를 재현했다.

보정은 artifact마다 입력 복사와 garbage 두 제어를 모두 실행하고, 제어별 false-pass도 보고한다.
해당 여섯 과제에는 SVG root, JSON dialect, CSV 첫 셀, UTF-8 BOM처럼 산출 형식과 핵심값을 확인하는
공통 checker를 추가했다. 따라서 단순히 파일이 존재하거나 입력과 다르다는 이유만으로 통과하지 않는다.

## 완료 검증

- `python3 -m unittest ... test_gym_discriminate.py ... test_workflow_contract_wiring.py`: 총 89건 통과, 의도된 1건 skip.
- `python3 gym/tools/discriminate.py --bin target/pr-review/release-test/rhwp --json`: 106개 과제, 156개 제어 모두 거부, false-pass 0건.
- `git diff --check`: 통과.

**자동 경고를 보정한 뒤 수용 후보.** merge 직전에는 원 PR 최신 head와 required check를 다시 확인한다.
