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
| 작성 시점 참고 상태 | #4385 `OPEN` / `MERGEABLE`, remote head가 원 PR head와 일치 |
| 1차 메인터너 보정 | `a595e58c8c5bfcfe0e48c5f373341844aca86196` — `docs(roadmap): distinguish #4385 evidence states` |
| 후속 상태 보정 | `5b32cff82a930c845859b2386e6fc742f7ac6e70` — `docs(roadmap): correct #4385 W32 evidence states` |

원 변경은 2026년 하네스 논문 다섯 편을 rhwp의 구현·로드맵과 대사하고 SVG 전도를
추가한다. 메인터너 보정은 그 Markdown·SVG와 이 review·구현 기록만 바꾸며 source,
test, workflow, fixture, baseline에는 영향이 없다. contributor history는 그대로 두고
원 head 뒤에 single-parent 문서 commit만 추가한다.

## 발견한 차단 결함

- 다섯 편 전체를 같은 시기 신간으로 썼지만 최초 제출일은 2026-04-18,
  2026-06-11, 2026-07-04, 2026-07-30, 2026-08-05로 나뉜다. 2026 W32에 최초
  제출된 논문은 2608.05446 한 편뿐이다.
- Markdown은 open PR #4330, #4361, #4381의 기능을 "머지된 실물"과 섞었다.
  R100 공개 실험 프로토콜도 PR #4356에서 검토 중인데 이미 열린 계약처럼 서술했다.
- SVG는 전체 층과 R83까지 머지됐다고 적었다. R83 schemaRegistry 대사 채널은
  open PR #4330 상태다. 패키징·workspace·CAS 등 다른 open PR도 머지 기능과
  시각적으로 구분되지 않았다.

1차 보정 뒤 W33 재검증에서 독립 오류를 확인했다. 2604.17025의 version history는
v1 2026-04-18, v2 2026-04-25, v3 2026-05-04인데 v2를 05-04로 적었다. 상대 시점
표현도 W33에서 의미가 바뀌므로 모두 `2026 W32`로 고정해야 한다. 또한 atlas는
open PR #4371, version policy와 schemaRegistry는 open PR #4330(R67/R83), 트랙
S·R101~R200은 open PR #4364인데 머지 실물이나 번호 없는 계획처럼 분류했다. SVG는
기본 run 계획까지 #4381의 CAS와 함께 검토 중으로 묶고 R67, #4364, #4371 상태를
빠뜨렸다.

## 메인터너 보정

- 모든 상대 시점을 `2026 W32`로 고정하고 각 arXiv abstract/version history의
  최초 제출일과 개정일을 적었다. 2604.17025는 v1 04-18, v2 04-25, v3 05-04다.
- 표의 구현 근거를 **머지 실물**, **검토 중 PR**, **로드맵 계획**으로 나눴다.
  open PR은 변경·종료될 수 있으며 채택·배포된 기능이 아니라고 명시했다.
- #4330(schemaRegistry/R83), #4361(workspace), #4381(CAS), #4356(R100 실험)의
  검토 중 상태를 Markdown에 표시했다.
- atlas #4371, version policy+schemaRegistry R67/R83 #4330, 트랙 S·R101~R200
  #4364를 각각 open PR로 분류했다.
- SVG 제목·각 층·실행 계층·유입 채널에서 머지 실물과 검토 큐를 분리했다.
  기본 run 계획은 머지, CAS만 #4381 검토 중으로 갈랐고 R67/R83 #4330,
  R101~R200 #4364, atlas #4371 상태를 명시했다.

## 2026-08-10 GitHub 상태 스냅샷

| PR | 확인 상태 | 확인 head |
| --- | --- | --- |
| #4330 | `OPEN`, 미머지 | `b54615026b97187050851c4b00c127e48911be64` |
| #4364 | `OPEN`, 미머지 | `39fb0a707cab7e1e9453cdce874c5a317d284c1e` |
| #4371 | `OPEN`, 미머지 | `863fbe9e850ed91327edc30316bc583eaf9a4113` |
| #4381 | `OPEN`, 미머지 | `026b947f64bffe807fda98a46c0aba2b7ba2c7c1` |

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| arXiv 날짜·링크 대조 | 5개 `abs` 링크와 제출일을 확인하고 2604.17025의 v1 04-18/v2 04-25/v3 05-04를 고정 |
| 절대 시점 검사 | 조사·review 기록에서 상대 시점 문구를 제거하고 `2026 W32`를 사용 |
| 현재 PR 상태 검사 | `gh pr view`로 #4385 source가 불변이고 #4330/#4364/#4371/#4381이 모두 `OPEN`·미머지임을 확인 |
| 구현 상태 계약 검사 | atlas #4371, R67/R83 #4330, R101~R200·트랙 S #4364, CAS #4381을 검토 중으로 표시 |
| SVG 상태 검사 | 기본 run 계획과 CAS를 분리하고 R67/R83 #4330, #4364, #4371 상태가 존재 |
| SVG well-formed·안전 검사 | XML parser 통과. script, event handler, `foreignObject`, href/xlink, `javascript:`/`data:` 참조 없음 |
| Markdown 상대 링크 검사 | 조사 문서와 review·구현 기록의 저장소 내부 링크 통과 |
| `python scripts/check_document_metadata.py` | 통과. 문서 522개의 front matter·canonical 관계 이상 없음 |
| `git diff --check origin/pr/4385..HEAD` | 통과 |
| Cargo·renderer visual sweep | 생략. 실행 renderer가 아니라 문서용 자체 포함 SVG의 텍스트·상태만 수정 |

## 리스크와 권고

- open PR 상태가 바뀌면 Markdown과 SVG를 같은 commit에서 다시 대사해야 한다.
- 위 상태는 2026-08-10의 읽기 전용 스냅샷이다. 해당 PR이 먼저 merge·close되면
  #4385 merge 전 상태 표기와 SVG를 다시 갱신한다.
- SVG는 외부 자산·script 없는 텍스트 전도다. 브라우저 실행 경로나 제품 renderer
  baseline을 바꾸지 않는다.
- 최신 PR head의 required checks와 mergeability는 실제 push 뒤 다시 확인해야 한다.
- 이 로컬 보정은 remote에 push하거나 GitHub 상태를 바꾸지 않았다.

**논문 시점과 PR 상태 구분을 유지하는 조건으로 merge 후보에 둘 수 있다.**
