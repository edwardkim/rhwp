# 구현 계획 — Task M100 #6395

- **이슈**: [#6395](https://github.com/edwardkim/rhwp/issues/6395)
- **브랜치**: `codex/issue-6395-page-break-caret-reveal`
- **기준 commit**: `upstream/devel` `2deb3dd6163d83d2932ab58ac5a0bf61bfce6d31`
- **작성일**: 2026-08-30 KST
- **문서 성격**: 이슈 등록 전에 만들어진 로컬 후보 구현의 파일 단위 설계를 사후 정식화하고, 커밋 전 변경
  경계와 검증 계약을 확정한다.

## 1. 이벤트 계약

`CanvasView.refreshPagesForMutation()`은 renderer revision을 선택하고 그 선택이 아직 current인지 확인한 뒤
`refreshPages()`로 page dimensions와 `VirtualScroll` 배치를 갱신한다. 그 직후 내부 event bus에
`document-layout-refreshed`를 보낸다.

이 이벤트 자체는 모든 mutation에서 발생할 수 있다. `InputHandler`는 별도 one-shot 예약이 있는 경우에만
소비하므로 일반 mutation의 캐럿이나 scroll 위치는 바꾸지 않는다.

## 2. 파일별 변경

### `rhwp-studio/src/engine/caret-layout-reveal.ts` (신규)

- 허용 operation type을 `pageBreak`, `columnBreak`로 제한한다.
- undo/redo history의 `snapshot:` prefix를 제거한 base type으로 같은 정책을 적용한다.
- `requestFor()`는 허용 명령일 때 pending을 세우고, `consume()`은 값을 반환하면서 즉시 지운다.

### `rhwp-studio/src/engine/input-handler.ts`

- snapshot edit 성공 뒤 실제 `operationType`으로 reveal을 예약한다.
- undo는 redo stack top, redo는 undo stack top의 history type으로 예약한다.
- `document-layout-refreshed` 수신 시 pending과 active 상태를 확인한다.
- 예약이 있으면 `cursor.updateRect()`로 새 page-local rect를 얻고 `updateCaret()`의 기존 DOM 배치와
  scroll-into-view 동작을 재사용한다.

### `rhwp-studio/src/view/canvas-view.ts`

- current mutation revision의 `refreshPages()` 직후 완료 이벤트를 보낸다.
- stale revision이나 선택 실패 경로에서는 이벤트를 보내지 않는다.

### `rhwp-studio/tests/caret-layout-reveal.test.ts` (신규)

- 쪽/단 나누기와 snapshot variant가 한 번만 소비되는지 확인한다.
- 일반 편집은 예약하지 않는지 확인한다.
- 경계 명령 뒤 일반 명령이 아직 도착하지 않은 예약을 지우지 않는지 확인한다.

### `rhwp-studio/e2e/page-break-caret-reveal.test.mjs` (신규)

- 새 문서에 텍스트를 입력하고 150% 배율, 첫 문단 끝으로 준비한다.
- 테스트 자체가 실제 `Meta+Enter` keyboard event를 보낸다.
- WASM/VirtualScroll page count가 2가 되고 cursor rect가 page index 1이 될 때까지 기다린다.
- 새 문단 첫 위치, 양수 page offset, DOM 캐럿과 예상 좌표 오차 1px 미만, 양수 `scrollTop`, viewport 20px
  여백 안의 캐럿을 단언한다.

### `rhwp-studio/package.json`, `rhwp-studio/e2e/MANIFEST.md`

- `e2e:page-break-caret` 실행 진입점과 단일 권위 manifest 행을 추가한다.

## 3. 검증 명령

```bash
cd rhwp-studio
npx tsc --noEmit
npm test
npm run build
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' npm run e2e:page-break-caret
cd ..
python3 scripts/check_e2e_manifest.py
git diff --check
```

새 E2E가 아직 untracked인 커밋 전 단계에서 manifest checker는 `git ls-files`를 사용하므로, 실제 index를
바꾸지 않는 임시 index에 intent-to-add를 설정해 신규 파일 포함 결과를 확인한다. PR candidate commit 뒤에는
일반 명령으로 다시 확인한다.

Rust source·test·baseline helper, npm/editor public API, HWP/HWPX fixture는 바꾸지 않으므로 Rust lint 묶음,
Cargo 전체 회귀, package 검증, PDF/SVG visual sweep은 적용하지 않는다. 사용자-visible 계약은 실제 Chrome
E2E의 cursor/page/DOM/viewport 수치로 직접 검증한다.

## 4. 완료 조건

1. 이슈 #6395 수용 기준과 구현 allowlist가 일치한다.
2. 쪽 나누기 직후 새 쪽 offset을 사용한 캐럿이 viewport 안에 표시된다.
3. 전체 Studio unit test와 build가 기존 회귀 없이 통과한다.
4. code candidate와 검증 결과를 stage·최종 보고서에 기록하고 Open PR로 제출한다.
