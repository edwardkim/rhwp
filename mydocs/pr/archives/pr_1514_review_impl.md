# PR #1514 검토 구현계획서 - rawSvg 재렌더 트리거 보정

- PR: https://github.com/edwardkim/rhwp/pull/1514
- 제목: `rhwp-studio 캔버스: 차트/OLE(rawSvg) 첫 로드 공백·고착 해소 (Refs #1456)`
- 작성자: `johndoekim` (외부 contributor, 작성 시점 `gh pr list --author johndoekim` 기준 누적 16 PR)
- 연결 이슈: #1456 (`Refs`, merge 후 close 확인 필요)
- 작성일: 2026-06-25
- 적용 경로: collaborator-mediated 외부 PR 처리 경로

## 1. 목적

PR #1514가 issue #1456의 근본 원인인 `rawSvg` 단독 페이지의 비동기 디코드 재렌더 누락을 올바르게 해소하는지 검토하고, merge 전 필요한 최신화·문서화·CI 재확인 절차를 정리한다.

이번 문서는 PR head에 검토 기록을 포함하기 위한 사전 계획 문서다. 아직 확정되지 않은 `mergeable`, `head SHA`, CI 상태, merge 여부는 작성 시점 참고값 또는 merge 전 조건으로만 기록한다.

## 2. 검토 기준

1. PR 메타 확인
   - base가 `devel`인지 확인한다.
   - `maintainerCanModify`가 `true`인지 확인한다.
   - 작성자 PR 누적 이력을 직접 확인한다.
   - 관련 이슈 #1456의 증상·원인·수정 방향과 PR 내용이 일치하는지 확인한다.

2. 코드 리뷰
   - `rhwp-studio/src/view/page-renderer.ts`의 변경이 실제 Canvas2D 화면 경로에 걸리는지 확인한다.
   - `imageCount`의 기존 의미를 보존하면서 `rawSvgCount`를 재렌더 트리거에 합산하는 방식이 적절한지 확인한다.
   - `renderPageFlow` 등 우회 경로가 현재 라이브 호출처인지 확인한다.
   - 문서 교체 시 `imageRetryCounts`가 초기화되어 같은 페이지·같은 카운트 문서에서 재렌더 예약이 누락되지 않는지 확인한다.
   - Rust 렌더러, native `export-svg`, HWP3 규칙에 영향이 없는지 확인한다.

3. 테스트 리뷰
   - 신규 E2E `issue-1456-chart-rerender.test.mjs`가 수정 전 실패·수정 후 성공을 포착하는지 확인한다.
   - 픽셀 기준이 차트 공백과 서로 다른 차트 재사용을 구분할 수 있는지 확인한다.
   - stale `pkg/`로 인한 `getShowParagraphMarks` 오류가 PR 결함과 별개인지 확인한다.

4. 검증
   - TypeScript 타입 검사와 관련 Node test를 base/head에서 비교한다.
   - 동일 E2E를 base 앱과 head 앱에 각각 붙여 negative control과 수정 후 결과를 확인한다.
   - GitHub Actions 실패가 PR 코드 결함인지 환경 실패인지 구분한다.

5. merge 준비
   - 최신 `upstream/devel`과 PR head의 충돌 여부를 확인한다.
   - PR head가 `BEHIND`이면 contributor 커밋을 rewrite하지 않고 `devel` merge commit으로 최신화한다.
   - 본 검토 문서와 `pr_1514_review.md`를 별도 문서 커밋으로 PR head에 추가한다.
   - push 후 GitHub Actions가 최신 head 기준 green인지 확인한다.
   - 작업지시자 승인 전 GitHub approve, merge, issue close를 수행하지 않는다.

## 3. Stage 계획

### Stage 1 - PR 메타와 변경 범위 확인

- GitHub PR 메타 확인: base/head, `maintainerCanModify`, `mergeable`, `mergeStateStatus`, 파일 목록, 커밋 목록, CI 상태.
- contributor 이력 확인: `johndoekim`은 작성 시점 기준 누적 16 PR, 첫 PR 아님.
- 변경 범위 분류:
  - 핵심 코드: `rhwp-studio/src/view/page-renderer.ts`
  - 회귀 테스트: `rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs`
  - contributor 내부 task 문서: `mydocs/plans`, `mydocs/working`, `mydocs/report`
- 완료 기준: volatile 상태값은 작성 시점 참고값으로만 정리한다.

### Stage 2 - 코드 리뷰

- `renderPage` 호출 경로가 `canvas-view.ts`의 현재 Studio 화면 경로인지 확인한다.
- `collectLayerPlaneSummary`가 `op.type === 'rawSvg'`를 카운트하고, `renderPage`가 `imageCount + rawSvgCount`를 `scheduleReRender`에 전달하는지 확인한다.
- `scheduleReRender`의 카운트 중복 방지 로직과 문서 교체 시 `resetImageRetryState` 호출을 함께 검토한다.
- `prefetchLayerImages`가 이미 `rawSvg` 내부 data URL 추출을 지원하므로, 이번 PR의 trigger gate 보정이 최소 수정인지 확인한다.
- 완료 기준: blocking finding 유무와 잔여 리스크를 결정한다.

### Stage 3 - 로컬 전후 비교 검증

- base/head worktree:
  - base: `/private/tmp/rhwp-pr1514-base` (`d932c582`)
  - head: `/private/tmp/rhwp-pr1514-head` (`b6419761`)
- 빠른 검증:
  - base/head 각각 `./node_modules/.bin/tsc --noEmit`
  - base/head 각각 `node --experimental-strip-types --test tests/render-backend.test.ts`
- E2E 비교:
  - base 앱에 head의 E2E를 붙여 실패해야 한다.
  - head 앱에 같은 E2E를 붙여 통과해야 한다.
  - stale `pkg/` 이슈가 있으면 `wasm-pack build --target web`로 최신 `pkg/`를 생성한 뒤 비교한다.
- 완료 기준: issue #1456 증상이 수정 전 재현되고 수정 후 해소됨을 같은 테스트로 확인한다.

### Stage 4 - merge 준비 절차 정리

- 최신 `upstream/devel`과 PR head merge 충돌 여부를 확인한다.
- PR이 `BEHIND`이면 다음 순서로 진행한다.
  1. PR head 로컬 브랜치 생성 또는 현재 worktree에서 브랜치화
  2. `upstream/devel` merge commit 추가
  3. `pr_1514_review_impl.md`, `pr_1514_review.md` 문서 커밋 추가
  4. contributor fork head branch `fix/1456-canvas-rawsvg-rerender`로 push
  5. 최신 GitHub Actions green 확인
- 완료 기준: 작업지시자에게 approve/merge 진행 여부를 다시 확인할 수 있는 상태가 된다.

## 4. merge 전 조건

- PR head 최신 커밋 기준 GitHub Actions 통과
- `mergeStateStatus` 최신 확인
- review 문서 2건이 PR diff에 포함됨
- GitHub review 또는 PR comment로 검토 결과를 남김
- 작업지시자 승인

## 5. merge 후 확인 사항

- merge commit이 `devel`에 포함되었는지 확인한다.
- #1456 자동 close 여부를 확인한다.
- 자동 close가 되지 않았으면 작업지시자 승인 후 수동 close한다.
- contributor 감사 코멘트는 과장 없이 사실 중심으로 남긴다.
- PR 처리 문서가 `mydocs/pr/archives/`에 포함되었는지 확인한다.
