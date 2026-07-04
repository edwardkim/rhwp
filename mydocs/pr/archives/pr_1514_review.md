# PR #1514 리뷰 - rhwp-studio rawSvg 재렌더 트리거 보정

- PR: https://github.com/edwardkim/rhwp/pull/1514
- 제목: `rhwp-studio 캔버스: 차트/OLE(rawSvg) 첫 로드 공백·고착 해소 (Refs #1456)`
- 작성자: `johndoekim` (외부 contributor, 작성 시점 기준 누적 16 PR)
- 연결 이슈: #1456
- base/head: `devel` `d932c582` <- `johndoekim:fix/1456-canvas-rawsvg-rerender` `b6419761`
- 작성일: 2026-06-25
- 작성 시점 참고값: `MERGEABLE` / `BEHIND`, `maintainerCanModify=true`

## 1. 결론

**수용 권고.** Blocking finding 없음.

PR #1514는 issue #1456의 원인인 Studio Canvas2D 경로의 `rawSvg` 단독 페이지 재렌더 트리거 누락을 최소 변경으로 보정한다. `imageCount` 의미를 유지한 채 `rawSvgCount`를 별도로 추가하고, 실제 화면 렌더 경로의 `scheduleReRender` 입력에 `imageCount + rawSvgCount`를 전달하는 방식은 타당하다.

로컬 전후 비교에서도 수정 전 base는 차트 공백과 고착이 재현되고, PR head는 동일 테스트가 통과했다. 다만 작성 시점 GitHub PR 상태가 `BEHIND`이고 `Build & Test` 체크가 red이므로, merge 전에는 PR head 최신화와 최신 head 기준 CI green 확인이 필요하다.

## 2. PR 메타 확인

| 항목 | 내용 |
|---|---|
| base | `edwardkim/rhwp:devel` |
| head | `johndoekim/rhwp:fix/1456-canvas-rawsvg-rerender` |
| 작성자 | `johndoekim` (`gh pr list --author johndoekim` 기준 누적 16 PR, 첫 PR 아님) |
| 관련 이슈 | #1456 (`Refs`, merge 후 close 확인 필요) |
| 작성 시점 merge 상태 | `MERGEABLE` / `BEHIND` |
| maintainer 수정 가능 | `true` |
| GitHub CI | `Build & Test` 실패, CodeQL/Render Diff 성공, WASM Build skipped |

`Build & Test` 실패 로그는 `rustc-LLVM ERROR: IO failure on output stream: No space left on device` 및 runner `_diag` 로그 생성 실패로, PR 코드 결함보다는 GitHub runner 디스크 부족으로 판단한다. 그래도 merge 조건은 최신 head 기준 CI green이다.

## 3. 변경 범위

| 파일 | 변경 |
|---|---|
| `rhwp-studio/src/view/page-renderer.ts` | `LayerPlaneSummary.rawSvgCount` 추가, `collectLayerPlaneSummary`에서 `rawSvg` 카운트, `scheduleReRender`에 `imageCount + rawSvgCount` 전달 |
| `rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs` | 차트 HWP 첫 로드 비공백 및 서로 다른 차트 비재사용 E2E 추가 |
| `mydocs/plans/task_m100_1456.md` | contributor 수행계획서 |
| `mydocs/plans/task_m100_1456_impl.md` | contributor 구현계획서 |
| `mydocs/working/task_m100_1456_stage{1,2,3}.md` | contributor 단계 보고서 |
| `mydocs/report/task_m100_1456_report.md` | contributor 최종 보고서 |

코드 변경은 Studio TypeScript에 한정된다. Rust 렌더러, native `export-svg`, HWP3 파서 규칙에는 영향이 없다.

## 4. 코드 리뷰

### 4.1 원인과 수정 지점 일치

Issue #1456의 원인은 다음 흐름이다.

1. Studio 화면은 `renderPageToCanvasFiltered` Canvas2D 경로를 사용한다.
2. 차트/OLE는 PageLayerTree에서 `rawSvg` op로 나오고, `web_canvas.rs`에서 `HtmlImageElement` 비동기 디코드 경로를 탄다.
3. 최초 렌더 시 캐시 미스와 디코드 지연이 있으면 아무것도 그리지 않는다.
4. 기존 `scheduleReRender`는 `imageCount > 0`일 때만 작동한다.
5. 기존 `collectLayerPlaneSummary`는 `op.type === 'image'`만 세고 `rawSvg`를 세지 않아 `rawSvg` 단독 페이지에서 즉시 bail했다.

PR의 수정은 이 원인과 직접 대응한다. `rawSvg`를 재렌더 트리거 카운트에 포함시키면 기존 200/600/1500ms 재렌더와 `prefetchLayerImages` 안전망이 발화한다.

### 4.2 구현 방식

