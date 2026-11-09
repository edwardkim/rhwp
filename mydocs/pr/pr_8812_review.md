# PR #8812 검토

## 최종 판정

**승인 — 로컬 검증 완료, 원격 통합 승인은 대기 중.** 원 head의 기존 CI 상태와 새 통합 head에 필요한 GitHub CI를 구분한다.

- 원 PR: [#8812](https://github.com/edwardkim/rhwp/pull/8812), dependabot[bot], devel 대상, non-draft.
- 변경: chore(deps-dev): bump vite from 8.3.5 to 8.3.6 in /rhwp-studio in the vite-stack group.
- 고정 원 head: `bb3d70c892bfffb5ee59d7570328ee86186ec5e4`. reviewer edwardkim 지정·API 확인 뒤 상세 조사·fetch를 진행했다.
- 기준 devel: `024d610633ede166ebcbb915f9c90e7a9d0649d8`. 결합 검증 source: `2320351d2325a41d900e91f4e07f559d4a924a7d`. 누적 merge에 충돌이 없고 원 head는 현재 branch의 조상으로 보존됐다.
- 추가 하네스·고지 보정 commit은 `42f82843ec5c957e4e06c414e7f9d0ade7cf052b`다. [실제 보정과 검증 source의 동일성](assets/dependabot-20261109/source-review.md)을 별도로 확인했다.
- 역할: maintainer_general, intake_and_review·local_validation·multi_pr_update_branch·rework_and_exceptions·visual_fixture_evidence·post_merge.

원 commit을 보존한 결합 graph에서 실제 소비 범위의 필수 로컬 검증을 완료했다.

적용 검증은 npm 설치·실제 TypeScript/build·브라우저와 lockfile 전이 감사다. [의존성별 실제 소비·원 CI 원인](assets/dependabot-20261109/dependency-review.md)의 해당 행과 [공통 실행 원장](assets/dependabot-20261109/README.md), [원 head 최신 CI](assets/dependabot-20261109/source-review.md)를 따른다. 공유 검증을 19번 실행한 것으로 집계하지 않는다.

조판 규칙·geometry·pagination 분기 자체를 바꾸는 구현은 없다. 실제 출력 영향이 가능한 의존성은 Native/fresh WASM 각 9쪽 전쪽 TSV·직접 확대 판독과 편람 Native 383쪽에서 검증했다. 새 fixture·golden·절대 좌표 핀·기대값 변경이 없다. Action과 build 도구 갱신은 backend 조판 개선을 주장하지 않는다.

승인 뒤 단일 통합 PR에 원 기여를 보존해 push하고 정확한 head CI·self-review·최신 base merge-tree를 확인한 뒤 merge한다. 원 PR별 완료 상태를 확인하고, 옵션 2의 별도 후속 기록 PR·devel 동기화 뒤 결과 댓글과 이번 전용 작업공간·output·미사용 debug 정리를 수행한다. 원 CI의 실패·취소는 보존하며 승인 없는 재실행·분류 변경은 하지 않는다.
