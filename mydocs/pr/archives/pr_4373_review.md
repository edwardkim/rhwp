---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4373 검토 - Setup action 반복 호출 격리

## 검토 경로

기본 경로는 `maintainer_general.md`, 보조 경로는 `intake_and_review.md`,
`local_validation.md`, `multi_pr_update_branch.md`, `review_only_fast_pass.md`다.
Composite action과 자체검증 workflow만 바뀌므로 renderer와 시각 fixture 영향은 없다.

## 접수 메타데이터

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#4373](https://github.com/edwardkim/rhwp/pull/4373) / `kevin9327` |
| 관련 이슈 | [#4353](https://github.com/edwardkim/rhwp/issues/4353) |
| base / contributor head | `devel` / `c575d1a69e3940a93aaeb624bb7d2d9fac45f07a` |
| 규모 | 2 files, +116 / -0, contributor commits 3개 |
| 상태 | `MERGEABLE` / `CLEAN`, Full CI·CodeQL 및 3-OS Action Self-test 성공 |
| 가시성 branch | `review/kevin9327-20260810-pr4373` |
| 메인터너 code candidate | `1e7c844472da1b101f6fbcc3808bb93c9b163c89` |

## Contributor 변경 범위

`3cc9097730e7d1bc0c9fbd48e97c209bbe2d8a26`은 release asset을 checksum 검증 후
PATH에 추가하는 root composite action과 Linux/Windows/macOS self-test를 만들었다.
`cad397a1894e82f69a0c17f1356b071143816df5`는 smoke의 EPIPE를,
`c575d1a69e3940a93aaeb624bb7d2d9fac45f07a`는 macOS checksum 도구 차이를 보정했다.

## 원래 차단점

모든 호출이 `${RUNNER_TEMP}/rhwp-setup`을 공유했다. 같은 job에서 action을 두 번 호출하면
두 번째 `gh release download`와 압축 해제가 첫 호출의 파일과 충돌하고, 기존 self-test는 호출을
한 번만 수행해 이 계약을 검증하지 못했다.

## 메인터너 보정

`1e7c844472da1b101f6fbcc3808bb93c9b163c89`
(`fix(maintainer): #4373 반복 설치 경로를 격리`)은 다음을 추가했다.

- `action.yml`: 호출마다 `mktemp`로 고유 경로를 만들고 Windows Git Bash에서는
  `cygpath`로 입력·PATH 경계를 변환한다.
- `.github/workflows/action-selftest.yml`: 같은 job에서 동일 release를 두 번 설치한다.
- `scripts/tests/test_setup_action.py`: 고유 경로와 2회 호출 계약을 고정한다.
- `.github/workflows/ci.yml`: focused contract test를 lint job에 배선한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `python -m unittest scripts.tests.test_setup_action scripts.tests.test_workflow_contract_wiring -v` | 5 / 5 통과 |
| `git diff --check origin/pr/4373..1e7c844472da1b101f6fbcc3808bb93c9b163c89` | 통과 |
| commit graph | correction commit의 유일한 parent가 contributor head와 일치 |

로컬에는 `actionlint`가 없고 실제 hosted runner release 다운로드를 재현하지 않았다. 새 self-test가
Linux, Windows Git Bash, macOS에서 두 번 연속 성공하는지 최신 원격 head에서 확인해야 한다.

## 최종 권고

**메인터너 보정 포함 조건부 수용 권고.** correction이 action, test, CI workflow를 바꾸므로 기존
3-OS 녹색 결과는 재사용 대상이 아니다. push 승인 뒤 두 trailing commit을 fast-forward로 반영하고
최신 Full CI, CodeQL, Action Self-test 3종, required aggregate와 mergeability를 확인한다.
별도 merge 승인 전에는 review 게시나 merge를 수행하지 않는다.

실행 및 rollback은 [PR #4373 구현·통합 계획](pr_4373_review_impl.md)을 따른다.
