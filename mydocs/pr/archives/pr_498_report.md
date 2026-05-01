# PR #498 처리 보고서

**제목**: test: add canvas visual diff pipeline
**작성자**: seo-rii (Seohyun Lee)
**처리 결과**: cherry-pick 머지 (7 commits)

## 처리 본질

[PR #456](https://github.com/edwardkim/rhwp/pull/456) (Canvas → PageLayerTree replay 전환 P2) 의 후속 P3 검증 레이어 — **legacy Canvas vs PageLayerTree replay Canvas 픽셀 diff 테스트** + GitHub Actions Render Diff workflow 추가. 렌더링 동작 변경 아닌 회귀 검증 도구.

## cherry-pick 정합

| Commit (head) | Commit (cherry-picked) | 영역 |
|---------------|------------------------|------|
| `e3c59f7` | (preserved) | test: add canvas visual diff e2e — E2E 본체 |
| `2ae8246` | (preserved) | test: improve canvas render diff diagnostics |
| `fd812a5` | (preserved) | docs: document canvas render diff workflow |
| `9895f16` | (preserved) | test: add render diff ci runner |
| `7a1a2d0` | (preserved) | fix: harden render diff security |
| `8f27193` | (preserved) | fix: allow studio root in render diff server |
| `ac7a3b7` | (preserved) | fix: harden render diff fixture handling |

7 commits 모두 author 보존 (seorii) + 단계 commit 분리 보존.

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

→ **모두 JS E2E + CI workflow + 문서 + Vite 설정**. **Rust 변경 0**.

## 검증 게이트

| 검증 | 결과 |
|------|------|
| cargo test --lib | **1102 passed** ✓ |
| cargo test --test svg_snapshot | **6/6** ✓ |
| cargo test --test issue_418 | **1/1** ✓ |
| cargo test --test issue_501 | **PASS** ✓ (Task #501 회귀 0) |
| cargo clippy --lib -- -D warnings | **0건** ✓ |

WASM 빌드는 Rust 변경 없으므로 영향 없음 (PR CI 의 WASM Build = skipping 정합).

## 영향 영역

- `src/renderer/` 변경 0 — 본 사이클 정정 (Task #501 + PR #478 7 Task) 영역과 분리
- 본 PR 머지 후 `Canvas visual diff` CI job 자동 실행 — 향후 PR 의 Canvas parity 회귀 자동 검출
- 작성자 본 PR 검증: 기본 fixture 3개 (KTX, biz_plan, tac-case-001) 모두 0 diff

## 머지

- 머지 commit: `7a55510`
- devel push 완료
- 이슈 #364 (PageLayerTree 공식 API, CLOSED) 후속 P3

## 작업지시자 후속 정책 — Skia 별도 브랜치

작업지시자 통찰: **메인테이너가 ios/devel 처럼 skia 쪽 렌더러도 별도 브랜치로 생성해서 위험도를 낮추는 방법을 고려 중**.

본 정책 정합:
- ios/devel 브랜치 (맥북 전용) 처럼 skia/devel (또는 유사 명명) 별도 브랜치 운영
- 본 devel 의 Rust 본질 영역 (parser/renderer/serializer) 와 격리
- skia / CanvasKit / ThorVG / CoreGraphics 등 독립 backend 실험 가능
- 위험도 낮추기 — 본 devel 의 렌더링 안정성 + skia 영역 자유로운 실험 동시 보장

## PR #498 본 PR 의 정합

- 본 PR 은 **검증 도구** 추가 — Skia/CanvasKit 같은 렌더러 구현 아님
- legacy Canvas (PageRenderTree) ↔ layer Canvas (PageLayerTree replay) 두 path 의 parity 검증
- PageLayerTree 가 다른 backend (Skia 등) 의 입력으로 사용될 때, 본 검증 인프라가 backend 별 회귀 검출에 활용 가능

## 다음 단계

- PR #498 댓글 + close
- Skia 등 별도 backend 실험 시 별도 브랜치 운영 (작업지시자 정책)

## 메모리 룰 정합

- `feedback_pr_comment_tone` — 차분한 사실 중심
