---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 23:18 KST
pr: 6376
issue: 6373
author: lpaiu-cs
---

# PR #6376 review - preserve picture rotation storage bits

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6376
- 작성자: `lpaiu-cs`
- reviewer request: 미등록. 원격 상태 변경은 사용자 승인 뒤 수행한다.
- 원 PR head: `11e50cf44ccc1504fce849d8ae8ca05726e0d3eb`
- 원 PR 상태: non-draft, `CLEAN`
- 통합 검토 브랜치: `review/lpaiu-cs-6372-6376-20260829`
- 적용 commit: `296f579f2`
- 기준: `upstream/devel@2bcf9b261`
- 선행 근거: #6372의 `tools/hangul_rotation_oracle/EVIDENCE.md`
- loaded docs: `AGENTS.md`, `mydocs/manual/pr_review_workflow.md`,
  `mydocs/manual/pr_review/intake_and_review.md`,
  `mydocs/manual/pr_review/collaborator_external_pr.md`,
  `mydocs/manual/pr_review/multi_pr_update_branch.md`,
  `mydocs/manual/pr_review/local_validation.md`,
  `mydocs/manual/pr_review/visual_fixture_evidence.md`

## 검토 판단

**수용 권고.** 그림 회전 편집이 각도 변경을 이유로 `rotate_image`와 `flip` bit19를 강제로 세우던
동작은 한컴 저장 관례와 맞지 않는다. #6372의 오라클 증거상 bit19는 회전 상태의 함수가 아니므로,
회전 편집은 파싱된 저장 비트를 보존하는 쪽이 맞다.

## 증적과 검증

- 원 PR CI: Full CI, CodeQL, Adapter inter-diff, Proptest roundtrip 모두 성공.
- 원 PR CI B/C/D archive와 shard는 각각 성공했다. 세부 시각은
  `mydocs/pr/assets/pr_6372_6376_validation_20260829.md`에 보관했다.
- `node scripts/run-rust-test.mjs issue_6373_picture_rotation_storage_bits -- --cargo-profile release-test --target-dir target/pr-review`:
  2 pass
- `node scripts/rust-test-suite-manifest.mjs --prepare && node scripts/rust-test-suite-manifest.mjs --check`:
  pass
- #6372 진단 표면으로 `samples/ta-pic-001-r.hwp`의 회전 34도/0도 그림 모두 bit19가 켜져 있음을 확인했다.
- `cargo fmt --all -- --check`: pass
- `git diff --check upstream/devel...HEAD`: pass

공통 검증 증적: `mydocs/pr/assets/pr_6372_6376_validation_20260829.md`

## 시각 증적 판단

이 PR의 감시선은 렌더링 픽셀이 아니라 저장 속성 보존이다. renderer/layout/paint 경로를 직접
바꾸지 않으므로 visual sweep은 필수 증적이 아니다. 회전 편집 후 저장·재파싱 보존은
`issue_6373_picture_rotation_storage_bits`에서 고정했다.

## 코멘트 처리 메모

merge 후 원 PR에는 #6372의 실측 근거에 따라 `rotate_image`와 `flip` bit19를 회전 편집에서
건드리지 않는 변경을 수용했다고 남긴다. 별도 메인터너 보정은 없다.
