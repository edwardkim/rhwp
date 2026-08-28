# Task M100 #6108 결과 보고 — 쪽 배치별 맞춤 배율 단일화

- **이슈**: [#6108](https://github.com/edwardkim/rhwp/issues/6108)
- **브랜치**: `codex/issue-6108-zoom-fit`
- **최초 작업 기준**: `upstream/devel@2166f4065`
- **최종 통합 기준**: `upstream/devel@1fe0d1480`
- **보고일**: 2026-08-28 KST
- **절차 상태**: 로컬 구현·최신 devel 통합·재검증 완료, native stack 게시 준비

## 결과

자동·한 쪽·두 쪽·맞쪽·여러 쪽의 화면 점유 행·열을 하나의 맞춤 계산 계약으로 통합했다.

- 자동·한 쪽은 1×1, 두 쪽·맞쪽은 2×1, 여러 쪽은 지정한 columns×rows를 사용한다.
- 폭 맞춤은 한 행 전체 폭과 내부 쪽 간격을, 쪽 맞춤은 가로×세로 블록 전체와 내부 간격을 사용한다.
- 모든 결과는 문서 화면 배율의 공통 10~500% 범위로 제한한다.
- 보기 명령·상태 표시줄·확대/축소 대화상자·저장된 맞춤 복원이 같은 resolver를 사용한다.
- 여러 쪽에서는 적용되지 않는 고정·폭·쪽·사용자 정의 비율 선택을 비활성화하고, 지정 배열 전체 맞춤을
  강제한다.
- 소비되지 않던 쪽 배치 이벤트·CanvasView wrapper·슬라이더 class toggle을 제거했다.

## 단계별 변경

### Stage 1 — 순수 계산 계약

- commit: `9b8cc77d0 fix(studio): 쪽 배치별 맞춤 계산을 단일화한다`
- 쪽 배치별 columns·rows, frame padding, 내부 page gap, 10~500% clamp를
  `zoom-fit.ts`의 공통 계산으로 고정했다.
- 여러 쪽 전용 중복 계산을 제거하고 대화상자 상태 계산도 공통 resolver로 위임했다.
- focused test 28/28을 통과했다.

### Stage 2 — 진입점 단일화와 무사용 경로 정리

- commit: `1c75c49ca refactor(studio): 맞춤 배율 진입점을 단일화한다`
- `view.ts`의 `getZoomFitMetrics()`를 메뉴·상태 표시줄·대화상자가 공유한다.
- 대화상자 확인 시점에 현재 metrics를 다시 읽어 최종 배율을 계산한다.
- 여러 쪽 비율 입력 잠금과 legacy 이벤트·wrapper·class toggle 정리를 완료했다.
- focused test 42/42와 TypeScript 검사를 통과했다.

### Stage 3 — 실제 브라우저 회귀

기존 `zoom-fit-mode-persistence.test.mjs`를 실제 확대/축소 대화상자 경로까지 확장했다.

- 자동 배치의 상태 표시줄 폭·쪽 맞춤과 문서별/세션별 복원
- 수치 슬라이더 조작 시 저장된 맞춤 규칙 해제
- 한 쪽·두 쪽·맞쪽의 대화상자 폭 맞춤과 쪽 맞춤
- 여러 쪽 2×2 선택 시 모든 비율 radio·사용자 정의 입력 잠금
- 여러 쪽의 가로·세로 쪽 수 입력 활성화와 2×2 전체 블록 맞춤

Chrome E2E 28개 assertion이 모두 통과했다. 측정값은 다음과 같다.

| 배치·모드 | 실제 zoom | 기대 zoom | 결과 |
| --- | ---: | ---: | --- |
| 자동 폭 맞춤 | 1.537 | 1.537 | 통과 |
| 한 쪽 쪽 맞춤 | 0.640 | 0.640 | 통과 |
| 두 쪽 폭 맞춤 | 0.762 | 0.762 | 통과 |
| 맞쪽 폭 맞춤 | 0.762 | 0.762 | 통과 |
| 여러 쪽 2×2 전체 맞춤 | 0.315 | 0.315 | 통과 |

로컬 HTML 보고서는 `output/e2e/zoom-fit-mode-persistence-report.html`, 화면은
`rhwp-studio/e2e/screenshots/issue-6108-*.png`에 생성했다. 두 경로는 재현 가능한 검증 산출물이며
gitignore 대상으로 PR에는 포함하지 않는다. 한 쪽·두 쪽·맞쪽·여러 쪽 대표 화면을 직접 확인해 페이지
행·블록이 계산값과 같은 범위에 들어오는 것도 확인했다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `npm test` (`rhwp-studio`) | 1225 tests, 1224 pass, 1 skip, 0 fail |
| `npm run build` (`rhwp-studio`) | TypeScript + Vite production build 통과, 238 modules |
| `npm run e2e:zoom-fit-mode` (`VITE_URL=http://127.0.0.1:7728`) | 28/28 assertion 통과 |
| `node --check rhwp-studio/e2e/zoom-fit-mode-persistence.test.mjs` | 통과 |
| `cargo fmt --all` | 파생 regression suite 준비 후 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

`cargo fmt --all`은 이 worktree에 없는 CI용 `tests/generated/regression_suite_*.rs` 때문에 최초 실행이
시작 전 중단됐다. 저장소 절차의 `node scripts/rust-test-suite-manifest.mjs --prepare`로 파생 suite만
준비해 포맷을 검증했고, `tests/generated/`와 `tests/suites/manifest.json`은 확인 뒤 제거했다. source와
Cargo registry는 변경하지 않았다.

## 범위 경계와 다음 단계

- 빈 값·비숫자·범위 오류 표시와 `aria-invalid`는 #6109에 남겼다.
- 쪽 배치와 배율의 원자 view-settings transaction도 #6109에 남겼다.
- 핀치/슬라이더 preview·render scale·페이지 LRU는 #6040·#6041·#6042 범위다.
- 반응형 눈금자 정책과 resize 깜빡임은 #6187 범위다.
- Stage 3 결과 승인 뒤 4개 커밋을 최신 `upstream/devel@1a43a507c` 위로 rebase했다. 충돌은 양쪽에서
  생성한 `mydocs/orders/20260828.md` 한 파일뿐이었고, devel의 CI·#4969 기록과 #6108·#6109 행을 모두
  보존했다.
- 통합된 exact head `4912c9024`에서 Studio 전체 test·production build·Chrome E2E 28/28을 다시 통과했다.
- 게시 승인 뒤 공식 `gh-stack`에 기존 #6108·#6109 브랜치를 등록하고 최신
  `upstream/devel@1fe0d1480` 위로 cascading rebase했다. 정확한 #6108 하단 head `c6d9063bc`에서
  Studio 1,225건(1,224 pass·1 skip), production build 238 modules, Chrome E2E 28/28을 재통과했다.
- #6108을 bottom, #6109를 top으로 하는 native stacked PR의 로컬 분기 조건을 충족했다.
- remote push·PR 생성은 아직 수행하지 않았다.
