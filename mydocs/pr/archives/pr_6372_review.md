---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 23:18 KST
pr: 6372
issue: 6371
author: lpaiu-cs
---

# PR #6372 review - rotation storage bit diagnostics

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6372
- 작성자: `lpaiu-cs`
- reviewer request: 미등록. 원격 상태 변경은 사용자 승인 뒤 수행한다.
- 원 PR head: `83257081d7b14a29778ee9a8d485da78b8763463`
- 원 PR 상태: non-draft, `CLEAN`
- 통합 검토 브랜치: `review/lpaiu-cs-6372-6376-20260829`
- 적용 commit: `8a80d7f47`
- 기준: `upstream/devel@2bcf9b261`
- loaded docs: `AGENTS.md`, `mydocs/manual/pr_review_workflow.md`,
  `mydocs/manual/pr_review/intake_and_review.md`,
  `mydocs/manual/pr_review/collaborator_external_pr.md`,
  `mydocs/manual/pr_review/multi_pr_update_branch.md`,
  `mydocs/manual/pr_review/local_validation.md`,
  `mydocs/manual/pr_review/visual_fixture_evidence.md`

## 검토 판단

**수용 권고.** `dump`가 `ShapeComponentAttr.flip` 원본 워드와 `rotate_image`를 드러내지 않아
회전 편집이 저장 비트를 잘못 건드리는지 판정할 수 없던 문제를 직접 해결한다. 그림과 표 셀
그림 경로가 같은 변환 진단 줄을 내도록 한 것도 적절하다.

## 증적과 검증

- 원 PR CI: Full CI, CodeQL, Adapter inter-diff, Proptest roundtrip 모두 성공.
- 원 PR CI B/C/D archive와 shard는 각각 성공했다. 세부 시각은
  `mydocs/pr/assets/pr_6372_6376_validation_20260829.md`에 보관했다.
- `tools/hangul_rotation_oracle/test_oracle.py`: 8 pass
- `rhwp dump samples/ta-pic-001-r.hwp`: 회전 34도/0도 그림 모두 `flip` 및 `rotateImage` 출력 확인
- `hangul_rotation_oracle --survey samples/ta-pic-001-r.hwp`: bit19가 회전 표식이 아님을 확인
- `cargo fmt --all -- --check`: pass
- `git diff --check upstream/devel...HEAD`: pass

공통 검증 증적: `mydocs/pr/assets/pr_6372_6376_validation_20260829.md`

## 시각 증적 판단

이 PR은 진단 출력과 오라클 도구 변경이다. HWP 샘플은 `dump`와 오라클 입력으로만 사용되며,
renderer/layout/paint 사용자 시각 결과를 바꾸지 않는다. 따라서 visual sweep은 필수 증적이 아니다.

## 코멘트 처리 메모

merge 후 원 PR에는 diagnostic surface와 오라클을 수용했고, 이 근거가 #6376의 저장 비트 보존
판단에 사용됐음을 남긴다. 별도 메인터너 보정은 없다.
