---
kind: review-plan
status: active
---

# JamesPsh HWPX 통합 검토·반영 계획 (PR #3646, #3651)

기준은 `upstream/devel`의 `6d36bafaab0d253781a19e5b17eb51104e403834`다. 작성자
`@JamesPsh`의 열린 HWPX serializer PR 두 건은 개별 merge하지 않고, 이 기준 위
[#3659](https://github.com/edwardkim/rhwp/pull/3659) 하나로 누적 반영한다.

| 순서 | 원 PR | 원 기능 commit | 통합 후보 반영 |
| --- | --- | --- | --- |
| 1 | #3646 | `4b4eed64a` | `30d850618` |
| 2 | #3651 | `1ac0270b1` | `00450ccb1` |

원 head의 `devel` 병합 commit `2be15696b`·`9f1c2addf`은 중복을 피하기 위해 제외했다.
두 기능 patch는 `git range-diff`에서 동등하며 체리픽 conflict는 없다.

## 검증과 범위 경계

- #3646 원 head와 #3651 원 head의 full CI가 각각 success다.
- #3659 code head `00450ccb1`은 lint, Native Skia, default-feature 8 shards, `Build & Test`,
  CodeQL, Canvas visual diff를 모두 통과했다.
- 로컬 전체 Cargo는 원 PR 및 통합 head CI가 확인한 범위를 중복하지 않도록 작업지시에 따라 실행하지
  않았으며, 성공 근거로 적지 않는다.
- 두 변경은 HWPX 속성 보존·스키마 적합성만 다룬다. renderer·layout·typeset 변경이 아니므로 새 PDF
  정합이나 수동 visual sweep을 성공 근거로 주장하지 않는다.
- #3543은 이 통합 PR이 닫는다. #3545는 `dirty="0"` 초기 안내문 잔재를 적재에서 물리 삭제하는
  별도 설계 축이 남아 있으므로 open으로 유지한다.

## 반영 순서

1. archive review·오늘할일만 한 commit으로 #3659 head에 추가한다. source/test/workflow와 기존
   sample·PDF·golden은 이 commit에 섞지 않는다.
2. push 전 변경 파일의 LFS attribute와 `git lfs status`를 확인한다. 비-LFS면
   `GIT_LFS_SKIP_PUSH=1` dry-run과 실제 push를 사용한다.
3. source/local/remote SHA를 대조하고, latest review-only head의 fast-pass와 `CLEAN`·`MERGEABLE`을
   확인한다.
4. #3659을 squash merge한 뒤 merge SHA와 #3543 close, #3545 open 상태를 확인한다.
5. 원 #3646·#3651에는 통합 PR과 merge SHA, 구체적인 검토 근거를 실제 LF body-file comment로 남긴 뒤
   supersede close한다. source fork branch는 삭제하지 않는다.
6. `devel` sync, upstream integration branch·local source refs의 정확한 종료 대상만 post-merge 절차에
   따라 정리한다. 이번 검토에서는 전용 Cargo target을 만들지 않았으므로 제거할 target은 없다.
