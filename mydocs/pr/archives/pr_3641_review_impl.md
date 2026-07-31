---
kind: review-plan
status: active
---

# NacreousCloud 통합 검토·반영 계획 (PR #3641, #3642)

기준은 `upstream/devel`의 `44d046bf7da12226c383f2f1c297b97b770c3bcd`다. 작성자
`@NacreousCloud`의 layout PR 두 건은 개별 merge하지 않고, 이 기준 위
[#3657](https://github.com/edwardkim/rhwp/pull/3657) 하나로 누적 반영한다.

| 순서 | 원 PR | 원 head | 통합 후보 반영 |
| --- | --- | --- | --- |
| 1 | #3641 | `1357ea41` | `02c176c18` · `59ff9ab9b` · `d06e14516` |
| 2 | #3642 | `100d4de3` | `2be4fa07d` · `a948d8b29` · `27786a712` |
| 3 | 통합 보정 | — | `f45799a46` |

원 PR의 `devel` 병합 commit은 중복을 피하기 위해 제외했다. 기능 commit은 `git range-diff`에서
patch 동등이고 체리픽 conflict는 없다. `f45799a46`은 #3641 fixture 탐색을 특정 쪽 번호가 아닌
전체 순차 조판으로 바꿔 #3642의 pagination 변화에도 회귀가 안정적으로 같은 셀을 찾게 한다.

## 검증과 범위 경계

- #3641 원 head `1357ea41`, #3642 원 head `100d4de3`, 통합 #3657 code head `f45799a46`의
  GitHub Actions full CI가 각각 success다.
- #3657은 lint, Native Skia, default-feature 8 shards, `Build & Test`, CodeQL, Canvas visual diff를
  통과했다. 로컬 release-test는 작업지시에 따라 중복 실행하지 않았으며 성공 근거로 적지 않는다.
- #3641은 한컴 2024 PDF 대조 p37의 3-way/OVL asset을 보존한다. 낮은 ink match는 font·조판 차이가
  지배하므로 PDF 전체 정합 주장으로 확대하지 않는다.
- #3642는 기준 PDF가 없는 두 fixture에서 render-tree marker와 p3→p4·p59→p60 연속 panel로
  행 정체성 회복을 판정한다.
- 마지막 꼬리 문단 종료 결함은 #3642 범위가 아니다. ignored 회귀를 보존하고 [#3658](https://github.com/edwardkim/rhwp/issues/3658)로 분리한다.

## 반영 순서

1. archive review·대표 PNG·오늘할일만 한 commit으로 #3657 head에 추가한다. source/test/workflow,
   기존 sample/PDF, golden/baseline은 이 commit에 섞지 않는다.
2. push 전 변경 파일의 LFS attribute와 `git lfs status`를 확인한다. 비-LFS면
   `GIT_LFS_SKIP_PUSH=1` dry-run과 실제 push를 사용한다.
3. 현재 base를 포함한 green code candidate `f45799a46`과 latest review-only head의 preflight,
   `Build & Test` aggregate, `CLEAN`·`MERGEABLE`을 확인한다.
4. #3657을 squash merge한 뒤 merge SHA와 #3593/#3595 close 상태를 확인한다. #3658은 open으로 유지한다.
5. 원 #3641·#3642에는 통합 PR과 merge SHA, 구체적인 검토 근거를 실제 LF body-file comment로 남긴 뒤
   supersede close한다. source fork branch는 삭제하지 않는다.
6. `devel` sync, upstream integration branch·local review refs·전용 target의 정확한 종료 대상만
   post-merge 절차에 따라 정리한다.
