# 작업 기록 — task_m100_3790 Stage 5B frontend-only canary

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **브랜치**: `issue-3790-stage5b-codeql-canary`
- **worktree**: `tmp/issue-3790-stage5b-canary`
- **최초 기준**: `upstream/devel` `c64b5c70a700` (#4519 merge)
- **재개 기준**: `upstream/devel` `525cf8e8ed9f` (#4565 merge), 동기화 merge `cec04e66a`
- **상태**: Stage 5B selective/full canary 실측 완료, 측정 전용 PR #4573 close 준비

## 목적과 종료 조건

Stage 5B가 활성화된 첫 frontend-only PR에서 CodeQL 언어 선택 진리표와 절감량을 실측한다. 이 PR은 제품
반영 대상이 아닌 measurement-only canary다. selective/full 결과를 같은 head SHA에서 확보해 #3790과
canary PR에 기록한 뒤 merge하지 않고 close한다.

## 변경 형태

- Stage 3 PR #3951과 Stage 4 PR #4078이 사용한 것과 같은
  `rhwp-studio/src/command/shortcut-map.ts` + `rhwp-studio/tests/shortcut-map.test.ts` 변경을 재사용한다.
- 단축키 정의와 mapping entry를 TypeScript 읽기 전용 계약으로 좁히고, 개체 속성 `P` 단축키의
  영문·한글·IME pending 입력 회귀를 추가한다.
- 계획·오늘할일·이 작업 기록은 `mydocs/**` review-only 경로다. 이번 측정의 주 대조군은 과거 canary가
  아니라 같은 새 head SHA의 manual full run이므로 문서 동반 여부가 CodeQL 절감량 해석을 바꾸지 않는다.

## classifier v2 기대값

| 출력 | 기대값 |
| --- | --- |
| `classification_status` | `classified` |
| `classifier_version` | `2` |
| `frontend_mode` | `unit` |
| `render_required` | `false` |
| `rust_required` | `false` |
| `native_skia_required` | `false` |
| `codeql_languages` | `javascript-typescript` |
| `reason` | `classified:studio-unit` |

## 원격 selective/full 진리표

일반 PR run에서는 Frontend unit과 JavaScript/TypeScript CodeQL만 실제 실행한다. Frontend package,
Canvas visual diff, Rust lint·세 builder·네 worker와 Native Skia는 skip되어야 한다. Python·Rust
`Analyze (...)` job은 check identity를 유지하되 실제 checkout·init·analysis 없이 no-op success여야 하며,
`Build & Test`는 success여야 한다. GHAS `CodeQL`은 선택되지 않은 configuration을 찾지 못했다는
`neutral` summary를 낼 수 있으므로 세 `Analyze (...)` job과 분리해 해석하고, failure는 허용하지 않는다.

같은 head SHA에서 CI·CodeQL·Render Diff를 `workflow_dispatch`하면 full 경로가 되어 package·Canvas·Rust·
Native Skia와 세 CodeQL 언어를 실제 실행해야 한다. 두 실행의 job duration 합계와 workflow wall time을
비교해 Stage 5B CodeQL 절감량과 Stage 3~5 전체 최종 절감량을 분리해 기록한다.

## 기존 head 결과와 재개 이유

최초 head `6fb1bd77b`의 PR selective는 CI run `31474292794`, CodeQL run `31474292480`, Render Diff
run `31474292528`에서 모두 통과했다. 같은 SHA의 수동 full은 CodeQL run `31474845857`과 Render Diff
run `31474848038`이 통과했지만, CI run `31474843602`는 #4029의 기존 cold `release/30` 제한으로
archive builder가 완료되기 전에 취소됐다.

#4029는 PR #4581과 v0.8.4 태그 CI로 해결됐다. 일반 수동 CI는 이제 `release-test/30`, main/tag는
`release/60`을 선택한다. 또한 PR #4573의 base가 `c64b5c70a700`에 머문 사이 `devel`이 전진해 PR이
`CONFLICTING / DIRTY`가 됐다. 기존 측정 SHA를 재작성하지 않도록 최신 `upstream/devel@525cf8e8e`을
merge하고, 유일한 충돌 `mydocs/orders/20260811.md`는 canary 행과 upstream 운영 기록을 모두 보존해
해소했다. 새 head에서는 focused 검증과 PR selective, 같은 SHA의 수동 full을 다시 수행한다.

## 최초 head focused 검증 결과

- `node --test rhwp-studio/tests/shortcut-map.test.ts` — 7/7 통과.
- `npx --prefix rhwp-studio tsc --project rhwp-studio/tsconfig.ci-unit.json --noEmit` — 통과.
- `npm --prefix rhwp-studio test` — 837건 중 836 pass, 정책 skip 1, fail 0.
- 실제 변경 6개 파일 목록으로 `node scripts/ci-impact-classifier.cjs --input <canary-file-list.json>`을
  실행해 classifier v2 기대값 전체가 일치했다.
- `git diff --check` — 통과.

새 worktree의 `npm --prefix rhwp-studio ci`는 lockfile 기준 설치를 완료했으며 audit는 기존 의존성의
low 1건·high 3건을 보고했다. manifest·lockfile 변경은 없고 측정 canary 범위가 아니므로 자동 수정하지
않는다.

## 재개 head focused 검증 결과

- `npm --prefix rhwp-studio ci` — 최신 lockfile 기준 389 packages 설치 완료.
- `node --test rhwp-studio/tests/shortcut-map.test.ts` — 7/7 통과.
- `npx --prefix rhwp-studio tsc --project rhwp-studio/tsconfig.ci-unit.json --noEmit` — 통과.
- `npm --prefix rhwp-studio test` — 860건 중 859 pass, 정책 skip 1, fail 0.
- 실제 변경 7개 파일 목록으로 classifier를 실행해
  `classified:studio-unit`, `frontend_mode=unit`, `codeql_languages=javascript-typescript`,
  Rust·render·Native Skia false가 모두 일치했다.
- `node --test scripts/tests/ci-impact-classifier.test.cjs` — 28/28 통과.
- CI·CodeQL·Render Diff workflow 계약 unittest — 45/45 통과.
- `actionlint` (`ci.yml`, `codeql.yml`, `render-diff.yml`) — 통과.
- `git diff --check` — 통과.

제품 소스 변경은 `ShortcutDef`와 shortcut entry를 readonly 타입으로 좁히는 정적 계약이며 런타임 매핑을
바꾸지 않는다. 새 테스트도 기존 개체 속성 `P` 매핑의 영문·한글·IME 경계를 고정한다. 실제 브라우저
동작·Cargo·WASM·renderer·fixture는 바꾸지 않으므로 브라우저 E2E, Cargo, wasm-pack과 시각 검증은
focused 범위에서 생략한다.

## 첫 재개 push 판정 — fast-pass 표본 배제

head `a5e45e571`의 CI run `31581207834`와 CodeQL run `31581207609`는 성공했지만 새 selective를
실행하지 않았다. 두 preflight 모두 최신 devel 동기화 merge `cec04e66a`를 current-base update bridge로
인식하고, 그 conflict resolution이 `mydocs`에만 있음을 검증한 뒤 이전 code candidate `6fb1bd77b`의
green 결과를 재사용했다.

- CI: `fast_pass=pending-base-merge-tree`, candidate `6fb1bd77b`, merge-tree reason
  `current-base-merge-resolution-mydocs-only`; Frontend unit을 포함한 worker 전부 skipped.
- CodeQL: 같은 candidate의 `codeql-checks-green`을 재사용해 세 Analyze lane 전부 skipped.
- Render Diff run `31581207582`: 이전 candidate의 Canvas가 skipped라 재사용하지 않고 현재 diff를
  분류했으며 `render_required=false`, Canvas skipped로 성공.

따라서 이 head의 약 2분 wall time은 Stage 5B 절감량 표본에서 제외한다. 새 head를 genuine Studio unit
candidate로 만들기 위해 기존 `P` 회귀에 macOS `Meta+P`가 `file:print`로 유지되는 경계를 추가한다.
최신 commit에 non-review-only test 파일이 포함되므로 다음 PR run은 이전 candidate를 fast-pass할 수 없고,
trusted classifier의 `frontend_mode=unit`, `codeql_languages=javascript-typescript`를 실제 실행해야 한다.

### selective 강제 보정 focused 검증

- `node --test rhwp-studio/tests/shortcut-map.test.ts` — 7/7 통과.
- `npx --prefix rhwp-studio tsc --project rhwp-studio/tsconfig.ci-unit.json --noEmit` — 통과.
- `npm --prefix rhwp-studio test` — 860건 중 859 pass, 정책 skip 1, fail 0.
- 실제 PR 변경 7개 파일 목록으로 classifier를 실행해 `frontend_mode=unit`,
  `codeql_languages=javascript-typescript`, Rust·render·Native Skia false가 모두 일치했다.
- `git diff --check` — 통과.

이 보정 commit을 push한 뒤 canary 측정이 끝날 때까지 후행 docs-only commit을 만들지 않았다. 따라서
Studio test 변경이 최신 head에 포함된 상태에서 PR selective가 실제 Frontend unit과
JavaScript/TypeScript CodeQL을 실행했다.

## 최종 canary 실측 — head `7e5216f70`

첫 재개 push의 fast-pass 표본을 제외하고, non-review-only test 변경이 포함된 같은 head
`7e5216f709bd3b8ca037cf5c5f3f60f7dbf21810`에서 PR selective와 수동 full을 대조했다. 여섯 workflow는
모두 성공했다.

| workflow | selective run | selective wall / runner | full run | full wall / runner |
| --- | --- | ---: | --- | ---: |
| CI | [31582691020](https://github.com/edwardkim/rhwp/actions/runs/31582691020) | 95초 / 76초 | [31583545091](https://github.com/edwardkim/rhwp/actions/runs/31583545091) | 1,068초 / 3,562초 |
| CodeQL | [31582690810](https://github.com/edwardkim/rhwp/actions/runs/31582690810) | 158초 / 164초 | [31583557284](https://github.com/edwardkim/rhwp/actions/runs/31583557284) | 644초 / 889초 |
| Render Diff | [31582690807](https://github.com/edwardkim/rhwp/actions/runs/31582690807) | 13초 / 9초 | [31583566849](https://github.com/edwardkim/rhwp/actions/runs/31583566849) | 377초 / 369초 |

세 workflow job elapsed 합계는 full 4,820초에서 selective 249초로 **4,571초(94.8%) 감소**했다. workflow가
병렬로 실행된 실제 완료시간은 1,068초에서 158초로 **910초(85.2%) 감소**했다. Stage 5B가 직접 줄인
CodeQL job elapsed는 889초에서 164초로 **725초(81.6%) 감소**했고 CodeQL wall time은 644초에서
158초로 **486초(75.5%) 감소**했다.

Stage 4 canary의 서로 다른 당시 head·runner 표본 939초 runner / 575초 wall과 비교하면 최종 selective는
249초 / 158초지만, 이 값은 테스트 수와 runner 시점이 다르므로 참고값이다. Stage 5B의 권위 절감률은 위의
같은-head selective/full 대조다.

### 선택 실행 진리표

- CI preflight는 `fast_pass=false`, `classified:studio-unit`, `frontend_mode=unit`, Rust·render·Native
  Skia false를 냈다. Frontend unit은 860건 중 859 pass·정책 skip 1로 성공했고 package·Rust·Native
  Skia·Canvas는 기대대로 skipped, `Build & Test`는 success였다.
- CodeQL은 JavaScript/TypeScript만 실제 checkout·init·analysis를 수행해 144초에 성공했다. Python과
  Rust는 check identity를 유지한 6초 no-op success였다.
- GHAS `CodeQL`은 selective 직후 Python·Rust configuration 부재를 알리는 `neutral` summary였고,
  같은 SHA의 수동 세 언어 full 분석 뒤 `success`와 `No new alerts`로 갱신됐다.

### 수동 full 검증

- CI는 `release_grade=false`의 `release-test/30` 정책으로 WASM, lint, frontend package, Native Skia,
  세 archive builder와 slow/1/2/3 worker를 모두 실행했다. shard 합계는
  `3966 + 909 + 906 + 1 = 5782`로 expected 5,782와 일치했고 `Build & Test`가 성공했다.
- CodeQL은 JavaScript/TypeScript 150초, Python 104초, Rust 629초로 세 언어 모두 실제 분석에 성공했다.
- Render Diff는 Canvas 3/3, direct PDF compatibility 3/3, CanvasKit readiness 8/8에 성공했다. 기준 이미지와
  PDF raster의 크기·픽셀 차이 4쪽은 비차단 warning이며 error와 direct compatibility failure는 0건이다.

## 판정과 후속

Stage 5B의 trusted-base 언어 선택, 고정 세 Analyze check identity, 선택되지 않은 lane의 no-op success와
비-PR full fallback이 원격에서 모두 확인됐다. 추가 workflow 보정 없이 canary gate를 통과했다.

PR #4573은 제품 반영 대상이 아닌 measurement-only canary이므로 이 기록을 trailing review-only commit으로
보존한 뒤 **merge하지 않고 close**한다. #3790은 계속 열어 두고 다음 단계로 보존 중인 Stage 2.6 controller를
현재 Stage 3~5 진리표에 맞게 축소·재검증하는 후속 enforcement를 진행한다. controller 유일본은 대체
controller의 main 등록·live audit 또는 maintainer의 policy 미채택 결정 전까지 정리하지 않는다.
