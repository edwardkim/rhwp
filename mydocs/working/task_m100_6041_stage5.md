# Task M100 #6041 Stage 5 — #6454 판정 뒤 #6467 restack·재자격화

- 날짜: 2026-09-02 KST
- Issue: [#6041](https://github.com/edwardkim/rhwp/issues/6041)
- Draft PR: [#6467](https://github.com/edwardkim/rhwp/pull/6467)
- 이전 head: `ba68cd655aed5fd94804f725c033cf615231ce4b`
- 새 base: #6458 `c2932ff30fbc45e3d89eefad7c75a71518acde33`
- 상태: 고유 6개 커밋 restack·local 재검증 완료, Draft 유지

## 선행 판정

[#6454 Stage 2 정본 분석](https://github.com/edwardkim/rhwp/issues/6454#issuecomment-5503939025)은
geometry 1·2·3·6·50·500쪽과 실문서 3개의 smooth/direct/resize 27조합에서 공유 zoom-frame geometry
snapshot의 성능 진입 조건을 충족하지 못했다. [종료 기록](https://github.com/edwardkim/rhwp/issues/6454#issuecomment-5504010935)을
남기고 제품 prototype·PR 없이 `NOT_PLANNED`로 닫았다.

따라서 별도 #6454 기반 PR을 stack에 삽입하지 않고 기존 **#6458 → #6467 → #6042** 순서를 유지한다.
#6521 저배율 LOD 실험도 화질 저하 대비 이득 부족으로 이미 `NOT_PLANNED`이므로 stack은 3단이다.

## restack

- 안전 백업 `codex/backup-6041-pre-restack-20260902@ba68cd655`를 먼저 만들었다.
- 최초 #6458 base `dfe27e188` 뒤의 #6467 고유 6개 커밋만 `c2932ff30` 위로 재배치했다.
- `canvas-view.ts` 충돌에서는 #6458의 HF overlay·`resetAutoColumnCommit()`과 #6467의 retained 집합,
  surface plan·effective DPR reset을 모두 유지했다.
- `mydocs/orders/20260830.md`는 최신 base의 다른 작업을 모두 보존하고 #6041·#6042 두 행만 당시
  #6467 기록으로 갱신했다.
- `range-diff`에서 6개 커밋이 모두 대응됨을 확인했다. source 의미 차이는 위 두 통합 지점뿐이다.

## 새 base에서 발견한 테스트 계약

첫 전체 Studio 실행은 새 #6458의 실제 `CanvasView` zoom-path 테스트가 prototype fixture에
`renderSurfaceDecisions`를 만들지 않아 1건 실패했다. 제품 인스턴스는 class field initializer로 항상
상태를 갖는다. 제품에 optional guard를 추가하지 않고 테스트 fixture가 `renderSurfacePlan=null`과 빈
decision map을 명시하도록 보정했다.

## 검증

- `node --test tests/canvas-view-page-arrangement.test.ts tests/render-surface-budget.test.ts`: 21/21
- `npx tsc --noEmit`: 통과
- `npm test`: 1,373 pass·1 skip·0 fail
- `npm run build`: 247 modules, 기존 CanvasKit externalization·대형 chunk 경고만 확인
- `npm run e2e:manifest-check`: 125/125
- `git diff --check`: 통과
- Rust source/test/fixture 변경 없음: Rust lint 묶음 대상 아님

실제 브라우저는 #6458 검증 후보와 같은 WASM SHA-256
`9a18f5638bf3550a8ea148cd6d296d0d2fb3a6378e9982b680366f985e7f9a09`를 사용했다. 1280×720,
Canvas2D, `exam_kor.hwp` 20쪽의 자동 34%에서 3열과 편집 영역 중심 오차 약 0.17px를 확인했다.
visible 6쪽·retained 3쪽은 모두 예산 이내 raw DPR 2였고 연결되지 않은 page Canvas와 browser
warning/error는 0이었다.

## 인계

- #6467은 전체 stack 검증 전까지 Draft를 유지한다.
- remote push 뒤 PR base/head·Draft·mergeability·latest check를 다시 확인하고 본문의 4단 계획을
  3단 계획으로 현행화한다.
- #6042 로컬 두 커밋은 새 #6467 head 위로 다시 맞춘 뒤 고정 before SHA와 Stage 2 승인 기록을 갱신한다.
- Ready 전환, merge, #6041 close는 이번 단계에서 수행하지 않는다.
