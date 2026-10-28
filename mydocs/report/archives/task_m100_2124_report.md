# Task M100 #2124 리뷰 반영 보고 초안 - 프론트 Phase 0 baseline

- 이슈: #2124
- 상위 umbrella: #2022
- 선행 계획 이슈: #2023
- 선행 문서 PR: #2080
- 브랜치: `task2124-frontend-baseline`
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 작성일: 2026-07-10
- 상태: maintainer 최종 승인 / 최신 devel 재검증 후 merge

## 1. 현재 결론

#2124의 Phase 0 기준선 산출물과 local automated gate를 완료했다. PR #2174에서 maintainer가 네
설계 안건을 모두 승인하고 merge 전 경미 수정 두 건을 요청했다. 수정은 로컬 반영했지만 CI와 merge가
남아 있으므로 이슈 완료는 선언하지 않는다. 대형 모듈 해체, font path 이전, legacy `/web` 삭제,
runtime 기능·보안 정책 변경은 수행하지 않았다.

초기 draft의 판단을 다음처럼 보정했다.

- #1904 결산에서 최대 CC와 상위 20 합은 감소했지만 CC 총합은 11,435에서 11,701로 2.3% 증가해,
  복잡도 통이동을 개선으로 계상한 산식의 한계가 확인됐다. #2130이 공식화한 Total CC, 상위 20 합,
  CC>25 합과 함수별 diff를 frontend schema v2에도 반영하고 test 제외와 재현성 metadata를 추가했다.
- 프론트 전체 `SOLID 54/100`은 평가 단위와 reviewer calibration이 없어 공식 기준선으로 사용할 수
  없으므로 폐기하고 미채점 evidence 문서로 전환했다.
- v2의 CC>25 +1~2는 자동 허용치가 아니라 Max/LOC 감소, 단일 책임, gate 통과, 총량 공개와 해소
  anchor를 갖춘 reviewer 승인 예외로 명시했다. Phase 0 승인 예외는 0건이다.

## 2. 산출물과 상태

| #2124 산출물 | 근거 | 상태 |
|--------------|------|------|
| ESLint/SonarJS/TypeScript parser 기반 metrics 도구 | `scripts/frontend-metrics.mjs`, `scripts/frontend-metrics/` package/lock | 로컬 완료 |
| 제품 모집단·제외군과 산식 | `task_m100_2124_frontend_metrics_scope.md` | 로컬 완료 |
| schema v2 공식 snapshot | `mydocs/metrics/frontend/2026-07-11/` | 로컬 완료 |
| commit·환경·hash manifest | `task_m100_2124_baseline_manifest.md` | 로컬 완료 |
| editor/core/VS Code/extension public contract | `task_m100_2124_public_contract_snapshot.md` | 로컬 완료 |
| Rust-owned WASM JSON advisory | `task_m100_2124_wasm_json_schema_snapshot.md` | fresh binding 통과 |
| 36 fonts/license/consumer inventory | `task_m100_2124_font_inventory.md` | 로컬 완료 |
| extension CSP/WAR/security snapshot | `task_m100_2124_extension_security_snapshot.md` | 로컬 완료 |
| smoke manifest와 contract tests | `task_m100_2124_smoke_manifest.md`, `scripts/frontend-*.test.mjs` | local gate 통과 |
| SOLID review evidence | `task_m100_2124_frontend_solid_anchors.md` | 공식 점수 미채점 방침 maintainer 승인 |

## 3. 정량 기준선

| 항목 | 결과 |
|------|------:|
| 제품 파일 / code lines / functions | 203 / 67,155 / 4,728 |
| Total CC / 전체 Top 20 합 | 11,805 / 2,581 |
| CC>25 count / sum | 62 / 3,932 |
| CC>100 count / sum | 6 / 1,732 |
| Max CC | 453 |
| parse/fatal diagnostics | 0 |
| WOFF2 | 36 files / 22,630,940 bytes |

최대 hotspot은 `rhwp-studio/src/engine/input-handler-mouse.ts`의 `onClick`으로 CC 453, LOC 995다.
대형 파일과 hotspot은 검토 후보이며 그 자체가 SOLID 위반이나 분리 지시는 아니다.