- `imageCount`는 기존 래스터 image op 의미를 유지하고, `rawSvgCount`를 별도 필드로 둔 점이 좋다. 향후 image와 rawSvg 경로를 구분해 디버깅하기 쉽다.
- `renderPage`에서만 `imageCount + rawSvgCount`를 합산해 `scheduleReRender`에 전달한다. 실제 Studio 화면 호출은 `canvas-view.ts`의 `PageRenderer.renderPage(...)` 경로다.
- `renderPageFlow(...)`는 검색 결과 현재 라이브 호출처가 없으므로 이번 범위에서 제외해도 된다.
- `prefetchLayerImages`는 이미 rawSvg 내부 `data:image/...;base64,...` 추출을 지원한다. 따라서 이번 PR은 안전망을 새로 만드는 변경이 아니라 기존 안전망의 gate를 올바르게 여는 최소 수정이다.
- 문서 교체 시 `canvas-view.ts`의 `releaseAllRenderedPages()`가 `resetImageRetryState()`를 호출한다. 같은 페이지 index와 같은 총 이미지 카운트의 문서를 연속 로드해도 `imageRetryCounts` 때문에 재렌더 예약이 스킵되는 구조는 아니다.

### 4.3 잔여 리스크

- 재렌더는 여전히 타이머 + prefetch 기반이다. `HtmlImageElement.onload` 기반 이벤트 구동 재렌더는 후속 #2 후보로 남는다.
- E2E의 유채색 픽셀 비율 기준은 샘플 차트와 현 렌더 색상에 의존한다. 다만 실제 측정값이 threshold 0.3% 대비 충분히 높아 회귀 감지용으로는 타당하다.
- GitHub CI 실패가 환경성으로 보이더라도, 최신화 후 CI green을 merge gate로 삼아야 한다.

## 5. 로컬 검증

검증은 현재 작업트리를 건드리지 않고 `/private/tmp`의 분리 worktree에서 수행했다.

| 구분 | 위치 | 커밋 |
|---|---|---|
| base | `/private/tmp/rhwp-pr1514-base` | `d932c582` |
| head | `/private/tmp/rhwp-pr1514-head` | `b6419761` |

### 5.1 타입/단위 테스트

| 항목 | base | head |
|---|---|---|
| `./node_modules/.bin/tsc --noEmit` | 통과 | 통과 |
| `node --experimental-strip-types --test tests/render-backend.test.ts` | 23/23 통과 | 23/23 통과 |
| `git diff --check d932c582..b6419761 -- rhwp-studio/src/view/page-renderer.ts rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs` | 통과 | 통과 |

### 5.2 E2E 전후 비교

stale `pkg/`로 수동 비교 중 `this.doc.getShowParagraphMarks is not a function` 모달이 발생했으나, 이는 PR 결함이 아니라 WASM 산출물과 TS 코드 버전 불일치였다. Docker daemon이 실행되지 않는 환경이라 base worktree에서 `wasm-pack build --target web`로 `pkg/`를 재생성했고, PR에는 Rust 변경이 없으므로 같은 `pkg/`를 head worktree에 복사해 비교했다.

| 대상 | 결과 |
|---|---|
| base 앱 `http://127.0.0.1:17700/` | 파일 로드 성공, 차트 A/B 유채색 0.011%, A/B 차이 0.00%, E2E 실패. 수정 전 공백·고착 재현 |
| head 앱 `http://127.0.0.1:17701/` | 파일 로드 성공, 차트 A 3.754%, 차트 B 2.834%, A/B 차이 5.40%, E2E 통과 |

작업지시자 수동 브라우저 검증도 완료됐다.

## 6. 최신 devel 반영 여부

작성 시점 PR은 `BEHIND` 상태다. 최신 `upstream/devel` `42d7f6bc`와 PR head `b6419761`의 `git merge-tree` 시뮬레이션은 충돌 없이 tree hash를 반환했다.

따라서 PR head 최신화는 contributor 커밋을 rewrite하지 않는 merge commit 방식이 적절하다.

권장 순서:

```bash
git fetch upstream devel pull/1514/head:local/pr1514
git checkout local/pr1514
git merge --no-ff upstream/devel -m "Merge devel into PR #1514"
git add mydocs/pr/archives/pr_1514_review_impl.md mydocs/pr/archives/pr_1514_review.md
git commit -m "docs: PR #1514 검토 기록"
git push https://github.com/johndoekim/rhwp.git HEAD:fix/1456-canvas-rawsvg-rerender
```

push 후 GitHub Actions가 최신 head 기준으로 다시 실행되어야 한다.

## 7. merge 전 조건

- PR head 최신 커밋 기준 GitHub Actions green
- `mergeStateStatus` 최신 재확인
- 본 검토 문서 2건이 PR diff에 포함됨
- GitHub review 또는 PR comment로 검토 결과 공유
- 작업지시자 승인

위 조건을 만족하면 admin merge 수용을 권고한다.

## 8. merge 후 확인

- merge commit이 `devel`에 포함되었는지 확인한다.
- #1456은 `Refs` 기반 PR이므로 자동 close가 되지 않을 수 있다. merge 후 issue 상태를 확인하고, 작업지시자 승인 후 필요 시 수동 close한다.
- PR comment는 과장 없이 사실 중심으로 남긴다.
