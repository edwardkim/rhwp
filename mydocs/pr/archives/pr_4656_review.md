---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4656 검토 - Gym pack·leaderboard·release gate 확장

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md
```

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4656](https://github.com/edwardkim/rhwp/pull/4656) · @kevin9327 |
| 관련 이슈 | [#4653](https://github.com/edwardkim/rhwp/issues/4653) |
| 원 head | `7cfa61f0db2571d3aab15283e238ff1a9a368e46` |
| 원 PR 상태 | `OPEN`, `MERGEABLE`, maintainer 수정 허용 |
| 통합 기준선 | `upstream/devel` `1449474aaf5411e069afeb2954edefd13438eb52` |
| 중복 선행 commit | `bdabccfc`는 #4652와 같은 #4600 변경이므로 재적용하지 않음 |
| reviewer | `edwardkim`, `jangster77` reviewer request 완료 |

## 변경 판단

pack 단위 Gym 과제, 채점기·leaderboard·차등 비교·릴리스 게이트와 CI contract를 추가한다. 원 PR은
오래된 기준선에서 #4600을 중복 포함하므로, 최신 `devel` 위 누적 branch에는 중복 commit을 제외하고
기능 commit만 순서대로 적용했다.

## 메인터너 보정

- source history에 섞인 `bindings/node/dist` 36개 산출물을 통합 diff에서 제외했다.
- `release_gate.py`가 검증 단계에서 `--new` 바이너리를 잃어버리던 문제를 고쳤다.
- `build_baseline.py`가 기준 풀이를 생성만 하고 실제 채점하지 않던 문제를 고쳤다.
- legacy core 제출물이 없을 때 `release_diff.py`가 예외로 중단하던 문제를 관측값으로 처리하게 고쳤다.
- CI가 존재하지 않는 `test_setup_action.py`를 실행하던 stale 배선을 제거하고, policy test가 무시된
  로컬 바인딩 산출물을 Git 추적 배포물로 잘못 판단하지 않게 고쳤다.

보정 commit과 재현 결과는 [통합 이행 기록](pr_4652_4656_4666_review_impl.md)에 분리해 남긴다.

## 완료한 검증

- Gym score·pack·leaderboard·release diff·release gate 및 CI workflow contract Python test를 실행했고
  최종 명령은 모두 통과했다.
- 기준 풀이 생성·즉시 채점은 reference를 가진 86개 과제에서 성공 86, 실패 0이었다.
- release diff는 100개 과제·122개 관측 비교에서 `stable`, release gate는 `stable · 0`으로 끝났다.
- `git diff --check upstream/devel...HEAD`: 통과.

## 남은 추적 항목과 최종 판단

`core-cli`의 14개 legacy 과제는 현재 추적된 `reference/`와 과거 baseline 제출물이 완전하지 않아,
이번 기준 풀이 재생성 범위에는 포함되지 않았다. 따라서 과거 scorecard의 전체 수치가 현재 저장소만으로
완전히 재현된다고 주장하지 않는다. 이는 통합 뒤 기준자료 완결성 과제로 별도 추적할 항목이다.

**통합 후보 수용.** Rust 소스는 변경하지 않았으므로 작업지시자 지시에 따라 전체 Cargo 회귀는 생략했다.
원격 통합 PR 생성·CI·merge와 원 PR close/comment는 작업지시자 승인 뒤에만 수행한다.
