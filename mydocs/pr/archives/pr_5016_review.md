# PR #5016 검토 - `edit delete-text`

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#5016](https://github.com/edwardkim/rhwp/pull/5016) |
| 작성자 | `kevin9327` |
| source base / head | `devel` / `1c4eda555447cae584ec4a2e27c2787e79f174c8` |
| 검토 브랜치 | `review/kevin9327-unincluded-5175-20260817` |
| 실제 적용 source commit | `4fded47ed2025e161064dccc9f150933fce84d53` |
| 누적 commit | `5a45e608044fc3cb8786f4224665b27747a65637` |
| 파생 산출물 보정 | `5f7caf48a` (공통) |
| source PR 상태 | `OPEN`, non-draft |

## 검토 경로

```text
base route: maintainer_general.md
modifiers: intake_and_review.md, local_validation.md, multi_pr_update_branch.md
loaded documents: pr_review_workflow.md, pr_review/README.md, maintainer_general.md, intake_and_review.md, local_validation.md, multi_pr_update_branch.md
current head: 1c4eda555447cae584ec4a2e27c2787e79f174c8 (작성 시점 참고값)
```

## 검토 범위

문단 좌표에서 지정 글자 수를 삭제하는 고유 commit만 적용했다. `hwp_delete_text`는 `path`와 1 이상인 `count`를 필수로 선언하며 section·paragraph·offset의 기본값은 CLI와 같다. 원본 계약은 실제 글자가 있는 문단을 선택하도록 후속 보정된 fixture 경로를 사용한다.

## 검토 결과

차단·수정 필요 결함을 발견하지 못했다. 0 count, 숫자 형식 오류, 중복 입력 파일을 CLI에서 usage error로 구분하며, schema의 numeric boundary도 같은 제약을 표현한다.

## 누적 검증

전체 integration 회귀가 `6,643 passed, 38 skipped`로 통과했다. generated suite/manifest는 #5177 정책에 따라 검증 뒤 복원했다.

## 권고

누적 통합 범위에서 승인한다.
