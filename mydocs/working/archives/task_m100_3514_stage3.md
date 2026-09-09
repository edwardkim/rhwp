---
kind: report
status: completed
canonical: mydocs/plans/archives/task_m100_3514.md
last_verified: 2026-08-20
---

# Task M100 #3514 Stage 3 — 전체 재검증·운영 문서

- Issue: [#3514](https://github.com/edwardkim/rhwp/issues/3514)
- Stage 1 commit: `081a44af9`
- Stage 2 commit: `e2b6ec723`
- 검증 기준: `codex/issue-3514-extension-smoke@e2b6ec723`

## 전체 검증

| 게이트 | 결과 |
|---|---|
| harness `node --check` | 통과 |
| Firefox production build | 통과 |
| Chrome·Firefox 확장 Node/dist 계약 | 85/85 통과 |
| Studio TypeScript `npx tsc --noEmit` | 통과 |
| Studio 전체 `npm test` | 1,033 통과, 1 skip, 실패 0 |
| Chrome production build + package smoke | 새 profile 10개, retry 없이 10/10 통과 |
| `git diff --check` | 통과 |

각 smoke 실행은 새 임시 Chrome profile·download 경로와 loopback server를 사용했다. 매회 실제
extension ID와 MV3 worker, HWP3 viewer canvas, 다크 자산, options, print surface, content script를
검사했으며 추가 탭과 page/worker 오류가 없었다.

## 문서 결과

- 확장 빌드·배포 매뉴얼에 단일 smoke 명령, 반복 옵션, 검증·비검증 경계를 추가했다.
- 최종 보고서에 Stage commit, 전체 검증과 남은 #3513·#3515 경계를 기록했다.
- Hyper-Waterfall 복구 문서에 Stage 1·2 커밋과 새 Stage 3 증적을 연결했다.

## 환경 경계

로컬 Docker daemon이 꺼져 표준 `docker compose ... wasm` 최적화 경로는 실행하지 못했다. 초기 환경
준비에서 개발 가이드의 네이티브 `wasm-pack build --target web --out-dir pkg --no-opt` fallback으로
현재 source의 WASM을 만들었다. Stage 2에는 Rust 변경이 없으며 Chrome·Firefox package는 이 동일
WASM을 사용했다. 최종 최적화 산출물은 Docker 사용 가능 환경 또는 release pipeline에서 재확인한다.

렌더링·레이아웃 구현 변경이 없어 PDF/SVG 시각 비교는 적용하지 않았다. 실제 HWP3 canvas 생성과
다크 자산 로드는 package smoke에서 직접 확인했다.

## Stage 3 승인 대상

- `mydocs/manual/chrome_edge_extension_build_deploy.md`
- `mydocs/feedback/task_m100_3514_hyper_waterfall_recovery.md`
- `mydocs/orders/20260820.md`
- `mydocs/plans/task_m100_3514.md`
- `mydocs/plans/task_m100_3514_impl.md`
- `mydocs/report/task_m100_3514_report.md`
- `mydocs/working/task_m100_3514_stage3.md`

Stage 3 승인 전에는 최종 문서 커밋을 진행하지 않는다. 승인·커밋 뒤에도 remote push와 draft PR
생성은 별도 GitHub 승인 경계로 유지한다.

## 승인 결과

작업지시자는 2026-08-20 22:26 KST에 “진행해줘”로 Stage 3 전체 검증과 문서 결과를 승인했다.
이 보고서를 포함한 최종 문서 커밋 뒤 remote push 승인 게이트로 이동한다.
