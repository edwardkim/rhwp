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

## 최신 보정 head 시각 증적

- 통합 기록: [PR #6659, #6661, #6664 시각 스윕](pr_6659_6664_jeong_sik_visual_sweep.md)
- `issue2004_cell_image_stack.hwpx` p4: [배너·인물·본문 합성](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6661_issue2004_p4.png)
- p6: [후속 본문 합성](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6661_issue2004_p6.png)
- p4-p8: [contact sheet](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6661_issue2004_p4_p8_contact_sheet.png)
- 사람 검토에서 p4 그림 적층·본문 회피와 p5-p8 페이지 흐름이 유지됨을 확인했다. 현재 보정은 synthetic 재분류 stack에만 가로 오프셋을 적용한다.

## Merge 후 contributor PR comment 계획

- 이 기록 보완 PR이 merge되고 그 merge SHA의 devel CI가 성공한 뒤에만 원 PR #6661에 한 번 게시한다.
- 수용 사실: PR #6691 merge commit 573059ee7bd4b74626143723d31d0b74ab0320b8로 cherry-pick e33b0e324를 통합했고, 메인터너 보정 c89a7bf56이 synthetic 재분류 stack에만 가로 오프셋을 제한했다.
- 실제 CI: PR CI 33747890105와 CodeQL 33747890268, devel CI 33749587952와 CodeQL 33749587899, Adapter 33749587901, Proptest 33749587881이 success다.
- 시각 판정: issue2004 p4-p8은 5쪽 모두 flagged 0/5이며 pixel match 78.82516-83.19475%, ink/proxy 19.16494-30.48460%다. p4 배너·인물 적층과 본문 회피, p5-p8 페이지 흐름을 사람이 확인했다.
- 자동 ink/proxy 수치는 글꼴 폭·줄바꿈·raster 차이를 포함하는 보조값임을 명시한다. 시각 비교 방법 정본은 https://github.com/edwardkim/rhwp/blob/573059ee7bd4b74626143723d31d0b74ab0320b8/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment 이다.
- devel 안정 asset: ![PR 6661 issue2004 p4 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/573059ee7bd4b74626143723d31d0b74ab0320b8/mydocs/pr/assets/pr_6659_6664_jeong_sik_integration_20260903/review_6661_issue2004_p4.png)
- #6655는 devel 반영과 push CI 성공 뒤 자동 CLOSED 상태를 확인한다.
