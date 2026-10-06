---
kind: review-plan
status: active
---

# lpaiu-cs·planet6897 통합 검토·반영 계획 (PR #3665, #3666)

기준은 `upstream/devel`의 `3b28ab597cb9c45c2d08c37fa16a4c9377db7d67`다. 두 contributor PR은
개별 merge하지 않고 [#3671](https://github.com/edwardkim/rhwp/pull/3671) 하나에 누적한다. 원 PR별
판정은 [#3665 review](pr_3665_review.md), [#3666 review](pr_3666_review.md)에 분리해 남긴다.

| 순서 | 원 PR | source 기능 commit | 통합 반영 | 작성자 보존 |
| --- | --- | --- | --- | --- |
| 1 | #3665 | `24ad696b…` | `9dac26c0e…` | `lpaiu-cs` |
| 2 | #3666 | `98c63b1ab…` | `ab12d4f29…` | `planet6897` |
| 3 | #3666 보정 | — | `0d1839ae9…` | maintainer |
| 4 | #3665 P1 후속 | `06d03b110…` | `3a1db3234…` | `lpaiu-cs` |

`-x` trailer로 source와 integration commit을 추적한다. #3665의 `dc204635`·`1d68f1e`·`f4be36a`
및 #3666의 `e4ef27d`·`5f4d7d9` 같은 source `devel` merge commit은 이미 기준에 포함된 변경이므로
제외했다. 기능 commit 적용에 conflict는 없었다.

## 수용 계약과 범위 경계

- #3665는 narrow query의 ancestor clip 동등성과 document-bound object URL lifecycle을 수용한다.
  #3315는 Track 3·4 umbrella이므로 open으로 유지한다.
- #3666은 split-cell nested-table 뒤 문단의 out-of-page glyph를 focused 30px 계약 안으로 제한한다.
  #3637은 이미 closed이며 다시 close를 요청하지 않는다.
- `0d1839ae9`은 source PR의 의도를 넓히지 않고 fragment origin과 physical content bottom의 실제 경계만
  바로잡는다. 현재 PDF 12쪽↔rhwp 13쪽, p2·p4·p12 visual candidate는 별도 렌더 차이이며, 이번 fix의
  PDF 전체 정합 성공으로 숨기지 않는다.

## 검증과 후속 순서

1. code candidate `3a1db3234`의 full CI는 성공했다. lint, frontend gates, Native Skia, test archive,
   default-feature 8 shards, `Build & Test`, CodeQL, Canvas visual diff를 포함한다.
2. 이 archive review 두 건, 기준 PDF, p10/p12 review PNG, 오늘할일만 하나의 single-parent
   review-only commit으로 #3671 head에 추가한다. source/test/workflow/golden/baseline 또는 기존 fixture는
   이 tail에 섞지 않는다.
3. staged 경로별 LFS attribute·`git lfs status`를 먼저 판독한다. 비-LFS이면 `GIT_LFS_SKIP_PUSH=1`
   dry-run 뒤 같은 branch로 push한다.
4. `3a1db3234`의 성공 `Build & Test`를 candidate로 삼아 최신 review-only head의 CI preflight와
   aggregate가 success인지 확인한다. fast-pass가 fallback하면 full CI 종료까지 기다린다.
5. latest head가 `CLEAN`·`MERGEABLE`이고 required check가 성공하면, 작업지시자의 기존 자동 승인 범위에
   따라 #3671 하나만 squash merge한다.
6. merge SHA를 확인한 뒤 원 #3665·#3666에는 통합 PR, 저자별 구체적 검토 근거, merge SHA를 실제 줄바꿈
   body-file comment로 남긴다. 그 뒤 source fork branch는 보존한 채 원 PR을 supersede close한다.
7. `devel` sync, #3315 open/#3637 closed 상태 확인, upstream integration branch·`local/pr3665*`·
   `local/pr3666` refs·전용 target의 정확한 종료 대상만 post-merge 절차에 따라 정리한다.

오늘 작업 상태와 merge 전 조건은 [2026년 8월 1일 오늘할일](../../orders/archives/20260801.md)에 함께 기록한다.
