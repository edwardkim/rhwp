# planet6897 렌더 PR 4건 통합 처리 계획

## 대상과 적용 순서

| 원 PR | 원 head | 최신 devel 위 적용 commit | 제목 |
| --- | --- | --- | --- |
| #3019 | 786b2842f80a789cd4292a0f467bce1827251449 | 1d08e0a0058db89da416c9e08fb938e30ad7a470 | 가운뎃점 합성 원 반지름 보정 |
| #3084 | 68898247607b64941e82539503c00d7e5e577190 | 723db748705ecc9f0144623f8a41e7ebf3d466ef | pgnp 글자 크기와 장식 공백 교정 |
| #3086 | d2ba1a49851f56a91ba825db25f73dd1949286f1 | c45ad62887126d7baf9875abc960830222dcde4e | 가운뎃점 텍스트 추출 복구 |
| #3129 | e7b9d01e077688ec90464f00d73405e6e099e638 | 43b3ce00d3debf5f9a63889928a72a78ea832066 | 흐름 콘텐츠 body clip 꼬리 보존 |

- 통합 기준은 upstream/devel 1ddfc7da12a63d8ffc9df42e6952a30db25f5e32다.
- 통합 브랜치는 review/planet6897-render-20260724다. contributor 원 commit의 author 정보와 변경 순서를 보존했다.
- 최초 적용에서 #3019와 #3086이 form-002, aift page3 golden에서 겹쳤다. #3019의 작은 가운뎃점 반지름과 #3086의 투명 추출 텍스트를 모두 보존해 해결했다.

## Stage 1. 통합 검토와 검증

- 네 원 PR 모두 devel 대상, maintainer_can_modify=true, 문서 작성 전 참고 CI 성공, maintainer 보류 코멘트 없음으로 분류했다.
- 최신 devel이 검토 도중 전진해 50803cc1e에서 1ddfc7da1으로 바뀌었다. 원 commit 4개를 최신 devel 위로 다시 rebase했고 충돌은 없었다.
- cargo fmt --check, svg snapshot 8/8, issue_1486 6/6, issue_874 1/1, layout test_634 8/8, native-skia cargo check, git diff --check을 통과했다.
- KTX p002 visual sweep 결과와 안정 PNG는 각 review 문서 및 mydocs/pr/assets/pr_3129_planet_ktx_p002_review.png에 기록했다.

## Stage 2. 승인 후 통합 PR 준비

- 작업지시자 승인 후 upstream에 integrate/planet-render-batch-20260724 브랜치로 push하고 devel 대상 통합 PR을 생성한다.
- PR 본문에는 원 PR #3019, #3084, #3086, #3129와 관련 이슈 #2999, #3048, #3085, #3127을 명시한다. 이슈 자동 close 여부는 merge 뒤 GitHub metadata로 재확인한다.
- push 직전 최신 upstream/devel을 다시 fetch한다. 새 devel 커밋이 있으면 이 브랜치를 rebase하고 같은 핵심 검증을 재실행한다.
- 최종 PR diff에는 이 review 문서 4개, 본 계획서, visual asset, 그리고 PR 생성 직전 작성하는 오늘할일을 포함한다.

## Stage 3. CI, merge, 후속 처리

- 통합 PR 최신 head의 required GitHub Actions가 모두 완료될 때까지 모니터링한다. 실패하면 원인을 분리해 보정 범위와 재실행 여부를 작업지시자에게 보고한다.
- 모든 required check 성공과 작업지시자 merge 승인을 받은 뒤 squash merge를 수행한다. 새 merge로 behind가 되면 update branch 또는 최신 devel rebase 후 CI를 다시 확인한다.
- merge 뒤 관련 이슈 자동 close 여부와 원 PR 4개의 상태를 확인한다. 원 PR close 안내 및 감사 코멘트는 문안 제시 후 별도 승인받아 처리한다.
- devel 동기화 후 통합 remote 브랜치와 로컬 review 브랜치 및 worktree를 정리한다. main, devel, ios/devel, 진행 중인 다른 작업 브랜치는 정리 대상이 아니다.
