# Task M100 #6395 Stage 1 — 쪽 나누기 캐럿·viewport 추종 구현

- **Issue**: [#6395](https://github.com/edwardkim/rhwp/issues/6395)
- **브랜치**: `codex/issue-6395-page-break-caret-reveal`
- **기준**: `upstream/devel` `2deb3dd6163d83d2932ab58ac5a0bf61bfce6d31`
- **계획 commit**: `9ac158614`
- **수행일**: 2026-08-30 KST
- **상태**: 구현·로컬 검증 완료, code candidate commit 준비

## 1. 완료 결과

WASM 편집 결과보다 늦게 확정되는 `CanvasView`·`VirtualScroll` 페이지 배치를 캐럿 재표시의 완료 경계로
연결했다. 쪽/단 나누기 operation에만 one-shot reveal을 예약하고, current mutation revision의 페이지 갱신이
끝나면 cursor rect를 다시 계산해 기존 `updateCaret()`의 DOM 배치와 scroll-into-view를 실행한다.

일반 텍스트 입력은 예약을 만들지 않으며, 한 번의 완료 이벤트가 예약을 소비한 뒤에는 같은 캐럿 이동을
반복하지 않는다.

## 2. 변경 내역

- `CaretLayoutReveal` 순수 정책을 추가했다.
- snapshot 실행, undo, redo의 쪽/단 나누기 history type에서 reveal을 예약했다.
- `CanvasView.refreshPagesForMutation()`의 current revision page refresh 직후
  `document-layout-refreshed`를 발행했다.
- 예약된 `InputHandler`만 cursor rect와 캐럿을 다시 갱신했다.
- 정책 unit test 3건과 150% 실제 Chrome `Meta+Enter` E2E를 추가했다.
- E2E npm script와 단일 권위 manifest 행을 추가했다.

## 3. 검증 결과

| 검증 | 결과 |
| --- | --- |
| `cd rhwp-studio && npx tsc --noEmit` | PASS |
| `cd rhwp-studio && npm test` | PASS — 1,246건 중 1,245 통과, 1 skip, 실패 0 |
| `cd rhwp-studio && npm run build` | PASS — Vite chunk-size 경고만 발생 |
| `CHROME_PATH=... npm run e2e:page-break-caret` | PASS — headless Google Chrome |
| `python3 scripts/check_e2e_manifest.py` | PASS — tracked 121개 / manifest 121행 |

브라우저 E2E의 최종 관측값은 다음과 같다.

- 준비 fixture: WASM/VirtualScroll 모두 1쪽
- 쪽 나누기 뒤 커서: section 0, paragraph 1, offset 0
- 새 쪽: page index 1, page offset `1713.75`
- DOM 캐럿: `1912.2px`, 계산 기대값 `1912.2px`
- 편집 영역: `scrollTop=1214`
- viewport 안 캐럿: `698.2..718.15px / 738px`

첫 E2E 시도는 sandbox가 Vite의 로컬 포트 탐색을 허용하지 않아 `failed to find an available port`로
종료했다. 같은 명령을 승인된 호스트 실행으로 즉시 다시 수행해 통과했다. 계획 초안에 잘못 적었던 존재하지
않는 `check-e2e-manifest.mjs` 명령도 정본 Python checker로 정정하고 실제 통과를 확인했다.

## 4. 시각 판정과 범위

이 변경은 HWP/PDF 문서 렌더 결과나 pagination 알고리즘을 바꾸지 않고 Studio의 캐럿·viewport 표시 시점만
바꾼다. 따라서 HWP/HWPX/PDF fixture와 PDF/SVG visual sweep은 적용하지 않았다. 대신 실제 Chrome에서 새 쪽
page offset, DOM 캐럿 좌표와 viewport 경계를 함께 단언한 E2E를 사용자-visible 동작의 직접 증적으로 삼았다.

## 5. 잔여 위험

- `columnBreak`와 쪽/단 나누기 undo·redo는 동일 history type 정책을 unit test로 고정했지만 각각의 별도
  browser E2E는 추가하지 않았다.
- 한글 2024 비교는 사용자 제보를 기대 동작으로 사용했으며 제품 자동화로 직접 재측정하지 않았다.
- GitHub Actions와 mergeability는 PR 최신 head에서 별도로 확인해야 한다.
