---
kind: pr-review
status: pending-push
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
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

| 항목 | 2026-09-01 로컬 후보 |
| --- | --- |
| PR | [#6458](https://github.com/edwardkim/rhwp/pull/6458) |
| 관련 issue | [#6040](https://github.com/edwardkim/rhwp/issues/6040) |
| base / head | `devel` / `codex/issue-6040-zoom-topology` |
| 최신 통합 devel | `b9d408f0d` |
| devel merge commit | `a19020085`, `db1a15fb1` |
| Stage 1.2 code candidate | `333fa8f5d` |
| local integration candidate | `afa32ef18` |
| remote head | `0a6e6e2a320611ec5f2bb64483f927242597c251` |
| 누적 규모 | 20 files, `+1153/-51`, 11 commits |
| 원격 상태 | Draft, local candidate push·required CI 대기 |

PR #6458은 GitHub native stack의 bottom PR이다. 현재 로컬 후보는 최신 `upstream/devel`을 merge commit으로
통합했으며, 원격 Draft head는 maintainer review 이전 후보에 머물러 있다. 따라서 아래 Stage 1.2 해결 근거는
push 전 로컬 판정이고 원격 required checks의 성공을 주장하지 않는다.

현재 code candidate가 포함하는 범위는 다음과 같다.

- 고정 50% gate 대신 표시 페이지 폭·쪽 간격·편집 viewport 폭으로 자동 열 수를 고른다.
- 열 수를 실제 페이지 수로 제한하고, 빈 grid slot이 아니라 실제 점유 페이지 묶음을 가운데에 둔다.
- 이전 auto 열 commit을 live `VirtualScroll` 경로가 보존·전달해 경계에서 히스테리시스를 적용한다.
- horizontal·고정 배치·문서 교체에서 이전 auto commit을 reset한다.
- resize 레이아웃 확정 뒤 캐럿·선택 overlay를 같은 authoritative 좌표에 다시 투영한다.
- 155% A4 수평 눈금자에서 종이 밖으로 넘는 `21` 라벨만 숨기고 끝 tick과 종이 경계는 유지한다.

PR 본문은 `Refs #6040`을 사용한다. #6040 본문의 폐기된 CSS zoom preview·점진 Canvas 교체와 #6041
render scale, #6042 LRU·scheduler는 포함하지 않으므로 이 PR로 issue를 닫지 않는다.

## maintainer blocker와 로컬 해결 판정

Maintainer는 [review comment](https://github.com/edwardkim/rhwp/pull/6458#issuecomment-5487030630)에서
순수 helper만 `committedColumns`를 받고 live 경로가 이전 commit을 전달하지 않는 점을 차단했다.

| 요청 | 로컬 후보 판정 |
| --- | --- |
| live auto 경로가 이전 commit 소유·전달 | `VirtualScroll.committedAutoColumns`로 해결 |
| 동일 VirtualScroll 연속 입력 test | `800→814→806→818→806→801px`, `1→1→1→2→2→1열` 통과 |
| 실제 `CanvasView.onZoomChanged()` test | Vite SSR actual method 실행, zoom event당 layout commit 1회 확인 |
| Canvas pool/DOM 소유권 | 매 settled zoom에서 active page·DOM canvas·고유 slot 1:1 확인 |
| 실제 browser 경계 zoom·resize | 77쪽 `kps-ai.hwp`의 1↔2열 왕복과 ruler·caret·hit-test 확인 |
| 최신 devel 통합 | 첫 통합 뒤 전진한 `upstream/devel@b9d408f0d`까지 `db1a15fb1`로 재통합 |
| 내부 review 현행화 | 이 문서와 Stage 1.2 보고서를 local candidate 기준으로 갱신 |
| child #6467 restack | #6454 측정·결정 gate 뒤 수행하도록 보류 |

PR #6438의 progressive Canvas 교체는 active pool을 새 Canvas로 덮고 기존 DOM Canvas를 orphan으로
남기는 문제와 동기 렌더 장시간 작업이 review에서 확인됐다. 이 후보는 해당 경로를 가져오지 않고 기존
settled `releaseAllRenderedPages() → updateVisiblePages()`를 보존한다. actual CanvasPool test는 이 결정이
매 zoom 뒤 단일 소유권을 유지함을 고정한다.

## 완료한 로컬 검증

latest integration candidate `afa32ef18`에서 다음을 완료했다.

| 검증 | 결과 |
| --- | --- |
| focused | Stage 1.2 + 최신 Ruler actual harness 45/45 pass |
| TypeScript | `npx tsc --noEmit` 통과 |
| Studio test | 1,361건 중 1,360 pass, 1 policy skip, 0 fail |
| Studio build | 246 modules 빌드 통과, 기존 browser externalize·대형 chunk 경고만 확인 |
| WASM binding | 공식 `scripts/wasm-pack-locked.sh`로 최신 devel binding 생성 확인, tracked diff 없음 |
| diff | `git diff --check` 통과 |

첫 push 직후 #6570이 `devel`에 병합돼 오늘 작업 기록 한 파일이 충돌했다. 양쪽 기록을 모두 보존해
재통합했다. 제품 source는 자동 병합됐고, #6570의 실제 Ruler harness Canvas mock에 Stage 1.1이 사용하는
`measureText()`가 없어서 통합 test만 실패했다. 숫자 라벨의 양수 폭을 반환하는 fake를 한 줄 추가한 뒤
Ruler pointer·resize wrapper와 전체 Studio를 다시 통과시켰다.

Rust source·test·fixture·renderer/export 출력은 바꾸지 않으므로 Rust Clippy 묶음과 PDF/SVG visual sweep은
실행하지 않았다. Studio 화면 배치와 overlay 동작은 actual method test와 실제 browser 결과를 권위
근거로 사용한다.

## 실제 브라우저·시각 검증

Canvas2D로 77쪽 `kps-ai.hwp`를 열고 자동 배치 77%에서 편집 viewport 폭을 왕복했다.

| 편집 viewport 폭 | 1225px | 1235px | 1240px | 1235px | 1220px |
| --- | ---: | ---: | ---: | ---: | ---: |
| 확정 열 수 | 1 | 1 | 2 | 2 | 1 |
| DOM canvas / 고유 page slot | 3/3 | 3/3 | 4/4 | 4/4 | 3/3 |
| 현재 쪽 | 2/77 | 2/77 | 2/77 | 2/77 | 2/77 |

- 증가·감소 dead band 안에서는 기존 열 commit을 유지하고 경계 밖에서만 1↔2열로 전환했다.
- 캐럿은 매 단계 2쪽 안에 있었고 1열로 돌아오면 처음 좌표로 복귀했다.
- 수평 눈금자 canvas 폭은 매 단계 편집 viewport 폭과 정확히 일치했다.
- 2쪽 클릭 hit-test 뒤 현재 쪽이 2/77로 유지됐다.
- DOM canvas 수와 고유 page slot 수가 항상 같아 orphan/중복 Canvas가 없었다.

위 표는 최신 `devel@b9d408f0d` 통합 뒤 다시 얻은 결과다. #6570 눈금자 resize/paint 변경 뒤에도 ruler
bitmap/client 폭은 각 단계 `1225, 1235, 1240, 1235, 1220px`로 viewport와 정확히 일치했다.

Stage 1.1의 기존 브라우저 근거도 유지된다. 6쪽 문서 자동 60%는 2열, 50%는 3열이며 A4 155%에서
20cm 라벨은 보이고 21cm 라벨만 숨겨진다. 끝 tick과 종이 경계는 남는다.

## 외부 PR 인계와 stack 후속 조건

- #6444는 contributor credit과 source head 계보를 댓글로 남기고 #6458→#6467→#6042에 인계한 뒤
  2026-09-01 닫았다.
- #6438은 이 local candidate를 #6458 원격에 push해 maintainer가 대체 구현을 확인할 수 있게 한 뒤,
  contributor credit·검토에서 얻은 Canvas ownership 조건을 명시하고 닫는다.
- #6454에서 공유 frame 좌표계의 비용·효과를 먼저 측정한다. 그 결정 전에 #6467을 새 bottom head 위로
  반복 restack하지 않는다.
- #6454 결정 뒤 #6467을 #6458 위에 restack하고 layer diff·CI·34/50/100% 시각 증적을 다시 검증한다.
- #6042는 그 stack이 안정된 뒤 재개한다.

## 최종 권고

**Stage 1.2 local candidate는 push 가능한 상태지만 PR은 Draft로 유지한다.** Maintainer가 지적한 live
hysteresis, actual CanvasView 경로, Canvas 단일 소유권, 브라우저 caret/ruler/hit-test, 최신 devel 통합은
로컬에서 해결·검증했다. 다음 절차는 local candidate push와 #6458 설명 댓글, 새 required CI 확인이다.
그 뒤 #6438을 인계·종료하고 하이퍼워터폴 다음 gate인 #6454 측정 계획으로 이동한다.
