# PR #4824 자체검토 — workflow PR 후행 review-only fast-pass

## 절차 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
  review_only_fast_pass.md, rework_and_exceptions.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  review_only_fast_pass.md, rework_and_exceptions.md
```

## PR 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4824](https://github.com/edwardkim/rhwp/pull/4824) |
| 관련 이슈 | [#4740](https://github.com/edwardkim/rhwp/issues/4740) |
| 작성자 | `edwardkim` (maintainer self PR) |
| base / head | `devel` / `task_m100_4740_trusted_review_fast_pass` |
| code candidate | `09b923592e98c368dc06dd7d78294bdd9e2f3130` |
| 규모 | 3 commits, 13 files, +1,045 / -27 |
| code candidate 상태 | `MERGEABLE`, `CLEAN`, required checks 성공 |

maintainer 자신의 PR이므로 외부 reviewer를 지정하지 않고 자체검토로 처리한다. 이 PR은 1,000줄을
넘으므로 즉시 admin merge하지 않으며, 코드 후보 검토·CI와 review-only trailing head 검증을 서로 다른
cycle로 수행한다.

## 변경 범위와 설계 판단

- default branch의 `CI Impact Policy Controller`가 same-repository PR의 Full 검증 후보와 현재 head 사이
  first-parent 계보를 독립적으로 확인하고, current-base merge tree의 CI 영향까지 다시 분류한다.
- CI·CodeQL·Render Diff consumer는 controller가 exact head·base·run에 결합해 발행한 `rfp=1` commit
  status만 신뢰한다. status 부재·만료·불일치·외부 fork·정책 버전 불일치는 모두 Full 실행으로 닫힌다.
- 기존 정책이 workflow 변경을 항상 execution surface로 분류하는 경계는 유지한다. fast-pass는 이미 Full
  검증된 후보 뒤에 review-only 파일만 이어진 제한된 tail에서만 heavy worker를 재사용한다.
- status producer, 세 consumer, policy v5, 회귀 테스트와 운영 문서는 하나의 버전 계약이다. 이를 별도 PR로
  나누면 producer·consumer 또는 정책 버전이 맞지 않는 부분 배포 상태가 생기므로 한 PR로 유지했다.
- workflow와 정책·문서만 변경하며 renderer, layout, HWP/HWPX fixture, golden에는 영향이 없다. 따라서
  시각 검증과 WASM·브라우저 제품 검증은 적용하지 않았다.

## 자체검토에서 발견해 보정한 사항

초기 code candidate `68537dc73`의 review tail 판정은 각 commit이 single-parent인지만 확인하고, 그 parent가
바로 다음 검사 대상 SHA인지 강제하지 않았다. 이 상태에서는 API 응답에 분리된 ordinary commit이 끼어도
불연속 계보를 review-only tail로 오인할 여지가 있었다.

`09b923592`에서 다음을 보정했다.

- classifier가 head에서 candidate까지 `expectedSha`를 갱신하며 exact first-parent 연속성을 검증한다.
- controller가 PR commits 응답의 마지막 SHA와 live `pull_request.head.sha`가 같은지 확인한다.
- ordinary tail과 bridge tail 양쪽에 불연속 계보 회귀 계약을 추가하고, head 불일치가 fail-closed인지
  workflow 계약으로 고정한다.

이 보정 뒤 동일 범위 재검토에서 추가 차단 사항은 발견하지 않았다.

## 완료된 로컬 검증

- `node --test scripts/tests/ci-impact-classifier.test.cjs` — 31 passed
- `node --test scripts/tests/ci-impact-policy.test.cjs` — 31 passed
- 선택한 workflow Python 계약 — 70 passed
- PyYAML로 변경된 workflow 4개 parse — 통과
- manual 링크 검사와 `git diff --check upstream/devel...HEAD` — 통과
- 로컬에 `actionlint`가 없어 실행하지 못했으며, GitHub Actions의 실제 workflow parse·실행으로 보완한다.

## GitHub Actions와 활성화 경계

- code candidate `09b923592`는 controller가 아직 default branch `main`에 등록되지 않은 상태라 의도대로
  Full 경로를 실행한다.
- exact code candidate에서 CI
  [run 31883675640](https://github.com/edwardkim/rhwp/actions/runs/31883675640), CodeQL
  [run 31883675457](https://github.com/edwardkim/rhwp/actions/runs/31883675457), Render Diff
  [run 31883675475](https://github.com/edwardkim/rhwp/actions/runs/31883675475)가 성공했다. 전체 check
  집계는 20 success, 정책상 3 skip, failure·cancelled·pending 0이다.
- controller는 이 PR이 `devel`에 병합되는 것만으로 활성화되지 않는다. 이후 정상 release가 `main`에
  반영된 뒤 생성되는 새 `pull_request_target` run부터 후행 review-only fast-pass가 작동한다.
- review 문서 trailing head의 결과는 아래 최종 판정 전에 별도로 확인한다.

## 최종 권고

코드 후보의 보안 경계와 회귀 계약은 merge 가능한 범위로 판단한다. 다만 이 자체검토·오늘할일 문서를
trailing commit으로 push한 뒤, 그 최신 head의 required checks가 모두 통과하고 PR이 `MERGEABLE`인지
확인한 다음 메인테이너의 별도 merge 승인을 받아야 한다.
