---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4729 검토 - 외부 현실 채점 축

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4729](https://github.com/edwardkim/rhwp/pull/4729) |
| 작성자 / source | @kevin9327 / `task_m100_redteam` |
| 대상 / 원 head | `devel` / `7fce209b10938d13f518841d8bead2463bdd3215` |
| 누적 적용 | `4d2264b84` |
| 규모 | 4개 파일, +310/-0, 1개 commit |
| 관련 이슈 | #4728 (`closes`) |
| 작성 시점 참고 상태 | `MERGEABLE`, `CLEAN`, reviewer @jangster77 지정 |

## 메인터너 보정

`--live`가 gh·npm 갱신을 안내하지만 구현은 GitHub 값만 갱신했고, `measuredAt`도 남아 있었다.
`38a51a011`에서 npm downloads API 갱신, 부분 실패 허용, 실제 갱신 시 측정일 갱신을 추가했다.
또한 외부 채택이 프로젝트 별 수보다 작아야 한다고 가정한 테스트는 잘못된 상한이므로,
`db96780a7`에서 프로젝트 견인과 메타 채택의 독립 집계만 검증하도록 수정했다.

## 완료한 검증

네트워크를 쓰지 않는 모의 호출로 GitHub·npm·측정일 갱신을 확인했다. `test_reality_check.py`를
포함한 누적 Python 계약 55건, `reality_check.py --json`, JSON 파싱, 프레임 가드와 Markdown 링크
검사가 통과했다.

## 판정

**메인터너 보정 후 통합 수용 권고.** 이 변경은 점검 도구와 문서·CI 계약이며 renderer 시각
검증 대상은 아니다. 통합 PR의 최신 head CI와 작업지시자 승인을 다시 확인한 뒤에만 merge한다.
