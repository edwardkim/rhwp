# PR #6659 통합 검토 기록

## 판정: 승인

- PR: [#6659](https://github.com/edwardkim/rhwp/pull/6659)
- 원 head: `c1a5396df78e14eaa4836a9bb26c202409042cc1`
- 대상 브랜치: `devel`
- 통합 브랜치: `review/jeong-sik-nondraft-20260903`
- 검토 대상 integration head: `c89a7bf56d00acd465e18e4c50864434b64b83d4`
- 기준 `upstream/devel`: `eb2ea3addfc84e1fb472311d8c3132fc245f674b`
- 사전 담당자 지정: `jangster77`

이 판정은 원 PR metadata를 그대로 사용한 직접 merge 승인이 아니라, 원 code commit을 최신
`upstream/devel` 위의 통합 head에서 제한된 범위로 수용한다는 판정이다.

## 라우팅

- base route: `collaborator_external_pr.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `multi_pr_update_branch.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md` 및 위 modifier 문서

## provenance

- 원 PR head `c1a5396df78e14eaa4836a9bb26c202409042cc1`
- provenance-preserving cherry-pick `aafc00e7a`
- #6659 code에는 별도 메인터너 code 보정을 추가하지 않았다.
- 다른 source PR의 보정을 포함한 현재 integration head는 `c89a7bf56d00acd465e18e4c50864434b64b83d4`다.

## 검토 결과

- 저장된 다음 `vertical_pos` 차이를 문단 내 시각 줄 전진에 사용해 개체가 있는 줄 아래의 누적
  세로 편차를 줄이는 변경이다.
- focused 계약은 `hwpctl_ParameterSetID_Item_v1.2.hwp`의 대상 줄과 그림 위치를 고정한다.
- `text_overlap_baseline.tsv` 변경은 전체 baseline gate에서 통과했다.
- pagination fit에 같은 시각 전진값을 직접 적용하는 보정은 포함하지 않는다. 이전 실험에서 페이지 수,
  off-canvas, endnote 및 overlap 회귀를 일으켰으므로 renderer의 시각 전진과 typeset의 fit budget을
  같은 값으로 일반화하지 않는다.

## 이슈 종료 경계

원 PR 본문의 `closes #6656`은 통합 PR의 closing reference로 승계하지 않는다. #6656 DoD 중
`height_measurer` 측정 경로와의 정합은 이 변경 범위에서 해결되지 않았으므로
[#6656](https://github.com/edwardkim/rhwp/issues/6656)은 열린 상태로 유지한다.

## 시각 증적

- [hwpctl 3쪽 비교](https://raw.githubusercontent.com/jeong-sik/rhwp/30f19d796f629c9a0cebdce1843ef73d16c10229/01-hwpctl-p3.png)
- [exam_math 3쪽 비교](https://raw.githubusercontent.com/jeong-sik/rhwp/30f19d796f629c9a0cebdce1843ef73d16c10229/02-exam-math-p3.png)
- [3-09월 교육통합 6쪽 비교](https://raw.githubusercontent.com/jeong-sik/rhwp/30f19d796f629c9a0cebdce1843ef73d16c10229/03-edu-2024-p6.png)

위 자료는 contributor의 immutable commit에 고정된 한/글 PDF, 수정 전, 수정 후 비교다.

## 검증

- 원 head GitHub checks: `30 success`, `3 expected skip`, non-success 없음.
- 원 head 상태: `MERGEABLE`, `CLEAN`.
- `issue_6656_line_advance_text_height`: `1 passed`.
- 전체 release-test: `8977 passed`, `46 skipped`, 실패 0.
- native/WASM/workspace Clippy와 workspace build 통과.
- Native Skia 전체 및 필수 focused 2종 통과.
- 직접 WASM package build 통과: 2분 38초.
- `git diff --check`와 test-suite manifest 계약 통과.

## 병합 전 조건

최종 통합 PR 본문에서 #6656을 닫지 않고, 이 review 기록을 포함한 최신 head의 required CI와
mergeability를 다시 확인한 뒤에만 병합한다.
