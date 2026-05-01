# PR #498 검토 문서

**제목**: test: add canvas visual diff pipeline
**작성자**: seo-rii (Seohyun Lee)
**Base/Head**: `devel ← render-p3`
**상태**: OPEN, MERGEABLE (BEHIND — devel 추적 필요)
**규모**: +687 / -46, **7 commits**
**관련 이슈**: [#364](https://github.com/edwardkim/rhwp/issues/364) (CLOSED) — PageLayerTree 공식 API 노출

## PR 본질

[PR #456](https://github.com/edwardkim/rhwp/pull/456) (Canvas → PageLayerTree replay 전환 P2) 의 **후속 검증 레이어** — rhwp-studio E2E 에 **legacy Canvas vs PageLayerTree replay Canvas 픽셀 diff 테스트** + GitHub Actions Render Diff workflow 추가.

본 PR 은 **렌더링 동작 변경 아님** — P2 전환의 회귀 검증 도구만 추가.

## 변경 영역 (9 파일)

| 영역 | 파일 |
|------|------|
| **CI workflow (신규)** | `.github/workflows/render-diff.yml` |
| **E2E 테스트 (신규)** | `rhwp-studio/e2e/canvas-render-diff.test.mjs` |
| **CI runner (신규)** | `rhwp-studio/e2e/run-render-diff.mjs` |
| E2E 헬퍼 | `rhwp-studio/e2e/helpers.mjs` |
| Vite 설정 | `rhwp-studio/vite.config.ts` |
| 패키지 의존성 | `rhwp-studio/package.json` + `package-lock.json` |
| 문서 | `README.md` + `README_EN.md` |

→ **모두 JS E2E + CI workflow + 문서 + Vite 설정**. **소스 코드 (Rust / 렌더러) 변경 없음**.

## 7 commits 정합

| Commit | 영역 |
|--------|------|
| `e3c59f7` | test: add canvas visual diff e2e (E2E 테스트 본체) |
| `2ae8246` | test: improve canvas render diff diagnostics (진단 출력 보강) |
| `fd812a5` | docs: document canvas render diff workflow (README) |
| `9895f16` | test: add render diff ci runner (CI runner 추가) |
| `7a1a2d0` | fix: harden render diff security (보안 hardening 1) |
| `211f75c` | fix: allow studio root in render diff server (studio root 경로 허용) |
| `f1a73b3` | fix: harden render diff fixture handling (보안 hardening 2 — fixture 처리) |

→ 단계별 commit 분리 (test + docs + CI + 보안 hardening).

## 검증 정합 (작성자 보고)

- 기본 fixture 3개 (`basic/KTX.hwp`, `biz_plan.hwp`, `tac-case-001.hwp`) 모두 **0 diff**
- `wasm-pack build --target web --dev` 통과
- `CHROME_PATH=... npm run e2e:render-diff:ci` 통과
- `cargo test` / `cargo clippy` 미실행 (사유: 본 PR 직접 변경 범위가 JS E2E + CI workflow)

## CI 결과 (PR #498)

| 검증 | 결과 |
|------|------|
| Analyze (javascript-typescript) | ✅ pass |
| Analyze (python) | ✅ pass |
| Analyze (rust) | ✅ pass |
| Build & Test | ✅ pass |
| **Canvas visual diff** | ✅ pass (본 PR 추가 검증) |
| CodeQL | ✅ pass |
| WASM Build | skipping (정합 — Rust 변경 없음) |

## 영역 충돌 점검

| 영역 | 본 사이클 | PR #498 |
|------|----------|---------|
| `src/renderer/` | Task #501 (cell.padding) + PR #478 (#488 #490 #483 #489 #495 #480 #476) | **변경 없음** |
| `tests/` (Rust) | issue_501.rs 추가 + golden_svg 갱신 | **변경 없음** |
| `rhwp-studio/e2e/` | (PR #456 P2 머지) | E2E 신규 (canvas-render-diff) |
| `.github/workflows/` | — | **render-diff.yml 신규** |

→ **영역 충돌 0** — 본 사이클 정정 영역과 본 PR 변경 영역이 분리됨.

## 처리 옵션

| 옵션 | 진행 |
|------|------|
| A. **cherry-pick 머지 (7 commits 분리)** | 작성자 author 보존 + 단계 commit 보존 + 본 프로젝트 cherry-pick 패턴 정합 |
| B. squash 머지 | 단일 commit 통합 — author 보존되나 단계 commit 정보 손실 |
| C. merge 머지 (PR commits 그대로) | author + 단계 commit 보존, devel 에 head 분기 commit 그대로 — 본 프로젝트 패턴 외 |

## 권장 — 옵션 A (cherry-pick 7 commits)

본 프로젝트의 cherry-pick 패턴 정합 + 작성자 author 보존 + 7 commits 모두 의미 있는 단계 (test → diagnostics → docs → CI runner → 보안 hardening 3건).

## 검증 게이트 (머지 전)

- BEHIND devel — cherry-pick path 로 처리 (PR base 추적 미필요)
- cargo test --lib (회귀 0)
- cargo test --test svg_snapshot (6/6)
- cargo test --test issue_418 (1/1)
- cargo test --test issue_501 (PASS — 본 사이클 정정 회귀 0)
- cargo clippy --lib -- -D warnings (0건)
- WASM 빌드 (Rust 변경 없으므로 영향 없음 예상)

## 메모리 룰 정합

- `feedback_pr_comment_tone` — 차분한 사실 중심
- 본 PR 은 회귀 검증 도구 추가 (P3) — 렌더링 변경 없음, 머지 영향 영역 좁음

## 후속 작업 가능성

- 본 PR 머지 후 CI 가 자동 실행되어 향후 PR 의 Canvas parity 회귀 자동 검출 가능
- 기본 fixture 3개 (KTX / biz_plan / tac-case-001) 외 추가 fixture 확장 가능

## 다음 단계

작업지시자 승인 후 cherry-pick 7 commits 진행 (충돌 점검 + 검증 + 머지 + push + PR 댓글 + close).
