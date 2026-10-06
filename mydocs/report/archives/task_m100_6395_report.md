# Task M100 #6395 최종 보고서 — 쪽 나누기 캐럿·viewport 추종

- **Issue**: [#6395](https://github.com/edwardkim/rhwp/issues/6395)
- **브랜치**: `codex/issue-6395-page-break-caret-reveal`
- **기준 commit**: `upstream/devel` `2deb3dd6163d83d2932ab58ac5a0bf61bfce6d31`
- **충돌 해소 기준**: `upstream/devel` `d3b40a3d7c3ecb5d0f014ce604b99fda17b2bd9b`
- **계획 commit**: `9ac158614`
- **구현 commit**: `2d64b1d0f`
- **리뷰 보정 commit**: `a21bbc9a0`
- **PR**: [#6396](https://github.com/edwardkim/rhwp/pull/6396)
- **작성일**: 2026-08-30 KST
- **상태**: 최신 devel 충돌 해소·리뷰 보정·로컬 재검증 완료, Open PR 최신 head CI·merge 승인 대기

## 1. 결론

쪽 나누기 직후의 문서 커서와 화면 캐럿이 서로 다른 페이지 배치를 보던 비동기 경계를 정합화했다. 이제
새 쪽이 기존 viewport 밖에 생기는 150% 배율에서도 `Command+Enter` 뒤 캐럿이 새 쪽 첫 문단으로 이동하고,
편집 영역도 해당 캐럿이 보이도록 즉시 스크롤된다.

## 2. 원인

WASM의 page break 편집과 cursor position은 즉시 갱신됐지만, `CanvasView`의 mutation renderer 선택과
`VirtualScroll` page offset 재계산은 비동기로 끝났다. `InputHandler.afterEdit()` 직후의 첫
`updateCaret()`은 이전 페이지 목록을 사용해 새 page index의 offset을 0으로 계산했다. 새 레이아웃이 나중에
도착해도 쪽 나누기 캐럿을 다시 계산하고 reveal하는 완료 경계가 없었다.

## 3. 해결

- current mutation revision의 page refresh가 끝난 시점을 `document-layout-refreshed` 내부 이벤트로 알렸다.
- 쪽/단 나누기와 해당 history snapshot만 다음 완료 이벤트에서 한 번 reveal하도록 제한했다.
- 완료 이벤트에서 cursor rect를 새 page offset으로 다시 계산하고 기존 DOM 캐럿·scroll-into-view 경로를
  재사용했다.
- 일반 편집은 같은 이벤트를 받아도 예약이 없어 scroll 위치를 바꾸지 않는다.
- 문서 전환과 renderer 선택이 경합해 완료 이벤트가 생략돼도 이전 문서의 예약이 다음 문서로 넘어가지 않도록
  `deactivate()`에서 one-shot 상태를 초기화한다.

## 4. 회귀 검증

| 명령 | 결과 |
| --- | --- |
| `cd rhwp-studio && npx tsc --noEmit` | PASS |
| `cd rhwp-studio && npm test` | PASS — tests 1,314, pass 1,313, skip 1, fail 0 |
| `cd rhwp-studio && npm run build` | PASS — 245 modules |
| `CHROME_PATH=... npm run e2e:page-break-caret` | PASS — 실제 headless Chrome |
| `python3 scripts/check_e2e_manifest.py` | PASS — 122/122 |
| `git diff --check` | PASS |

핵심 E2E는 한 쪽짜리 새 문서를 150%로 표시한 뒤 실제 `Meta+Enter`를 입력했다. 새 커서는 paragraph 1,
offset 0과 page index 1을 가리켰다. 새 page offset `1713.75`를 사용한 DOM 캐럿 `1912.2px`가 계산 기대값과
일치했고, `scrollTop=1214`로 이동해 높이 738px viewport의 `698.2..718.15px` 안에 표시됐다.

## 5. 변경 파일

- `rhwp-studio/src/engine/caret-layout-reveal.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/tests/caret-layout-reveal.test.ts`
- `rhwp-studio/e2e/page-break-caret-reveal.test.mjs`
- `rhwp-studio/e2e/MANIFEST.md`
- `rhwp-studio/package.json`
- Task #6395 계획·단계·최종 보고 문서

## 6. 검증 범위 판정

변경은 rhwp-studio의 캐럿·viewport 표시 시점과 TypeScript test만 다룬다. Rust source, Rust test/baseline
helper, npm/editor public API, HWP/HWPX fixture, document renderer의 pagination 결과는 바꾸지 않았다. 이에
따라 Rust lint/Cargo 회귀, package 검증, PDF/SVG visual sweep은 적용하지 않았다. 실제 Chrome E2E가 이번
사용자-visible 표시 계약의 직접 증적이다.

## 7. 제출 결과와 잔여 조건

- Open PR #6396을 생성했고 collaborator self-review와 오늘할일을 같은 source branch의 trailing 문서
  commit으로 추가했다.
- `upstream/devel@d3b40a3d7` 병합에서 오늘할일과 `InputHandler` 충돌을 양쪽 동작 보존으로 해소했다.
  최신 merge tree의 TypeScript·전체 Studio/editor test·build·실제 Chrome E2E도 다시 통과했다.
- self-review에서 발견한 문서 간 pending reveal 누수를 `a21bbc9a0`에서 보정하고 전환 회귀 단위 테스트와
  전체 Studio·실제 Chrome 검증을 다시 통과했다.
- 최신 PR head의 GitHub Actions, mergeable 상태와 작업지시자의 merge 승인은 별도 조건이다.
- 이 보고서는 한글 2024를 자동화해 직접 측정했다는 주장을 하지 않는다. 사용자 제보의 기대 동작을 Studio
  브라우저 회귀 계약으로 고정한 결과다.
