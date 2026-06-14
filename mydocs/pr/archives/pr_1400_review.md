# PR #1400 검토 — 이미지 선택 후 Shift+방향키 크기 조절

- PR: https://github.com/edwardkim/rhwp/pull/1400
- 제목: `feat(studio): 이미지 선택 후 Shift+방향키 크기 조절 (#1231)`
- 작성일: 2026-06-13
- 작성자: `oksure`
- 관련 이슈: #1231
- base: `devel`
- head: `oksure:contrib/feat-1231-shift-arrow-resize` (`dab5b15165704a6078c4249a59b80fdc613731d8`)
- 검토 브랜치: `review/pr-1400`

## 1. 요약 판단

**수용 가능**으로 판단한다.

PR은 rhwp-studio의 개체 선택 모드에서 `Shift+방향키`로 이미지 크기를 조절하는 기능을
추가한다. 방향키 단독 이동은 유지하고, `Shift+방향키`만 resize 경로로 분기한다. 크기 계산은
`picture-resize.ts` 순수 함수로 분리되어 `node:test` 단위 테스트가 추가되었고, 실제 적용은
기존 드래그 리사이즈와 동일하게 `ResizeObjectCommand`로 undo/redo를 기록한다.

로컬 macOS 권장 검증과 GitHub CI가 모두 통과했다. 다만 방향 매핑은 `ArrowRight/Left`가
가로 증감, `ArrowDown/Up`이 세로 증감이라는 구현 기준이므로, 실제 한컴 동작과 완전히 같은지는
작업지시자의 수동 UI 확인을 거치면 더 안전하다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `MERGEABLE` |
| mergeStateStatus | `CLEAN` |
| 변경량 | 5 files, +194 / -3 |
| 작성자 | `oksure` |
| closing issues | PR 본문에 `Closes #1231` 명시 |

커밋:

- `3e012747` — `feat(studio): 이미지 선택 후 Shift+방향키 크기 조절 (#1231 한컴 정합)`
- `a16a09d9` — `fix(studio): 리뷰 반영 — 비변경 축 보존, 키 안전망, 2단계 적용, MIN_SIZE 단일화 (#1231)`
- `c5c6b6a7` — `Merge branch 'devel' into contrib/feat-1231-shift-arrow-resize`
- `dab5b151` — `Merge branch 'devel' into contrib/feat-1231-shift-arrow-resize`

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

## 3. 변경 검토

### 3.1 코드 변경

`rhwp-studio/src/engine/picture-resize.ts`:

- `MIN_SIZE_HWP = 283`을 새 순수 계산 모듈의 단일 출처로 정의
- `arrowResizeDelta()`로 방향키별 width/height 증감 매핑
- `computeArrowResize()`에서 현재 크기, 방향, step을 받아 before/after를 계산
- 비정상 width/height, 0 이하 step, 무변화 resize는 `null` 반환
- 변경 축만 반올림하고 비변경 축의 소수값은 보존

`rhwp-studio/src/engine/input-handler-keyboard.ts`:

- 개체 선택 모드 방향키 처리에서 `e.shiftKey` 분기 추가
- `Shift+방향키`는 `resizeSelectedPicture()`, 방향키 단독은 기존 `moveSelectedPicture()` 유지

`rhwp-studio/src/engine/input-handler-picture.ts`:

- `resizeSelectedPicture()` 추가
- 이동과 동일하게 `gridStepMm`을 HWPUNIT step으로 환산
- 다중 선택이면 선택된 개체 전체에 동일 방향 resize 적용
- 조회/계산을 먼저 완료한 뒤 set을 수행하는 2단계 구조로 부분 적용 가능성을 줄임
- undo/redo는 기존 드래그 리사이즈와 동일한 `ResizeObjectCommand` 경로 사용

`rhwp-studio/src/engine/input-handler.ts`:

- keyboard handler가 호출할 private wrapper 추가

`rhwp-studio/tests/picture-resize.test.ts`:

- 방향 매핑, step 적용, 최소 크기 clamp, 비정상 입력, 비변경 축 보존 테스트 추가

### 3.2 기존 경로와의 정합

- 드래그 리사이즈도 `setObjectProperties()`로 먼저 적용하고 `ResizeObjectCommand`를 기록한다.
- 새 keyboard resize도 동일한 command 타입을 사용한다.
- `ObjectResizeTarget`에 기록하는 필드는 기존 드래그 리사이즈 경로와 동일하게
  `sec/ppi/ci/type/cellPath/before/after`이다.
- 셀 내부 그림은 `cellPath` 경로를 탄다.

## 4. 로컬 검증

검토 브랜치: `review/pr-1400`

| 명령 | 결과 |
|---|---|
| `git diff --check upstream/devel...HEAD` | 통과 |
| `npm test` (`rhwp-studio`) | 통과, 70 passed |
| `cargo build --release` | 통과, 4m 01s |
| `cargo test --release --lib` | 통과, 1751 passed / 0 failed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `npm ci` (`rhwp-studio`) | 통과, 381 packages 설치, 취약점 0 |
| `npm run build` (`rhwp-studio`) | 환경 미충족으로 중단: `@wasm/rhwp.js` 타입/모듈 부재 |

`npm run build` 실패는 PR 변경 파일의 TypeScript 오류가 아니라 로컬 clone에 `pkg/` WASM 산출물이
없어서 발생했다. `rhwp-studio/tsconfig.json`과 `vite.config.ts`는 `@wasm/*`를 `../pkg/*`로
alias 처리하며, `pkg/`는 `.gitignore` 대상이다. 이번 PR은 Rust/WASM 변경이 아니므로 별도
`wasm-pack build --target web --out-dir pkg` 검증은 수행하지 않았다.

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| 실제 한컴 Shift+방향키 방향 매핑 차이 | 낮음~중간 | 현재 구현은 직관적 축 매핑. 수동 UI 확인 권장 |
| 셀 높이 자동 재조정 연동 | 낮음 | #1231 본문도 #1183 연동을 언급하지만 본 PR 범위 밖으로 보임 |
| header/footer 그림 undo 경로 | 낮음 | 기존 `ResizeObjectCommand`가 headerFooter 전용 target을 갖지 않는 구조. 본 PR은 기존 drag resize와 동일 패턴 |

## 6. 권고

로컬 테스트와 GitHub CI 기준으로는 merge 가능하다.

머지 전 마지막 확인:

- 작업지시자가 실제 UI에서 이미지 선택 후 `Shift+방향키` 방향 매핑을 한컴 기대와 비교
- 리뷰 문서를 merge 대상 diff에 포함할 반영 방식 결정
- 머지 후 #1231 auto-close 여부 확인
