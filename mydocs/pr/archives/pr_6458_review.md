---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6458
issue: 6040
author: postmelee
---

# PR #6458 review - 자동 쪽 배치 열 선택과 점유 중앙 정렬

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `pr_review/visual_fixture_evidence.md`,
  `pr_review/review_only_fast_pass.md`, `codex/docs_and_git_workflow.md`
- 작성자·self-review: `postmelee`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.

## metadata와 범위

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#6458](https://github.com/edwardkim/rhwp/pull/6458) |
| 관련 issue | [#6040](https://github.com/edwardkim/rhwp/issues/6040) |
| base / head | `devel` / `codex/issue-6040-zoom-topology` |
| 제출 기준 devel | `7592e9c99a1ca7c13604a33a9a22eb141b04973d` |
| code candidate | `afb33b50edfcef7cc59e3169c294c35137bad7f2` |
| 규모 | 12 files, `+695/-21`, 5 commits |
| 원격 상태 | Draft, `MERGEABLE`, `mergeStateStatus=BLOCKED`; code candidate required CI 성공 |

PR 생성 뒤 `upstream/devel`은 `d3b40a3d7c3ecb5d0f014ce604b99fda17b2bd9b`까지 24 commits
전진했다. 작성 시점의 최신 base와 code candidate는 `git merge-tree --write-tree upstream/devel HEAD`에서
충돌 없이 tree `d7c220a889846ebe7b0997fc17eee2e4f1d7f1a4`를 만들었다. review-only 기록만을 위해 source를
merge하거나 rebase하지 않으며, merge 전 최신 base와 mergeability를 다시 확인한다.

이 PR은 GitHub native stack의 bottom PR이다. #6041은 이 source branch를 base로, #6042는 #6041
source branch를 base로 추가한다. 자동 쪽 배치가 다음 계약을 지키도록 고친다.

- 고정 50% gate 대신 표시 페이지 폭·쪽 간격·편집 viewport 폭으로 자동 열 수를 고른다.
- 열 수를 실제 페이지 수로 제한하고, 빈 grid slot이 아니라 실제 점유 페이지 묶음을 가운데에 둔다.
- 155% A4 수평 눈금자에서 종이 밖으로 넘는 `21` 라벨만 숨기고 끝 tick과 종이 경계는 유지한다.
- 기존 `VirtualScroll` 기반 줌·눈금자·캐럿 좌표 경로는 유지한다.

PR 본문은 `Refs #6040`을 사용한다. #6040 본문의 CSS zoom preview, topology commit, 점진적 Canvas
교체 성능 수용 기준은 이 code candidate에 포함되지 않으므로 이 PR로 issue를 닫지 않는다.

## self-review와 범위 결정

**현재 Stage 1·1.1 범위는 조건부 수용 권고한다.** 표시 geometry에서 열 수를 계산하고 실제 점유
묶음을 별도로 가운데에 두는 경계는 2열 누락과 저배율 좌측 쏠림을 직접 해결한다. 눈금자 라벨은
canvas text bounds가 종이 안에 완전히 들어오는 경우에만 그려 끝 tick·경계 계산을 바꾸지 않는다.

초기 계획의 Canvas 전용 CSS zoom preview와 점진적 Canvas 교체 Stage 2·3도 로컬에서 구현했으나,
기존 `VirtualScroll` 좌표와 preview 좌표가 병행되면서 좌우·상하 jump, 눈금자 반영 지연, 캐럿·선택·
hit-test 회귀 위험이 확인됐다. 해당 구현은 안전 branch에만 보존하고 이 PR에서 제거했다. 사용자는
Stage 1 기준 재구성과 눈금자 끝 라벨 처리를 직접 확인해 현재 범위로 결정했다.

별도 `pr_6458_review_impl.md`는 만들지 않는다. maintainer 보정이나 conflict 해결이 없고, 구현·rollback과
후속 stack 순서는 [수행 계획](../../plans/task_m100_6040.md)과
[구현 계획](../../plans/task_m100_6040_impl.md)에 이미 기록돼 있다.

## 완료한 로컬 검증

제출 기준 `devel@7592e9c99`에 재배치한 code candidate에서 다음 검증을 완료했다.

| 검증 | 결과 |
| --- | --- |
| TypeScript | `npx tsc --noEmit` 통과 |
| Studio test | 1,278건 중 1,277 pass, 1 skip, 0 fail |
| Studio build | 243 modules 빌드 통과, 기존 대형 chunk 경고만 확인 |
| Rust format | 파생 regression suite 준비 뒤 `cargo fmt --all -- --check` 통과 |
| diff·문서 | `git diff --check`, 변경 Markdown 상대 링크 검사 통과 |
| 최신 base 병합 | `devel@d3b40a3d7`과 자동 merge tree 생성 통과 |

Rust source·test·fixture·renderer/export 출력은 바꾸지 않으므로 Cargo test와 Clippy, PDF/SVG visual
sweep은 실행하지 않았다. 파생 `tests/generated/`와 `tests/suites/manifest.json`은 ignored 검증 산출물로
PR에 포함하지 않았다.

## 실제 브라우저·시각 검증

`http://127.0.0.1:4176/?renderer=canvas2d`의 6쪽 문서를 실제 브라우저에서 열어 확인했다.

- 편집 viewport 1260px에서 자동 60%는 2열이고 점유 묶음 중심 오차는 0.14px였다.
- 같은 문서의 자동 50%는 3열이고 점유 묶음 중심 오차는 0.07px였다.
- A4 155%에서 20cm 라벨은 보이고 21cm 라벨은 숨겨졌으며 끝 tick·종이 경계는 남았다.
- 브라우저 warning/error log는 0건이었다.
- 작업지시자는 반복 zoom 영상 비교로 Stage 2·3 회귀를 확인했고, 제거 뒤 Stage 1 기준 재구성과
  Stage 1.1 결과를 직접 검증해 현재 범위에 동의했다.

이번 변경은 Studio viewport의 페이지 배치와 ruler overlay 동작이며 HWP/HWPX/PDF renderer 결과나
fixture를 바꾸지 않는다. 따라서 기준 PDF visual sweep 대신 자동 geometry 회귀와 실제 browser 동작을
직접 판정했다. #6040 이슈의 사용자 재현 이미지는 참고 근거이며 별도 제품 출력 기준 asset으로 승격하지
않는다.

## 위험과 후속 조건

- 이 PR은 #6040의 성능 최적화 전체가 아니라 검증된 배치 정합성 범위만 포함한다. 남은 Stage 2·3은
  기존 좌표계와 단일 소유권을 보장하는 별도 설계 없이는 다시 추가하지 않는다.
- #6041·#6042는 각각 직전 PR source branch를 base로 삼되, 각 layer의 diff·검증·승인 기록을 분리한다.
- code candidate의 required checks는 17 success, 15 scope-based skip, 0 failure·pending으로 완료됐다.
  [CI run 33301044206](https://github.com/edwardkim/rhwp/actions/runs/33301044206)의 Frontend package
  gate와 Build & Test aggregate,
  [CodeQL run 33301044201](https://github.com/edwardkim/rhwp/actions/runs/33301044201),
  [Render Diff run 33301044144](https://github.com/edwardkim/rhwp/actions/runs/33301044144),
  [Adapter inter-diff run 33301044199](https://github.com/edwardkim/rhwp/actions/runs/33301044199),
  [Proptest roundtrip run 33301044191](https://github.com/edwardkim/rhwp/actions/runs/33301044191)이 성공했다.
- review-only trailing head를 push한 뒤 exact code candidate의 녹색 결과 재사용 여부와 최신 required
  aggregate, mergeability를 확인한다.
- Draft 해제와 실제 merge는 stack 완성 상태를 공유한 뒤 작업지시자의 별도 승인을 받는다.

## 최종 권고

**Draft bottom PR로 조건부 수용.** 현재 Stage 1·1.1 구현과 로컬·브라우저 검증은 #6040의 배치 정합성
범위를 충족한다. 다만 code candidate와 trailing review-only head의 required checks가 모두 성공하고,
최신 base에 대한 mergeability를 다시 확인하기 전에는 Ready 전환이나 merge를 권고하지 않는다.