## 4. 계약과 guardrail 보정

| 항목 | 판단 |
|------|------|
| `@rhwp/editor` | zero-runtime-dependency와 iframe API 보존 |
| UI framework | #2023 v2 승인대로 프론트 전체에서 v1.0까지 도입 금지; 이후 별도 RFC/umbrella에서 재론 |
| `postMessage('*')` | 현재 구현 snapshot이며 영구 public contract가 아님; origin/source 정책은 별도 설계·회귀 test 필요 |
| `pkg/` | ignored generated output이며 Rust source와 fresh build가 authority; 수동 수정 금지 |
| font | #2124에서 경로·파일을 변경하지 않음; #2125에서 canonical 이동 |
| legacy `/web` | 미사용으로 단정하지 않고 repository/build/doc dependency scan 후 별도 제거 단위 판단 |
| extension security | 현재 sender/URL/file policy를 보존하고 알려진 위험은 별도 scope로 검토 |

## 5. local automated gate

repository Docker service로 release WASM을 생성한 뒤 모든 consumer를 같은 output에서 검증했다.

| 검증 | 결과 |
|------|------|
| Docker fresh WASM + `wasm-opt` | PASS |
| binding/editor contract | 2 PASS |
| combined frontend contract/shared | 68 PASS |
| Studio build / unit tests | PASS / 185 PASS |
| VS Code compile | PASS |
| Chrome/Firefox build | PASS |
| browser extension dist contract | 3 PASS |
| Studio renderer contract | PASS |
| metrics schema v2 self-compare | aggregate delta 0, function diff 0 |
| remote fallback | `upstream/devel`, `origin/devel`, ref 없음 3경로 PASS |

초기 기준선 검증(`782059d9`)의 build 전 stale declaration에는 Rust explicit export 네 개가 없었고
binding test가 이를 탐지했다. fresh output에서는 binding, Studio, VS Code gate가 모두 통과했다.
따라서 stale `pkg/`는 tracked source 결함으로 분리하지 않는다.

`ebf05268` 동기화에서는 metrics가 `782059d9` 이후 code lines +69, functions +5, Total CC +6을
포착했다. 최종 `6f1bd284` 동기화에서는 #2188의 Studio/Rust 변경을 반영해 code lines +32,
Total CC +14를 추가로 포착했고 Top 20 및 CC>25/100은 변하지 않았다. 두 시점 모두 upstream Rust
변경을 반영한 fresh WASM과 consumer gate를 다시 검증했다.

최종 dependency rebase 후 `npm ci` audit은 Studio 1 low, VS Code 1 high, metrics/Chrome/Firefox 0건이다.
VS Code 항목은 기존 dev-only 전이 경로 `copy-webpack-plugin → schema-utils → ajv → fast-uri@3.1.0`이며
#2174는 해당 package/lock을 변경하지 않는다. 자동 `npm audit fix`는 범위 밖이라 실행하지 않았고
별도 dependency 후속 후보로 분리한다.

## 6. maintainer 리뷰 반영

maintainer WSL2 재현에서도 당시 metrics 총량 3종 11,774 / 2,581 / 3,901과 함수별 자기 비교 delta 0이
확인됐다. stale `pkg/`에서는 binding gate가 `getStructure` 누락을 탐지했고, repo Docker fresh WASM
후 binding 1/1, extension-dist 3/3, editor-embed 1/1, Studio build와 185/185가 통과했다.

| 수정 요청 | 반영 |
|-----------|------|
| `upstream/devel` 하드코딩 | `upstream/devel` → `origin/devel` → 속성 생략 fallback 적용 |
| `orders/20260710.md` 충돌 | 최신 `upstream/devel` rebase, 원격 기록 유지 후 #2124 섹션 append |
| metrics 전용 `node_modules` 잔류 | `scripts/frontend-metrics/.gitignore`에 `node_modules/` 추가, `npm ci` 후 clean 상태 확인 |

