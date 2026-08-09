---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4356 검토 — R100 공개 실험의 재현 가능한 시작 경계

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
| 원 PR / 작성자 | [#4356](https://github.com/edwardkim/rhwp/pull/4356) / @kevin9327 |
| base | `devel` |
| 원 PR head | `125176f6eb1b7b78fc1b8a7bf5e58cc63c7322d3` |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4356` |
| 원 변경 규모 | 실험 프로토콜 1파일, `+77/-0`, contributor 커밋 1개 |

원 변경은 R100의 "30분 첫 유효 산출" 공개 실험 절차를 제안한다. 메인터너
보정은 프로토콜과 이 review·구현 기록만 바꾸며 source, test, workflow, fixture,
baseline에는 영향이 없다. contributor history는 그대로 두고 원 head 뒤에
single-parent 문서 commit만 추가한다.

## 발견한 차단 결함

원 프로토콜은 시작점을 "저장소 클론" 또는 "릴리스 바이너리" 중 하나라고 하면서
설치·클론을 모두 측정에 포함한다고 적었다. 두 시작 상태는 준비 작업과 발견 표면이
달라 같은 대장의 30분 결과를 비교할 수 없다. 또한 "첫 명령"은 명령 전 추론 시간을
제외하고, 참가자에게 과제표의 명령·검증 힌트를 어디까지 전달하는지도 정의하지 않았다.

## 메인터너 보정

- 시작 상태를 `빈 작업 디렉터리 + 네트워크 + git + 저장소 URL` 하나로 고정했다.
  checkout, binary, package, build 산출물과 warm cache는 사전 제공하지 않는다.
- 진행자가 전달하는 정보를 저장소 URL, 자연어 목표, 입력 파일, 제출 위치로 한정했다.
  과제 판정표, 명령 이름·flag·예상 key·검증 명령은 참가자에게 주지 않는다.
- `t0`를 고정 전달 패키지 전송 시각, `t1`을 진행자가 유효 산출을 확인한 시각으로
  정의했다. clone·설치·탐색·추론을 포함하며 무효 제출 뒤에도 시계가 계속 돈다.
- 저장소와 실행 파일에서 참가자가 스스로 찾은 README, `llms.txt`, `--help`,
  `capabilities`만 무안내 discovery로 인정한다. 추가 힌트나 외부 해법을 쓰면
  "안내됨" cohort로 분리한다.
- 결과 양식과 대장을 고정 시작 상태에 맞게 바꾸고, 다른 시작 상태는 별도 protocol로
  분리하도록 했다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| 프로토콜 불변식 검사 | `t0`/`t1`, 고정 시작 상태, 전달 목표·입력·제출 위치, 금지 힌트 경계가 모두 존재 |
| 모순 문구 제거 검사 | 기존 2개 시작점과 `클론 \| 릴리스 바이너리` 선택 양식이 제거됨 |
| Markdown 상대 링크 검사 | 프로토콜과 review·구현 기록의 저장소 내부 링크 통과 |
| `python scripts/check_document_metadata.py` | 통과. 문서 522개의 front matter·canonical 관계 이상 없음 |
| `git diff --check origin/pr/4356..HEAD` | 통과 |
| Cargo·시각 검증 | 생략. `mydocs` 아래 Markdown만 변경하며 실행 코드·렌더 출력 영향 없음 |

## 리스크와 권고

- host 성능 차이는 결과 양식의 OS·이미지·preinstalled 도구 필드로 남기고, 집계 시
  같은 환경 cohort끼리 비교해야 한다.
- 최신 PR head의 required checks와 mergeability는 실제 push 뒤 다시 확인해야 한다.
- 이 로컬 보정은 remote에 push하거나 GitHub 상태를 바꾸지 않았다.

**고정 시작 상태와 전달 경계를 유지하는 조건으로 merge 후보에 둘 수 있다.**
