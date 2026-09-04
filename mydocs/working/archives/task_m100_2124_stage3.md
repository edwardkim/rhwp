# Task M100 #2124 Stage 3 완료 보고 - public contract와 WASM boundary

- 이슈: #2124
- 단계: Stage 3 - public contract와 WASM JSON advisory snapshot
- 작성일: 2026-07-10
- 브랜치: `task2124-frontend-baseline`
- 기준 커밋: `upstream/devel` `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 선행 단계: `mydocs/working/task_m100_2124_stage2.md`

## 1. 완료 요약

frontend external/package/message/build surface와 Rust-owned WASM JSON boundary를 분리해 기록했다.
현재 동작 snapshot과 영구 guardrail을 구별했으며, generated `pkg/`를 source of truth로 취급하지 않는다.

## 2. 산출물

| 파일 | 내용 |
|------|------|
| `mydocs/tech/investigations/issue-2124/task_m100_2124_public_contract_snapshot.md` | editor, iframe, core, VS Code, browser extension, legacy web 계약 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_wasm_json_schema_snapshot.md` | Rust schema ownership과 frontend parse/stringify 경계 |
| `scripts/frontend-editor-embed.test.mjs` | editor package/message/byte contract test |
| `scripts/frontend-wasm-bindings.test.mjs` | Rust explicit export와 fresh declaration 일치 test |

## 3. 핵심 결정

| 항목 | 결정 |
|------|------|
| `@rhwp/editor` | zero-runtime-dependency와 iframe API 보존 |
| UI framework | #2023 v2 승인대로 프론트 전체에서 v1.0까지 도입 금지; 이후 별도 RFC에서 재론 |
| iframe message | method/envelope/byte shape 보존 |
| `postMessage('*')` | 영구 계약이 아니라 별도 설계·테스트가 필요한 보안 부채; #2124 동작 변경 없음 |
| WASM schema | Rust/core 소유, frontend는 typed consumer boundary만 소유 |
| generated `pkg/` | fresh build 검증물이며 수동 수정·독립 authority가 아님 |
| font path | #2124에서는 유지, #2125에서 별도 canonical 이동 검토 |

## 4. 검증과 잔여 gate

editor embed contract test 1개가 통과했다. 초기 기준선 검증(`782059d9`)의 repository Docker build 전
local ignored `pkg/rhwp.d.ts`는 Rust source보다 오래되어 explicit export 네 개가 빠졌고 binding test가
이를 정확히 탐지했다. Stage 4에서 fresh WASM을 생성한 뒤 같은 binding test, Studio build, VS Code
compile이 통과했다. 최종 `6f1bd284` 동기화에서도 fresh WASM과 consumer gate를 다시 통과했다. stale
generated output을 근거로 API 결함이나 별도 후속 이슈를 확정하지 않는다.
