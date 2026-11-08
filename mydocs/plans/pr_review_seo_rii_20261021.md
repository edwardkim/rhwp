# seo-rii P66–P68 묶음 검토

## 범위와 경로

2026-10-21 사용자의 seo-rii PR 묶음 처리 지시를 수행한다. 후속 지시에 따라 Draft는
제외한다. 대상은 #8088 → #8089 → #8097이며 Draft #7932는 변경·적용하지 않는다.
#536 전체 완료나 미지원 OS·하드웨어 GPU의 검증을 주장하지 않는다.

- 기본 경로: `collaborator_external_pr.md` 9.1.1의 최신 devel 기반 체리픽 통합.
- 보조: intake_and_review, local_validation, multi_pr_update_branch,
  visual_fixture_evidence, rework_and_exceptions, review_only_fast_pass, post_merge.
- reviewer: 각 원 PR의 `edwardkim` 지정과 requested_reviewers API 확인을 완료했다.
- 기본 작업공간의 `review/seo-rii-20261021`에서 검토한다. 공유 Cargo target은
  `/home/edward/mygithub/rhwp/target/pr-review`이며 Cargo 작업은 순차 실행한다.
- 기존 사용자 서버·작업공간·공용 output 자료와 contributor fork는 보존한다.

## 고정 신원

| PR | 원 head | 고유 범위 |
| --- | --- | --- |
| #8088 P66 | `ef613b4f453955358a329580624f371ba9642482` | 공통 영역 plan·예산 |
| #8089 P67 | `be6c8558fbf98e378e8e3c846a6d0f39a88d7152` | immutable page·세션 cache |
| #8097 P68 | `e2c28d8d8eeb75aa73872145b9d076428d832c80` | borrowed host surface·SVG 회전 보정 |

base는 `f45878c7caafb43ca1fd017ddd93484a0acc6ad2`다. 네 기능/보정 commit을 원 저자를
보존해 한 번씩 적용했다. 통합 code head는 `0b1c97075c0d2f7edb90dba781f9f7585bcfc627`,
tree는 `fd971bc26bd0961b6123cadcaf61108057f0986a`이며 원 P68 head와 최신 base의
충돌 없는 자동 merge-tree와 같다. reviewer의 source·test 보정은 아직 없다.

원 PR의 최신 check는 모두 완료됐고 실패가 없다. P68의 정확한 원 head Full CI는
32 success·5 skipped다. 과거 본문에 남은 pending 상태 대신 실제 API 결과를 사용한다.
원 head CI와 통합 후보의 새 CI를 구별한다. 통합 PR의 실제 CI 결과는 아래에 기록했다.

## 검토와 검증

1. P66의 geometry → 공통 plan → working/output 예산 → 실제 copy, P67의 세션 key·
   무효화·image 수명 → warm output, P68의 host CTM·clip·strict replay·상태 복원을 대조한다.
2. 동일 원 code의 녹색 CI는 중복 광범위 회귀 근거로 재사용한다. source 보정이나 충돌 해소가
   생기면 해당 경계를 다시 판정한다. 최종 후보의 merge 가능 상태·required CI는 별도 조건이다.
3. 공통 target에서 새 계약과 기존 exact crop의 집중 실행을 한 번 수행한다. 실제 문서로
   cold/warm 영역·borrowed surface와 full replay를 비교하고 성능 측정의 범위를 명시한다.
4. 실제 Native/fresh WASM과 독립 Print의 전쪽 TSV·대표 PNG를 확인한다. Native의 변경 API
   소비와 WASM의 영향 없는 cfg 경로를 구별한다. 기존 producer 재사용은 source/build 동일성
   근거를 연결하며 새 빌드라고 표현하지 않는다. base 대비 Decimal 하락0.1%p gate를 적용한다.
5. PR별 최종 판정·실제 검사·수용 범위·남은 조건을 원 PR 번호의 archive review에 기록한다.
   공통 증거는 공유하되 원 PR별 판정은 분리한다. today 기록은 초기 조사 중 만들지 않는다.
6. 통합 후보의 필수 검증을 완료한 뒤 원격 조치 대상을 구체적으로 보고한다. push·통합 PR·
   GitHub review/comment·원 PR close·merge는 해당 범위의 사용자 승인 뒤 수행한다.

실행 로그·원시 API·중간 출력은 ignored `output/pr-review/seo-rii-20261021/`에 보존한다.
필수 시각 증거와 입력은 최종 review가 참조할 커밋 경로에 보존한다.

## 현재 검토 결과

원 PR3개는 모두 명시한 통합 code head 기준 **승인**이다.
[P66 리뷰](../pr/archives/pr_8088_review.md), [P67 리뷰](../pr/archives/pr_8089_review.md),
[P68 리뷰](../pr/archives/pr_8097_review.md)에 원 head·의존·실제 소비·판정과 남은 조건을 분리했다.

공통 집중38 PASS, 실물4개 ×3배율의 prepared/host/full pixel 일치, moving-window cache
off/on9쌍과 실제 software Vulkan probe가 통과했다. Native base/후보 RGB가4개 모두 같고,
전쪽 TSV의 Native/WASM 최저99.63470%/99.29125%, base 대비 최대 하락0.00000%p다.
공통 [증거·명령·한계](../pr/assets/issue_536_seo_rii_20261021/README.md)를 연결한다.

## 통합 PR의 실제 CI와 승인

[통합 PR #8135](https://github.com/edwardkim/rhwp/pull/8135)의 정확한 후보
`c1fff8ce3d5cb1b048a53f04fda16942175760c7`에서 [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39613689693), [CI](https://github.com/edwardkim/rhwp/actions/runs/39613689636), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39613689798), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39613689949), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39613689632)가 모두 실제 성공했다.
CI preflight는 `fast_pass=false reason=no-green-build-candidate`였으며,
원 fork CI를 통합 PR CI로 대신하지 않고 새 source branch의 Full CI를 완료했다.
마지막 완료 이벤트의 정책 audit는 GitHub API가 Archive B를 늦게 반영해 pending이었다.
기존 테스트를 재실행하지 않고 [정책 집계 job](https://github.com/edwardkim/rhwp/actions/runs/39617769583)만
재실행해 최신 성공 상태로 갱신했다.
사용자는 push·Open PR 생성·CI 성공 후 병합·원 PR3개 종료와 후속 comment를 승인했다.
이 기록 뒤 변경은 허용된 mydocs 문서뿐이다. 최신 trailing head의 실제 fast-pass·required
check와 최신 base의 merge-tree를 확인한 뒤 승인된 병합·후속 처리를 수행한다.

통합 merge·원 PR close·후속 comment는 아직 수행하지 않았다. Draft #7932와 #536은 유지한다.
후속 정리에서 이번 임시 branch는 삭제하고 사용자 worktree·공유 target은 보존한다.
루트 output은 다른 작업의 release/POC·공유 글꼴 자료 등이 섞여 있어 현재4.1GiB의
전체 삭제는 미완료다. 소유·필요 여부가 불명확한 자료를 임의로 제거하지 않는다.
