---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4734·#4735 통합 검토 구현 기록

## 적용 기준

`upstream/devel@b5c14346d0eba652764111764ae77cb959006af4` 위의 사용자 가시성 branch
`review/planet6897-20260813`에서 오래된 #4734를 먼저, #4735를 다음으로 누적했다. 별도 worktree를
만들지 않아 VS Code와 현재 Git graph에서 적용 순서를 그대로 확인할 수 있다.

초기 검증 뒤 `upstream/devel`이 `e550a270f`에서 `b5c14346d`로 전진했으나, 사이 변경은 오늘 할 일,
archive review, 로드맵 문서뿐이었다. 통합 branch의 여섯 자체 commit을 새 base 위로 충돌 없이 rebase했고,
Rust source, test, fixture, workflow가 변하지 않아 완료한 전체 회귀 결과를 그대로 유지했다.

| 원 PR | 최신 source head | 통합 branch 반영 | 충돌 |
| --- | --- | --- | --- |
| #4734 | `9fc612ec839d3d21ff59640b305c200cfe8fcf59` | `4b11f7e0c`, `70087f9fb` | 없음 |
| #4735 | `3c995998e6dd1b759cc2364ddcb6e03457e26fe7` | `75dce4697`, `e33e07061`, `7a345db90` | 없음 |

검토 도중 #4735가 `3db39c931`에서 최신 세 commit head로 갱신됐다. 이미 수행 중이던 전체 nextest는
종료 결과를 수집했고, 최신 source의 실제 추가분이 IR baseline 한 행과 stage 기록뿐임을 분리 확인했다.
로컬에 임시로 넣었던 동일 baseline 행은 제거한 뒤 `92ddf99ee`, `3c995998e`를 순서대로 cherry-pick했다.
그러므로 contributor history를 rewrite하거나 원 PR source branch에 force-push하지 않았다.

## 메인터너 보정 판단

#4735의 최초 원격 실패는 테스트가 검출한 제품 코드 결함이 아니라 새 fixture를 전수 IR sweep에
등록하면서 필요한 정상화 baseline을 빠뜨린 경우였다. `list_header_width_ref` 124건은 HWP5 record
재생성에서 발생하는 기존 계열이고, 새 행은 해당 fixture·경로·완전 건수 하나만 허용한다. 다른 sample의
baseline을 재생성하거나 줄이지 않았으므로 기존 회귀를 숨기지 않는다.

최신 source head는 이 최소 baseline 보정과 stage 사유를 포함한다. 통합 branch는 그 head의 fixture,
test, baseline, stage 파일과 동일함을 다시 비교했다.

## 검증 순서와 결과

1. #4734의 hmtx 폭 계약 1건과 SVG snapshot 8건을 통과시켰다.
2. #4734 HWPX 두 문서의 한컴 2022 PDF 대조를 실행했다. 두 페이지 모두 visual sweep 자동 후보는 0건이며,
   대표 review PNG는 `mydocs/pr/assets/`에 보존했다.
3. #4735의 저장 조각 좌표 계약을 통과시켰고, `dump-extents`로 p3 `x=679.7`, `w=38.4`를 재확인했다.
4. 최신 baseline으로 IR field sweep 전체 823 sample을 실행해 217.02초에 통과시켰다.
5. 누적 후보에서 release-test nextest 전체를 실행해 5,930 passed / 37 skipped / 7 slow,
   486.981초 결과를 얻었다.
6. 마지막으로 두 원 PR head를 fetch해 source 파일 내용과 통합 branch 파일 내용이 각각 같은지 확인했고,
   `git diff --check`를 통과시켰다.

통합 PR은 이 원 PR 번호별 archive review와 필요한 asset을 같은 branch에 포함한다. 통합 PR 번호를 위한
별도 review 문서나 docs-only PR은 만들지 않는다. 원 PR close, 감사 comment, 관련 issue close는 통합 PR
merge와 작업지시자 승인 뒤 `post_merge.md` 절차로 처리한다.
