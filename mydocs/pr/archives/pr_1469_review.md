# PR #1469 처리 보고서 — CanvasKit replay 계약 가드 확장

- PR: https://github.com/edwardkim/rhwp/pull/1469
- 제목: `render: expand CanvasKit replay contract guards`
- 작성자: seo-rii (collaborator)
- 연결: Refs #536 (멀티 렌더러 트래킹, close 안 함)
- base ← head: `devel` ← `seo-rii:render-p29`
- 처리일: 2026-06-22

## 1. 처리 결정

**admin merge.** P28(#1447) 후속으로 CanvasKit `renderNode` 계약 가드를 확장. 런타임 렌더링
동작 무변경(테스트·CI 가드만)으로 저위험. CI 통과 + 충돌 0건.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `.github/workflows/render-diff.yml` | CI 에 `node --check` + `npm run e2e:renderer-contract` 단계 추가 |
| `rhwp-studio/e2e/renderer-contract.test.mjs` | renderNode group/clipRect/leaf dispatch, rect/ellipse/line/path/form replay, glyphOutline color-layer 가드 확장 |

## 3. 검증

| 항목 | 결과 |
|---|---|
| GitHub CI | 6 pass |
| 충돌 시뮬레이션 | 0건 |
| `node --check renderer-contract.test.mjs` | OK |
| `node e2e/renderer-contract.test.mjs` | renderer backend contract guard passed |

## 4. 판단

런타임 무변경 + 계약 가드 강화 → 향후 CanvasKit 직접 replay 작업이 fallback 동작을
실수로 바꾸는 것을 방지. Non-goals 에 "런타임 렌더링/공개 Canvas·native Skia·PDF export 경로
무변경" 명시. merge 적절.
