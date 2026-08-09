---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4385 검토 — 논문 시점과 구현 상태가 구분된 하네스 대사

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `review_only_fast_pass.md`

loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`pr_review/maintainer_general.md`, `pr_review/intake_and_review.md`,
`pr_review/local_validation.md`, `pr_review/multi_pr_update_branch.md`,
`pr_review/review_only_fast_pass.md`

## 메타데이터와 적용 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4385](https://github.com/edwardkim/rhwp/pull/4385) / @kevin9327 |
| base | `devel` |
| 원 PR head | `fb0b069803a14bcaa5ba8c99d965633d0e22d23f` |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4385` |
| 원 변경 규모 | 조사 Markdown·손제작 SVG 2파일, `+119/-0`, contributor 커밋 1개 |

원 변경은 2026년 하네스 논문 다섯 편을 rhwp의 구현·로드맵과 대사하고 SVG 전도를
추가한다. 메인터너 보정은 그 Markdown·SVG와 이 review·구현 기록만 바꾸며 source,
test, workflow, fixture, baseline에는 영향이 없다. contributor history는 그대로 두고
원 head 뒤에 single-parent 문서 commit만 추가한다.

## 발견한 차단 결함

- 다섯 편 전체를 "이번 주 arXiv 신간"으로 썼지만 최초 제출일은 2026-04-18,
  2026-06-11, 2026-07-04, 2026-07-30, 2026-08-05로 나뉜다. 이번 주 신간은
  2608.05446 한 편뿐이다.
- Markdown은 open PR #4330, #4361, #4381의 기능을 "머지된 실물"과 섞었다.
  R100 공개 실험 프로토콜도 PR #4356에서 검토 중인데 이미 열린 계약처럼 서술했다.
- SVG는 전체 층이 머지됐다고 표방하고 `R81–R84 머지`라고 적었다. R83
  schemaRegistry 대사 채널은 open PR #4330 상태다. 패키징·workspace·CAS 등 다른
  open PR도 머지 기능과 시각적으로 구분되지 않았다.

## 메인터너 보정

- 시점을 "이번 주에 대사한 2026년 논문 5편"으로 고치고 각 arXiv abstract/version
  history의 최초 제출일과 개정일을 적었다.
- 표의 구현 근거를 **머지 실물**, **검토 중 PR**, **로드맵 계획**으로 나눴다.
  open PR은 변경·종료될 수 있으며 채택·배포된 기능이 아니라고 명시했다.
- #4330(schemaRegistry/R83), #4361(workspace), #4381(CAS), #4356(R100 실험)의
  검토 중 상태를 Markdown에 표시했다.
- SVG 제목·각 층·실행 계층·유입 채널에서 머지 실물과 검토 큐를 분리했다.
  R83은 `R83 PR #4330 (검토 중)`으로 정정했다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| arXiv 날짜·링크 대조 | 5개 ID의 `abs` 링크, 최초 제출일 2026-04-18/06-11/07-04/07-30/08-05와 알려진 v2 날짜를 문서에서 확인 |
| 구현 상태 계약 검사 | #4330/#4361/#4381/#4356을 검토 중으로 표시하고 "이번 주 신간 5편", "전 층이 머지" 표현을 제거 |
| SVG 상태 검사 | `R81·R82·R84 머지 · R83 PR #4330 (검토 중)`과 각 open PR 상태가 존재 |
| SVG well-formed·안전 검사 | XML parser 통과. script, event handler, `foreignObject`, href/xlink, `javascript:`/`data:` 참조 없음 |
| Markdown 상대 링크 검사 | 조사 문서와 review·구현 기록의 저장소 내부 링크 통과 |
| `python scripts/check_document_metadata.py` | 통과. 문서 522개의 front matter·canonical 관계 이상 없음 |
| `git diff --check origin/pr/4385..HEAD` | 통과 |
| Cargo·renderer visual sweep | 생략. 실행 renderer가 아니라 문서용 자체 포함 SVG의 텍스트·상태만 수정 |

## 리스크와 권고

- open PR 상태가 바뀌면 Markdown과 SVG를 같은 commit에서 다시 대사해야 한다.
- SVG는 외부 자산·script 없는 텍스트 전도다. 브라우저 실행 경로나 제품 renderer
  baseline을 바꾸지 않는다.
- 최신 PR head의 required checks와 mergeability는 실제 push 뒤 다시 확인해야 한다.
- 이 로컬 보정은 remote에 push하거나 GitHub 상태를 바꾸지 않았다.

**논문 시점과 PR 상태 구분을 유지하는 조건으로 merge 후보에 둘 수 있다.**
