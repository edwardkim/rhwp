---
kind: pr_review
status: accepted-with-maintainer-correction-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4106 검토 - preflight 큐 규율 검사

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4106](https://github.com/edwardkim/rhwp/pull/4106) / @kevin9327 |
| 원 head | `b7dd83c6c275d7217df0b730372ae69130fe0312` |
| 규모 | 3개 파일, +247/-1 |
| 원격 참고 상태 | `MERGEABLE` / `CLEAN`, 원 head CI·CodeQL 성공 |
| 시각 검증 | 비대상. Python preflight와 운영 문서만 바꾼다. |

열린 PR 잔량, 선언된 동일 이슈 중복, `task/<issue>-`·`wip/fix-<issue>-` 브랜치의 미잠금 착수를
경고 전용으로 검사한다. `--no-network`와 네트워크 실패는 조용히 건너뛰고 종료 코드는 유지한다.

## 발견 사항과 보정

원 구현은 코멘트 본문에 `착수`라는 단어만 포함되면 잠금으로 판정했다. 따라서 "아직 착수하지
않습니다"나 과거 인용문도 미할당 착수 경고를 억제할 수 있었다.

통합 보정은 protocol이 정한 실제 잠금 형식인 `착수합니다 — <범위>`만 인식하도록 정규식을 좁혔다.
단어 인용은 잠금으로 취급하지 않는 회귀 1건과 올바른 형식은 잠금으로 인정하는 회귀 1건을 추가했다.
CI 워크플로 파일은 작업지시자 지시에 따라 수정하지 않았다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_agent_preflight_queue.py` | 2 passed |
| `python3 tools/agent_preflight.py --static-only --no-network` | 성공 |
| `python3 -m unittest scripts/tests/test_workflow_contract_wiring.py` | 3 passed |
| `actionlint .github/workflows/ci.yml` | 성공, 워크플로 변경 없음 |
| `git diff --check` | 성공 |

Cargo 전체 회귀는 Python·문서 보정 범위이므로 실행하지 않았다.

**메인터너 보정 포함 통합 수용.** 보정 code/test는 #4106 원 source가 아니라 누적 통합 PR에만
반영한다. 따라서 통합 PR의 최신 head full CI가 merge 전 조건이다.
