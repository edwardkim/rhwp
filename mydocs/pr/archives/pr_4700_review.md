---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4700 검토 - 한국어 주입 탐지 절 경계 오탐 축소

| 항목 | 기록 |
| --- | --- |
| PR | [#4700](https://github.com/edwardkim/rhwp/pull/4700) |
| 작성자 / 원 head | @planet6897 / `2e198cd3ebd12c689ce425ebdcb66dc84755dd85` |
| 적용 commit | `0b2ca0b71` |
| 통합 후보 | `c7cfaefb9` |

한국어 문장에서 목적어를 목적격 위치에서 찾도록 절 범위를 제한해, 무해한 문장 뒤의 지시문 패턴을
주입으로 오인하지 않게 한다.

## 메인터너 보정

통합 중 `14f1f982f`로 절 범위가 인라인 그림 전용 줄을 일반 텍스트처럼 건너뛰지 않도록 보완하고,
제어문 line-seg 대응을 회귀 테스트로 고정했다. 이는 탐지 범위를 넓힌 변경이 아니라 그림 줄 소유와
문단 경계를 명확히 해 오탐·누락 양쪽을 방지하는 호환 보정이다.

## 완료한 검증

- 비공개 한국어 문서의 injection scan은 findings 0건이었다.
- 절 경계 focused 회귀와 누적 후보 전체 `nextest` 5,923건이 모두 통과했다.

**통합 수용 대상이다.**
