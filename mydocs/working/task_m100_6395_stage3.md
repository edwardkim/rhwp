# Task M100 #6395 Stage 3 — PR 리뷰 상태 수명 보정

- **Issue**: [#6395](https://github.com/edwardkim/rhwp/issues/6395)
- **PR**: [#6396](https://github.com/edwardkim/rhwp/pull/6396)
- **브랜치**: `codex/issue-6395-page-break-caret-reveal`
- **시작 head**: `b760dab95aef2d124bd8c5954eca48f17b301019`
- **보정 계획 commit**: `dfd6e124e`
- **수행일**: 2026-08-30 KST
- **상태**: 구현·로컬 검증 완료, code candidate commit 준비

## 1. 리뷰 판정

`CaretLayoutReveal`의 one-shot 예약은 성공한 `document-layout-refreshed`에서만 소비됐다. mutation renderer
선택 중 문서가 교체되면 `CanvasView.refreshPagesForMutation()`이 stale revision으로 조기 종료해 완료 이벤트를
보내지 않을 수 있다. 같은 `InputHandler` 인스턴스가 새 문서에도 재사용되고 기존 `deactivate()`가 예약을
초기화하지 않아, 다음 문서의 첫 full mutation이 이전 예약을 소비하며 불필요한 캐럿 reveal·스크롤을 일으킬
수 있었다.

이 위험은 드물지만 문서 간 상태 누수이므로 PR #6396 merge 전에 보정하는 것으로 판정했다.

## 2. 보정 내용

- `CaretLayoutReveal.clear()`를 추가해 완료 이벤트와 별개로 예약을 폐기할 수 있게 했다.
- 문서 초기화 공통 시퀀스가 호출하는 `InputHandler.deactivate()`에서 `clear()`를 실행했다.
- `pageBreak` 예약 뒤 문서 전환 초기화를 거치면 다음 `consume()`이 `false`인지 단위 테스트로 고정했다.
- 기존 allowlist, 일반 명령 보존, 1회 소비 계약과 실제 쪽 나누기 동작은 바꾸지 않았다.

## 3. 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cd rhwp-studio && node --test tests/caret-layout-reveal.test.ts` | PASS — 4/4 |
| `cd rhwp-studio && npx tsc --noEmit` | PASS |
| `cd rhwp-studio && npm test` | PASS — tests 1,314, pass 1,313, skip 1, fail 0 |
| `cd rhwp-studio && npm run build` | PASS — 245 modules, 기존 chunk-size 경고만 발생 |
| `CHROME_PATH=... npm run e2e:page-break-caret` | PASS — 실제 headless Chrome |
| `python3 scripts/check_e2e_manifest.py` | PASS — tracked 122개 / manifest 122행 |
| `git diff --check` | PASS |

Chrome E2E는 보정 전과 같은 결과를 확인했다.

- 새 커서: section 0, paragraph 1, offset 0
- 새 쪽: page index 1, page offset `1713.75`
- DOM 캐럿: `1912.2px`, 기대값 `1912.2px`
- 편집 영역: `scrollTop=1214`
- viewport 안 캐럿: `698.2..718.15px / 738px`

## 4. 범위와 남은 조건

보정은 rhwp-studio TypeScript 상태 수명과 단위 테스트만 변경한다. Rust source·test, npm/editor public API,
fixture, 렌더러의 문서 출력은 바꾸지 않아 Rust lint 묶음, package 검증, PDF/SVG visual sweep은 적용하지
않았다.

code candidate와 trailing review 기록을 push한 뒤 최신 PR head의 GitHub Actions와
`MERGEABLE/CLEAN`을 다시 확인한다. 병합은 작업지시자의 별도 승인 조건이다.
