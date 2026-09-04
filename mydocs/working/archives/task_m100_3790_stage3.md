# task_m100_3790 Stage 3 — frontend unit/package/render 활성화

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **브랜치**: `codex/issue-3790-stage3-frontend`
- **최종 동기화 기준**: `upstream/devel` `f864e851a98f`
- **상태**: PR #3943 merge 및 canary PR #3951 측정 완료
- **기록일**: 2026-08-04 KST

## 변경 요약

Stage 2.5까지 관찰만 하던 base-SHA classifier의 `frontend_mode`와 `render_required`를 실제 job 조건에
연결했다. Rust lint·세 builder·네 worker·Native Skia와 CodeQL 조건은 이 단계에서 바꾸지 않는다.

- `frontend_mode=unit`: Node 설치, Studio dependency 설치, 전체 `src` typecheck, Studio 전체 759개 test
- `frontend_mode=package`: 기존 fresh WASM, Studio test/build, shared·extension 검증
- `render_required=false`: Render Diff preflight와 분류 기록은 남기고 무거운 Canvas job만 skip
- aggregate `Build & Test`: `none=skipped/skipped`, `unit=success/skipped`,
  `package=skipped/success` 진리표를 강제
- 강제 full: 같은 SHA를 수동 `workflow_dispatch`로 실행하며 일반 label 변경은 workflow를 재시작하지 않음

## 분류 안전 경계

명확히 비렌더인 `rhwp-studio/src/command/**`, `src/engine/command.ts`와 일반 Studio test만 unit/no-render로
좁혔다. `src/view/**`, `src/ui/**`와 나머지 Studio runtime은 Vite·asset·plugin 오류를 놓치지 않도록
package+render로 유지한다. WASM binding을 직접 소비하거나 앱·embed/package 경계를 형성하는
`src/core/**`, `src/embed/**`, `src/main.ts`, `public/**`, `src/hwpctl/**`도 package lane을 사용한다.

unit lane은 fresh WASM compile을 피하기 위해 `tsconfig.ci-unit.json`에서 `@wasm/rhwp.js`만 최소 stub으로
치환한다. 이 때문에 WASM 경계 파일은 unit으로 분류하지 않으며, CI 전용 tsconfig와 stub 자체 변경도
`fail-closed:frontend-unit-contract`로 전체 검증한다. package lane의 실제 `npm run build`는 계속
`wasm-pack`이 생성한 `pkg/rhwp.d.ts`를 사용한다.

PR 리뷰에서 일반 label도 실행 중인 run을 취소하고 새 run을 만드는 문제가 확인돼 `labeled|unlabeled`
activity와 아직 존재하지 않는 `ci:full` label 계약을 제거했다. label 기반 강제 실행은 post-main trusted
controller 단계로 미루고, Stage 3 canary는 selective PR run과 같은 SHA의 수동 full run을 비교한다.

workflow·classifier·Cargo·WASM·rename·빈/경계 파일 목록·미분류 경로와 수집/실행 실패는 기존처럼 full로
닫는다. classifier는 PR head가 아니라 base SHA에서 credential 없이 sparse checkout한다. post-main
controller enforcement는 Stage 3~5 진리표 확정과 정상 devel→main 릴리즈 뒤의 별도 단계다.

## 검증

| 검증 | 결과 |
| --- | --- |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 24/24 통과 |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py scripts/tests/test_render_diff_workflow.py` | 14/14 통과 |
| `actionlint .github/workflows/ci.yml .github/workflows/render-diff.yml` | 통과 |
| `npm --prefix rhwp-studio run e2e:renderer-contract` | 통과 |
| `npx --prefix rhwp-studio tsc --project rhwp-studio/tsconfig.ci-unit.json --noEmit` | 통과 |
| `npm --prefix rhwp-studio run test` | 759/759 통과 |
| `wasm-pack build --target web --dev` → `npm --prefix rhwp-studio run build` | fresh WASM 선언 기반 Vite production build 통과 |
| `git diff --check` | 통과 |

전체 Rust·WASM package 검증은 Stage 3 PR 자체가 workflow/classifier 변경으로 full lane에 들어가므로 원격
CI에서 확인한다. 로컬에서는 Stage 3가 새로 추가한 unit lane과 분류·workflow 계약에 집중했다.

## 완료 및 canary 결과

PR #3943은 리뷰 보정과 최신 full CI를 통과해 devel에 merge됐다. 후속 frontend-only canary PR #3951은
`frontend_mode=unit`, `render_required=false`를 판정해 unit gate만 59초에 실행하고 package와 Canvas를
정확히 skip했다. 같은 SHA의 수동 full에서 package 2분 47초와 Canvas 5분 59초가 성공해 직접 runner
time 7분 47초 절감을 확인했다.

수동 full 전체는 기존 cold release archive 30분 timeout으로 완료되지 않았고, 실제 main push에서도
동일 증상이 확인돼 #4029로 분리했다. Stage 3 진리표와 절감효과는 성공한 동일 SHA frontend·Canvas
구간으로 확정했으며 canary PR은 제품 변경이 아니므로 merge 없이 close했다. 후속은 Stage 4
Rust lint·builder/worker·Native Skia 조건화다.
