# PR #6661 통합 검토 기록

## 판정: 메인터너 보정 후 수용 가능

- PR: [#6661](https://github.com/edwardkim/rhwp/pull/6661)
- 원 head: `847810448e3569fef1e28466ec776361682445e1`
- 대상 브랜치: `devel`
- 통합 브랜치: `review/jeong-sik-nondraft-20260903`
- 기준 `upstream/devel`: `eb2ea3addfc84e1fb472311d8c3132fc245f674b`
- 메인터너 보정: `c89a7bf56d00acd465e18e4c50864434b64b83d4`
- 검토 대상 integration head: `c89a7bf56d00acd465e18e4c50864434b64b83d4`
- 사전 담당자 지정: `jangster77`

## 라우팅

- base route: `collaborator_external_pr.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `multi_pr_update_branch.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md` 및 위 modifier 문서

## provenance

- 원 PR head `847810448e3569fef1e28466ec776361682445e1`
- provenance-preserving cherry-pick `e33b0e324`
- 메인터너 보정 `c89a7bf56d00acd465e18e4c50864434b64b83d4`

## 원 head 보류 사유

원 구현은 셀 안 모든 `Para` 기준 inline 그림에 `horzOffset`을 적용한다. 현재 corpus에서는
재분류된 그림 스택만 이 조건에 걸리지만, 원문 inline 그림과 정규화가 만든 합성 그림을 구분하는
provenance가 없어 향후 일반 inline 그림까지 같은 규칙으로 이동시킬 수 있다.

원 PR 본문도 합성 `LineSeg`에 `TAG_IMPLEMENTATION_PROPERTY`가 빠졌다고 기록하고 있다.

## 메인터너 보정

- `reclassify_cell_floating_stacks`가 만든 합성 line segment에 `TAG_IMPLEMENTATION_PROPERTY`를 남긴다.
- 셀 inline 배치는 해당 synthetic tag가 있는 줄에서만 `Para` 기준 `horzOffset`을 복원한다.
- 파일에서 원래 inline이었던 그림에는 이 offset 규칙을 일반 적용하지 않는다.

이 보정은 contributor의 목표인 5~8쪽 그림별 offset 복원을 유지하면서 적용 대상을 정규화 산출물로
제한한다.

## 시각 증적

- [issue2004 6쪽 offset 비교](https://raw.githubusercontent.com/jeong-sik/rhwp/c5d2dfbdd6beedbc5aa3d63838f67a6a01ea343d/01-p6-offset-2092.png)
- [issue2004 4쪽 offset 0 대조군](https://raw.githubusercontent.com/jeong-sik/rhwp/c5d2dfbdd6beedbc5aa3d63838f67a6a01ea343d/02-p4-offset-0.png)

위 자료는 contributor의 immutable commit에 고정된 한/글 PDF, 수정 전, 수정 후 비교다. 남는 공통
약 3.6px 표 바깥 여백은 #6643 범위이며 #6655의 그림별 offset 손실과 구분한다.

## 검증

- 원 head GitHub checks: `28 success`, `5 expected skip`, non-success 없음.
- 원 head 상태: `MERGEABLE`, `CLEAN`.
- 보정 head `issue_6655_inline_stack_horz_offset`: `1 passed`.
- 전체 release-test: `8977 passed`, `46 skipped`, 실패 0.
- native/WASM/workspace Clippy와 workspace build 통과.
- Native Skia 전체 및 필수 focused 2종 통과.
- 직접 WASM package build 통과: 2분 38초.
- `git diff --check`와 test-suite manifest 계약 통과.

## 병합 전 조건

원 head를 직접 merge하지 않는다. 보정 SHA를 포함한 integration head의 최신 GitHub CI와
mergeability를 다시 확인한 뒤에만 수용하며, #6655 후속 처리는 실제 통합 merge와 devel CI 성공 뒤에
`post_merge.md`를 따른다.
