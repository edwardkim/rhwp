# seo-rii 비 draft PR 20개 통합 검토 계획

## 범위와 승인

사용자 지시: 열린 seo-rii PR 중 draft가 아닌 것을 모두 체리픽 통합·검토하고 필요하면 메인터너 보정한다.
모든 필수 검증을 마친 뒤 통합 PR, CI, 원 PR별 기록, 머지 및 post_merge 후속 처리를 수행한다.
#536은 기존 기여자의 멀티 렌더러 추적 이슈다. 이 작업만으로 draft P63/P65나 미지원 플랫폼까지 전체 해결했다고 닫지 않는다.

- base route: collaborator_external_pr.md (9.1.1 체리픽 통합).
- modifiers: intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, rework_and_exceptions, review_only_fast_pass, post_merge.
- 관련 정본·dev_environment_guide·시각 governance/guide·backend 소비 검증을 읽었다.
- 브랜치 `review/seo-rii-20261017`, 격리 worktree `/home/tsjang/rhwp-seo-rii-20261017`.
- primary 사용자 Studio JS는 그대로 보존한다. 모든 Cargo는 `/home/tsjang/rhwp/target/pr-review`를 순차 재사용한다.
- 원본 fork와 commit 저자를 보존한다. 원 PR 번호별 review/implementation 기록을 같은 통합 PR에 넣는다.
- draft #7930·#7932는 적용하지 않는다. #7931 최종 head는 P62 대비 접근성 7파일이며 Qt 창 코드가 제거돼 있다.
  상속된 Qt 5개 commit은 제외하고, 후속 Qt 제거 commit은 적용 결과의 비중복 여부를 기록한다.

## 실행 단계

1. 원 PR metadata/head/reviewer, source 의존 관계, 원본 입력·기준 PDF를 고정한다.
2. 기능·test·문서 commit을 오래된 순서로 중복 없이 cherry-pick한다. devel merge commit은 제외하며 그 충돌 해소 의미는 최종 source delta와 대조한다.
3. 원 PR별 구현 주장·적용 경로·독립 기대값·반례를 검토한다. 충돌·실제 결함은 통합 branch의 별도 보정으로 기록한다.
4. 작은 focused와 영향 출력 확인 뒤 Rust lint 묶음·전체 nextest·Native Skia 3종·workspace/native FFI·frontend·fresh WASM·실물 backend 검증을 완료한다.
5. 같은 원본/Print PDF의 고정 devel 및 통합 head Native/fresh WASM 전쪽 TSV와 대표 review/overlay를 직접 비교한다. 90% 미달·쪽수 불일치·0.01%p 하락은 보정 후 다시 검증한다. 미실행을 통과로 표시하지 않는다.
6. PR 전에 upstream head 변화를 확인한다. 코드 기준이 바뀌면 필요한 rebase와 영향을 재검증한다.
7. 검증 완료 후 통합 PR을 생성하고 실제 CI를 확인한다. 같은 PR에 원 PR별 최종 review·필요한 오늘할일을 문서 trailing으로 보완하고 최신 check를 확인한다.
8. normal merge 후 duration·devel sync·원 PR별 credit/comment/close·증적 보존·이번 branch/worktree 및 output 전체 정리를 완료한다. draft/fork/shared cache/사용자 변경은 보존한다.

## 고정 대상

기준 devel: `fe5725dc384689b45d60ef8ea4c83138bc2701d8`. 아래 head는 접수 시점 snapshot이며 PR 전에 다시 조회한다.

| PR | source head | 고유 비 merge commits |
| --- | --- | ---: |
| [#7660](https://github.com/edwardkim/rhwp/pull/7660) | `860f8c22ddd892e74030fb13bae8c8af07ef2aff` | 15 |
| [#7890](https://github.com/edwardkim/rhwp/pull/7890) | `79942a3807d4066dd7cb44abda7697c19ffb614e` | 24 |
| [#7891](https://github.com/edwardkim/rhwp/pull/7891) | `8273ece22860b5793767b9888dd320b3bdc414b8` | 29 |
| [#7892](https://github.com/edwardkim/rhwp/pull/7892) | `6abf22be0cb0fa10f0a5b9dfe57bdd2539e5df4f` | 30 |
| [#7893](https://github.com/edwardkim/rhwp/pull/7893) | `01254c417ed0cf0333015d0d57099fd9207d6fbc` | 5 |
| [#7894](https://github.com/edwardkim/rhwp/pull/7894) | `f8e30c30e25ce9c2429cb811d58f768e82880f1e` | 2 |
| [#7895](https://github.com/edwardkim/rhwp/pull/7895) | `930d2e96f09eccc2762a43a24740d9ca204cb1af` | 2 |
| [#7914](https://github.com/edwardkim/rhwp/pull/7914) | `509f53258977ea2da04a480510b91c340f4e5e2a` | 2 |
| [#7915](https://github.com/edwardkim/rhwp/pull/7915) | `e978db0dbe5922e22e1507124b4607b6bd60a61b` | 3 |
| [#7916](https://github.com/edwardkim/rhwp/pull/7916) | `94a7f2da7d712b0f20abc37c1981bc32d55dfd71` | 3 |
| [#7917](https://github.com/edwardkim/rhwp/pull/7917) | `a0ecf253a78c2ccf106c91e9cba19115c88317f2` | 3 |
| [#7918](https://github.com/edwardkim/rhwp/pull/7918) | `91a46f90f987cdec23a78fdf9c7463a77db38ed2` | 4 |
| [#7919](https://github.com/edwardkim/rhwp/pull/7919) | `25c56fde48d4afc500782f296cb120d5b42b139f` | 4 |
| [#7920](https://github.com/edwardkim/rhwp/pull/7920) | `4d8c9014ba12db6b7c22806f364d16b0a81deaa3` | 6 |
| [#7924](https://github.com/edwardkim/rhwp/pull/7924) | `1fce487fc80401c17f43599c02c3e0a2ae01057c` | 3 |
| [#7925](https://github.com/edwardkim/rhwp/pull/7925) | `c90789077afe74e595abaa672db0afd591e28c1b` | 5 |
| [#7927](https://github.com/edwardkim/rhwp/pull/7927) | `73f2f180608f3088bb7a1fd32c0595b5d8fb6797` | 5 |
| [#7928](https://github.com/edwardkim/rhwp/pull/7928) | `e2ddab5377f828dda0260cf7dc10d2b41d9271d7` | 9 |
| [#7929](https://github.com/edwardkim/rhwp/pull/7929) | `556b6b706dbb69d070ccbb76f58a2cdce6ada3a6` | 1 |
| [#7931](https://github.com/edwardkim/rhwp/pull/7931) | `b4003f89fceb8df6e64652dfdf9ec1cfcd642cf1` | 5 |
