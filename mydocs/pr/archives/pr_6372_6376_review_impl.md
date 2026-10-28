---
kind: pr-review-impl
status: local-validated
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 23:18 KST
author: lpaiu-cs
branch: review/lpaiu-cs-6372-6376-20260829
prs: [6372, 6376]
---

# lpaiu-cs #6372 #6376 통합 검토 - 2026-08-29

## 범위

포함:

- #6372 `83257081d7b14a29778ee9a8d485da78b8763463`
- #6376 `11e50cf44ccc1504fce849d8ae8ca05726e0d3eb`

제외:

- #5953: draft 상태라 제외했다.

## 적용

- review branch: `review/lpaiu-cs-6372-6376-20260829`
- base: `upstream/devel@2bcf9b261`
- code candidate head before review-document commit: `296f579f2`
- #6372: `8a80d7f47`
- #6376: `296f579f2`
- conflict: 없음
- 메인터너 보정: 없음

## 검토 요약

- #6372는 `dump`의 그림/표 셀 그림 변환 진단 표면을 확장하고, 한컴 저장 관례를 재는
  `tools/hangul_rotation_oracle`을 추가한다.
- #6376은 #6372의 실측 근거를 바탕으로 그림 회전 편집에서 `rotate_image`와 `flip` bit19를
  강제로 세우지 않고 원본 저장 비트를 보존한다.
- 두 PR은 순서 의존성이 있다. #6376의 판단 근거와 주석이 #6372의 오라클/증적을 참조한다.

## 검증

상세 증적: `mydocs/pr/assets/pr_6372_6376_validation_20260829.md`

- 원 PR #6372, #6376 모두 non-draft이고 GitHub CI가 green이다.
- 원 PR 기준 B/C/D archive와 `test-archive-*-shard-1`은 모두 success다.
- `tools/hangul_rotation_oracle/test_oracle.py`: 8 pass
- Rust suite manifest prepare/check: pass
- `issue_6373_picture_rotation_storage_bits`: 2 pass
- `rhwp dump samples/ta-pic-001-r.hwp`에서 `flip`과 `rotateImage` 출력 확인
- `hangul_rotation_oracle --survey samples/ta-pic-001-r.hwp`로 bit19가 회전 표식이 아님을 확인
- `cargo fmt --all -- --check`: pass
- `git diff --check upstream/devel...HEAD`: pass

## 시각 증적 판단

HWP 샘플이 언급되지만, 이번 변경은 렌더러 출력 비교가 아니라 `dump` 진단과 저장 속성 보존
계약이다. visual sweep은 필수 대상이 아니다. 통합 PR을 만들면 code change PR이므로 CI의 B/C/D
archive와 shard가 의도대로 실행되는지 별도로 구분해 확인한다.

## 판단

- #6372: 수용 권고.
- #6376: 수용 권고.

통합 PR은 사용자의 별도 `pr` 지시 후 생성한다. PR 생성 전에는 최신 `upstream/devel`, 원 PR head,
mergeability, CI 상태를 다시 확인한다.

## merge 후 코멘트 메모

- #6372: 진단 표면과 오라클을 수용했고, 이 결과가 #6376의 회전 저장 비트 보존 판단에 사용됐음을 설명한다.
- #6376: #6372 실측 기준으로 `rotate_image`/`flip` bit19를 회전 편집에서 강제로 세우지 않는 변경을 수용했다고 설명한다.
