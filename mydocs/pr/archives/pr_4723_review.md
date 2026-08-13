---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4723 검토 - 에이전트 조직과 자동 배치

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4723](https://github.com/edwardkim/rhwp/pull/4723) |
| 작성자 / source | @kevin9327 / `task_m100_org` |
| 대상 / 원 head | `devel` / `1e67907778979f20334b66632635b6f5d2fea35b` |
| 누적 적용 | `d3cf3e234` |
| 규모 | 9개 파일, +605/-0, 1개 commit |
| 관련 이슈 | #4720 |
| 작성 시점 참고 상태 | `MERGEABLE`, `CLEAN`, reviewer @jangster77 지정 |

## 메인터너 보정

접수 JSON을 직접 전달하면 JSON Schema enum을 거치지 않아, 존재하지 않는
`targetDepartment`가 조용히 접수처로 바뀌었다. 메인터너 보정 `38a51a011`은 미지정과
`any`만 접수처로 보내고, 목록 밖 id는 CLI 사용법 오류(exit 2)로 거부하도록 바꿨다.
스키마 설명과 README를 같은 의미로 갱신하고 단위 회귀를 추가했다.

## 완료한 검증

`test_agent_org.py`를 포함한 누적 Python 계약 55건이 통과했다. 실제 stdin 호출도
`unknown` 부서를 `agent_dispatch.py: error: 알 수 없는 부서 id: unknown`으로 거부했다.
JSON 파싱, Markdown 링크, 최신 기준선 merge tree와 공백 검사도 통과했다. 통합 PR #4733의
code candidate `db96780a7`은 Full CI·CodeQL을 모두 통과했다.

## 판정

**메인터너 보정 후 통합 수용 권고.** renderer 출력 변경은 없으며, 통합 PR의 최신 head CI와
trailing docs-only head의 fast-pass와 mergeability, 작업지시자 승인을 다시 확인한 뒤에만 merge한다.