dependency-only 전진분에서는 기존 snapshot을 보존했다. 최종 #2188 전진분은 실제 Studio source와
Rust policy를 바꾸므로 `6f1bd284`의 measured source clean 상태에서 공식 snapshot을 재생성했다.
직전 snapshot 비교에서 Total CC +14와 두 함수 diff를 확인한 뒤 새 기준선 자기 비교 delta 0을
확인했다. Rust 변경도 포함되므로 fresh WASM과 consumer gate를 다시 실행했다.

그 뒤 `acc841c9` 전진분은 `mydocs/manual/memory/`만 바꾼 문서 전용 커밋이었다. 최신 devel provenance를
맞추기 위해 공식 snapshot을 다시 생성했고 모든 정량 지표와 함수별 diff가 0-delta임을 확인했다.
frontend/Rust/package 입력이 변하지 않았으므로 `6f1bd284`에서 통과한 fresh WASM consumer gate는
재실행하지 않았다.

이후 `3077f96d`까지 #2184/#2191의 Studio CanvasKit과 Rust renderer/layout 변경이 전진했다. 공식
snapshot은 code lines +86, functions +3, Total CC +17, CC>25 count/sum +1/+31을 포착했고,
`renderTextRun` +11, `create` +3, `renderShapedScriptText` +2, 익명 함수 +1로 전부 설명됐다. 이를 최신
upstream 영점으로 재고정하고 fresh WASM과 consumer gate를 다시 실행했다.

원격 Chrome/Firefox/Studio Vite·TypeScript dependency update와 metrics 의존성이 함께 보존됐으며,
최종 head의 CI·merge 상태는 PR #2174 checks를 실시간 source of truth로 사용한다.

Studio/metrics/Chrome/Firefox/VS Code 다섯 패키지의 lockfile로 `npm ci`를 다시 실행했다. Studio 185
tests와 production build, VS Code compile, Chrome/Firefox production build, build 후 contract/shared
68건이 모두 통과했다.

TypeScript 7.0.2는 기존 compiler API를 기본 export하지 않고 `@typescript-eslint/parser@8.63.0`의
peer 범위도 `<6.1.0`이어서 Studio devDependencies 공유 방식에서는 metrics가 로드되지 않았다.
Studio의 TypeScript 7은 유지하고 분석 도구를 `scripts/frontend-metrics/` private package로 분리해
TypeScript 6.0.3을 고정했다. 제품/runtime dependency에는 영향을 주지 않는다.

## 7. 후속 이슈 판단

fresh WASM 기준으로 binding, Studio build, VS Code compile이 통과했으므로 build 결함 후속 이슈를
생성하지 않는다.

VS Code의 기존 `fast-uri` dev dependency audit은 runtime/build 결함과 분리해 dependency 후속 후보로
남긴다. baseline PR에서 자동 lockfile 변경으로 섞지 않는다.

E2E/render diff/browser install/VS Code Extension Host smoke는 #2124가 runtime·asset을 바꾸지 않으므로
완료 조건이 아니다. Phase A/B에서 실제로 영향을 받는 변경 유형의 gate로 선택한다.

#2188 시각 판정에서 확인된 NotoSansKR 서브셋의 U+25A0 계열 누락은 #2190으로 분리됐다. #2124는
font 파일을 변경하지 않으며, #2125에서 canonical 이전을 설계할 때 서브셋 재생성 규칙과의 접점을
검토한다.

## 8. 완료와 다음 단계 조건

| 작업 | 현재 판단 |
|------|-----------|
| local commits | 단계별 정리 완료 |
| PR | #2174 maintainer 승인. 최종 head gate 통과 후 collaborator merge |
| #2124 checklist/comment/close | PR merge와 reviewer 승인 뒤 별도 사용자 승인 필요 |
| #2022 tracking update | #2124 close 시 근거 링크와 함께 수행 |
| #2183 | #2124 close 후 frontend CI gate로 우선 진행 |
| #2187 | #2183 merge 후 최신 contract snapshot과 CI 기준으로 collaborator 리뷰 |
| #2125 | #2187 처리 후 착수하고 #2190 서브셋 규칙 연계 검토 |

#2124는 local implementation과 maintainer 요청 반영을 마쳤고 최종 CI와 merge를 기다리는 상태다.
