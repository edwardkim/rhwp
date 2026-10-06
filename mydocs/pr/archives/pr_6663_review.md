# PR #6663 통합 검토 기록

## 판정: 메인터너 보정 후 수용 가능

- PR: [#6663](https://github.com/edwardkim/rhwp/pull/6663)
- 원 head: `241335219626a946c0c25805ed069509bdedae18`
- 대상 브랜치: `devel`
- 통합 브랜치: `review/jeong-sik-nondraft-20260903`
- 기준 `upstream/devel`: `eb2ea3addfc84e1fb472311d8c3132fc245f674b`
- 메인터너 보정: `c89a7bf56d00acd465e18e4c50864434b64b83d4`
- 검토 대상 integration head: `c89a7bf56d00acd465e18e4c50864434b64b83d4`
- 사전 담당자 지정: `jangster77`

## 라우팅

- base route: `collaborator_external_pr.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md` 및 위 modifier 문서

## provenance

- 원 PR head `241335219626a946c0c25805ed069509bdedae18`
- provenance-preserving cherry-pick `bc466368c`
- 메인터너 보정 `c89a7bf56d00acd465e18e4c50864434b64b83d4`

## 원 head 보류 사유

그림 위치 현황 문서가 현재 열린 #6655를 "최근에 닫힌 것"으로 기록했다. review 시점과 integration
candidate 시점 모두 #6655는 `OPEN`이므로 미래의 merge 결과를 완료 사실처럼 적은 문장이었다.

## 메인터너 보정

닫힌 이슈 목록에서 #6655를 분리하고, 이 통합 검토에서 보정 중이며 아직 닫히지 않았다고 기록했다.
그림 954장 현황, oracle 제외 기준, 글꼴 대체 경계와 나머지 원인 순위는 원 문서 내용 그대로 보존했다.

## 검증

- 원 head GitHub checks: `10 success`, `18 expected skip`, non-success 없음.
- 원 head 상태: `MERGEABLE`, `CLEAN`.
- 문서 변경을 포함한 integration head에서 `git diff --check` 통과.
- 같은 integration head의 전체 release-test: `8977 passed`, `46 skipped`, 실패 0.
- 직접 WASM package build까지 통과했으며 문서 보정으로 코드 동작은 추가 변경되지 않았다.

## 병합 전 조건

원 head를 직접 merge하지 않는다. 보정 SHA와 이 기록을 포함한 integration head의 최신 required CI와
mergeability를 확인한 뒤에만 수용한다. #6662의 상태 변경은 통합 PR의 실제 closing reference와
`post_merge.md` 절차를 별도로 따른다.

## 시각 증적 적용 여부

- 이 PR은 working 문서만 변경하므로 별도 시각 fixture 검증 대상이 아니다.
- 함께 검토한 renderer 변경의 시각 결과는 [PR #6659, #6661, #6664 시각 스윕](pr_6659_6664_jeong_sik_visual_sweep.md)에 분리해 기록했다.

## Merge 후 contributor PR comment 계획

- 이 기록 보완 PR이 merge되고 그 merge SHA의 devel CI가 성공한 뒤에만 원 PR #6663에 한 번 게시한다.
- 수용 사실: PR #6691 merge commit 573059ee7bd4b74626143723d31d0b74ab0320b8로 cherry-pick bc466368c를 통합하고, 메인터너 보정 c89a7bf56으로 #6655의 미래 종료 표현을 현재 OPEN 상태에 맞게 고쳤다.
- 실제 CI: PR CI 33747890105와 CodeQL 33747890268, devel CI 33749587952와 CodeQL 33749587899, Adapter 33749587901, Proptest 33749587881이 success다.
- 이 PR은 working 문서 전용이므로 자체 visual fixture나 PNG를 주장하지 않는다. 관련 renderer 시각 증적의 정본은 pr_6659_6664_jeong_sik_visual_sweep.md다.
- #6662는 남은 원인 현황 원장이므로 OPEN을 유지한다.
