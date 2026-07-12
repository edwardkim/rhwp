# Task M100 #2124 Stage 4 완료 보고 - font, security, smoke, SOLID evidence

- 이슈: #2124
- 단계: Stage 4 - font, extension security, smoke, SOLID evidence
- 상태: 로컬 완료 / maintainer 승인
- 작성일: 2026-07-10
- 브랜치: `task2124-frontend-baseline`
- 기준 커밋: `upstream/devel` `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 선행 단계: `mydocs/working/task_m100_2124_stage3.md`

## 1. 완료 요약

font inventory, browser extension security snapshot, smoke manifest, frontend SOLID evidence 문서를
작성했다. repository Docker service로 fresh WASM을 생성한 뒤 binding, Studio, Chrome, Firefox,
VS Code package gate를 같은 output에서 검증했다.

## 2. 산출물

| 파일 | 내용 |
|------|------|
| `mydocs/tech/task_m100_2124_font_inventory.md` | 36 fonts, 정확한 hash/size 위치, license와 consumer 경로 |
| `mydocs/tech/task_m100_2124_extension_security_snapshot.md` | manifest/CSP/WAR/sender/URL/file 기준선과 잔여 위험 |
| `mydocs/tech/task_m100_2124_smoke_manifest.md` | local/manual gate와 변경 유형별 최소 검증 |
| `mydocs/tech/task_m100_2124_frontend_solid_anchors.md` | 미채점 SOLID evidence와 reviewer 채점 절차 |
| `scripts/frontend-extension-dist.test.mjs` | browser extension dist 계약 test |

## 3. 핵심 판단

| 항목 | 판단 |
|------|------|
| font | #2124에서 asset을 변경하지 않으며 #2125가 canonical 이동을 담당한다. legacy `/web` 정리는 dependency scan 후 별도 단위로 판단한다. |
| security | 기존 policy를 snapshot으로 보존한다. `postMessage('*')`와 origin/source 미검증은 영구 계약이 아니라 별도 설계가 필요한 부채다. |
| generated WASM | `pkg/`는 source of truth가 아니다. 초기 pre-build stale 상태는 binding test가 탐지했고, 최종 `6f1bd284` fresh Docker build 후 모든 consumer gate가 통과했다. |
| SOLID | 초기 `54/100`을 폐기했다. 공식 점수는 구체 review unit과 maintainer/collaborator calibration 후에만 기록한다. |
| Phase A | #2125는 #2124 reviewer 승인 뒤 시작한다. extension 또는 giant handler가 이를 앞서지 않는다. |

## 4. fresh WASM 검증

```bash
docker-compose --env-file .env.docker run --rm wasm
node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs
npm --prefix rhwp-studio run build
npm --prefix rhwp-studio run test
npm --prefix rhwp-vscode run compile
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
node --test scripts/frontend-extension-dist.test.mjs
```

결과:

- Docker release WASM + `wasm-opt`: PASS.
- binding/editor: 2 tests PASS.
- combined frontend contract/shared: 68 tests PASS.
- Studio build와 185 unit tests: PASS.
- VS Code compile: PASS.
- Chrome/Firefox build와 extension dist 3 tests: PASS.

초기 기준선 검증(`782059d9`)의 build 전 stale declaration은 Rust explicit export 네 개가 빠져 있었고
binding test가 이를 탐지했다. fresh output에서 동일 test와 기존 실패 지점이 모두 통과했다. 최종
`6f1bd284` 동기화에서는 pre-build binding도 통과했으며 upstream Rust 변경을 반영한 fresh output으로
모든 consumer gate를 재실행했다. 별도 source 결함으로 분리하지 않는다.

## 5. 미실행 수동 검증

브라우저 설치, Studio E2E/render diff, VS Code Extension Host smoke는 runtime·asset 변경 PR에서 선택할
release/manual gate다. #2124는 기준선 작업이므로 Stage 4 완료 조건으로 승격하지 않는다.

## 6. 다음 단계

local Stage 4 gate는 완료했다. draft PR에서 maintainer/collaborator에게 metrics 산식, SOLID 미채점 판단,
contract와 smoke 분류를 리뷰 요청한다. 승인 전에는 Stage 5 완료, #2124 체크리스트·close, #2022 완료
반영, #2125 착수를 수행하지 않는다.
