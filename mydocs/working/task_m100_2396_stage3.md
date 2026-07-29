# Task #2396 Stage 3 완료 보고 — 전체 frontend 검증

- 이슈: #2396
- 브랜치: `codex/issue-2396-custom-scheme-legacy-request`
- 검증 head: `11af6aa2`
- 선행 단계: `mydocs/working/task_m100_2396_stage2.md`

## 단계 범위

Stage 3는 코드 변경 없이 전체 frontend test, 현재 브랜치 소스의 dev WASM 재생성, Studio production
build와 Git 추적 범위를 검증했다.

## 검증 결과

| 게이트 | 결과 |
|---|---|
| `npm --prefix rhwp-studio test` | 364/364 PASS |
| focused embed protocol 포함 | 14/14 PASS |
| `wasm-pack build --target web --out-dir pkg --dev` | PASS |
| `npm --prefix rhwp-studio run build` | PASS |
| `git diff --check` | PASS |
| `git diff upstream/devel...HEAD --check` | PASS |

전체 Node test에는 `rhwp-studio/tests/*.test.ts`와 `npm/editor/tests/*.test.mjs`가 포함된다. 신규
custom scheme positive/negative 계약과 기존 MessageChannel, legacy fallback, binary ownership,
cleanup 계약이 함께 통과했다.

## build 관찰

- dev WASM은 현재 브랜치에서 다시 생성했으며 `pkg/rhwp.js`와 `pkg/rhwp_bg.wasm`을 포함한 package
  생성이 완료됐다.
- Studio TypeScript compile과 Vite production build가 통과했다.
- Vite의 CanvasKit `fs`/`path` browser externalization 및 500kB 초과 chunk 경고는 비차단 경고였다.
- wasm-pack은 현재 플랫폼의 prebuilt `wasm-bindgen` 부재를 알리고 fallback 경로를 사용했지만 build는
  정상 완료했다.

## 추적 파일 범위

다음 생성물은 모두 gitignored이며 Stage 3 커밋에 포함하지 않는다.

- `pkg/`
- `target/`
- `rhwp-studio/node_modules/`
- `rhwp-studio/dist/`

Stage 3의 tracked 변경은 오늘할일 상태와 이 완료보고서뿐이다. Stage 1 테스트와 Stage 2 runtime 구현에는
추가 diff가 없다.

## 잔여 검증

실제 macOS WKWebView custom scheme downstream representative suite는 이 저장소 환경에서 실행하지 않았다.
이슈 작성자가 제시한 downstream 진단 결과와 저장소 unit/runtime 계약을 근거로 하며, 최종 보고서에 이
제한을 명시한다.

## 다음 승인 게이트

Stage 4에서는 최종 보고서, 오늘할일 완료 상태와 PR 본문 초안을 작성한다. push와 PR 생성은 별도 승인을
받기 전에는 실행하지 않는다. 작업지시자 승인 전에는 Stage 4를 시작하지 않는다.
