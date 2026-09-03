# PR #6664 통합 검토 기록

## 판정: 승인

- PR: [#6664](https://github.com/edwardkim/rhwp/pull/6664)
- 원 head: `ff9b24a75e1fbcc25731643ffd7fdf38b5de6e4b`
- 대상 브랜치: `devel`
- 통합 브랜치: `review/jeong-sik-nondraft-20260903`
- 검토 대상 integration head: `c89a7bf56d00acd465e18e4c50864434b64b83d4`
- 기준 `upstream/devel`: `eb2ea3addfc84e1fb472311d8c3132fc245f674b`
- 사전 담당자 지정: `jangster77`

이 판정은 #6664의 제한된 9쪽 continuation-page 중첩 표 수정 범위에 대한 승인이다.

## 라우팅

- base route: `collaborator_external_pr.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `multi_pr_update_branch.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md` 및 위 modifier 문서

## provenance

- 원 PR head `ff9b24a75e1fbcc25731643ffd7fdf38b5de6e4b`
- provenance-preserving cherry-pick `f3f9e25b9`
- #6664 code에는 별도 메인터너 code 보정을 추가하지 않았다.

## 검토 결과

- continuation page에 표 전용 host line만 남은 경우, 줄 높이만큼 `para_y`를 이동한 뒤 표를 다시
  배치하던 이중 전진을 제거한다.
- 표가 host line 시작 위치에 배치되고 뒤따르는 flow cursor는 유지된다.
- 조건을 visible text가 없는 table-host fragment로 제한해 일반 중첩 표 배치에는 적용하지 않는다.

## 이슈 종료 경계

이 변경은 `hwpx_sample2.hwp` 9쪽의 약 161px 빈 띠만 해결한다. 8쪽 첫 문단의 15px 편차,
표 이후 누적 27.8px 편차와 9쪽의 잔여 10.6px는 범위 밖이므로
[#6653](https://github.com/edwardkim/rhwp/issues/6653)은 닫지 않는다.

## 시각 증적

- [hwpx_sample2 9쪽 비교](https://raw.githubusercontent.com/jeong-sik/rhwp/37fdc3bc458e3b8b5c6fcaad1b48d340a35695c4/01-hwpx-sample2-p9.png)

위 자료는 contributor의 immutable commit에 고정된 한/글 PDF, 수정 전, 수정 후 비교다.

## 검증

- 원 head GitHub checks: `28 success`, `5 expected skip`, non-success 없음.
- 원 head 상태: `MERGEABLE`, `CLEAN`.
- `issue_6653_nested_table_host_line`: `1 passed`.
- 전체 release-test: `8977 passed`, `46 skipped`, 실패 0.
- native/WASM/workspace Clippy와 workspace build 통과.
- Native Skia 전체 및 필수 focused 2종 통과.
- 직접 WASM package build 통과: 2분 38초.
- `git diff --check`와 test-suite manifest 계약 통과.

## 병합 전 조건

최종 통합 PR 본문에서 #6653을 닫지 않고, 이 review 기록을 포함한 최신 head의 required CI와
mergeability를 다시 확인한 뒤에만 병합한다.
