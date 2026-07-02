# PR #1656 리뷰 — CanvasKit replay diagnostics 정합

- PR: #1656 `render: align CanvasKit replay diagnostics`
- 작성자: @seo-rii
- 기준 브랜치: `devel`
- PR head: `f7632d306b6df86ad2407b9f920874875b963af4` (문서 작성 시점 참고값)
- merge commit: `220923692bdb60b8c9c3537ac3a07321eb58c88e`
- 규모: 4 files, +180/-25
- 관련 이슈: #536
- 처리 결과: GitHub Actions 통과 후 admin merge 완료
- 후속 코멘트: https://github.com/edwardkim/rhwp/pull/1656#issuecomment-4864959740
- 관련 이슈 처리: #536은 전체 멀티 렌더러/CanvasKit parity 트래킹 이슈라 open 유지

## 변경 요약

CanvasKit replay plan의 Rust 진단 결과와 Studio CanvasKit runtime이 실제로 직접 그리는 branch를 맞추는 PR이다.

핵심 변경:

- `src/renderer/canvaskit_policy.rs`
  - `footnoteMarker`를 direct replay로 분류
  - `formObject`, `placeholder`를 `basicStaticReplay` direct 항목으로 분류
  - plain rotated `TextRun`을 direct path에 남기고 `rotatedText` transition detail에서 제외
  - vertical/decoration/emphasis/outline/shadow/emboss/engrave/superscript/subscript/shade/ratio TextRun 효과는 policy-visible detail로 유지
- `rhwp-studio/src/view/canvaskit-renderer.ts`
  - `recordTextRunCoverageGaps`를 추가해 runtime에서 TextRun compatibility gap을 `unsupportedOps`에 기록
  - rotation은 기존 `canvas.rotate` direct replay path로 유지
- `rhwp-studio/src/core/types.ts`
  - LayerTextStyle에 TextRun diagnostics parity에 필요한 style 필드 추가
- `rhwp-studio/e2e/renderer-contract.test.mjs`
  - renderer contract guard에 TextRun diagnostics와 rotation replay drift 방지 assertion 추가

## 로컬 merge 검증

`upstream/devel`은 PR head `f7632d306b6df86ad2407b9f920874875b963af4`의 조상이다.

```bash
git merge-base --is-ancestor upstream/devel HEAD
```

결과: exit code 0. 작성 시점 기준 PR head는 최신 `devel`을 포함한다.

## 로컬 검증

새 PR review 지침에 따라 cargo 검증 전 `target` 하위 항목을 삭제한 뒤 수행했다.

- `cargo fmt --all -- --check`: 통과, real 2.47s
- `env CARGO_INCREMENTAL=0 cargo test renderer::canvaskit_policy::tests --lib`: 16 passed, real 38.20s
- `node --check e2e/renderer-contract.test.mjs`: 통과, real 0.05s
- `npm run e2e:renderer-contract`: 통과, real 0.22s
- `npm run build`: 통과, real 3.13s
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과, real 28.14s
- `git diff --check`: 통과

## GitHub Actions

최신 PR head `f7632d306b6df86ad2407b9f920874875b963af4` 기준:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- WASM Build: skipped
- Analyze (javascript-typescript): success
- Analyze (python): success
- Canvas visual diff: success
- CodeQL: success
- Analyze (rust): success
- Build & Test: success

GitHub Actions 완료 후 `MERGEABLE` + `CLEAN` 상태를 확인했다.

## 리뷰 결과

Blocking finding 없음.

소스 대조와 contract 검증 기준으로 Rust policy와 Studio runtime이 같은 방향으로 정리되어 있다. PR은 새로운 CanvasKit
렌더링 coverage를 크게 넓히는 변경이라기보다, Studio runtime에 이미 존재하는 direct branch와 Rust replay plan
diagnostics를 맞추는 변경이다. TextRun effect 계열은 화면 정밀 구현 완료가 아니라 `unsupportedOps`/policy detail에
남기는 진단 정합 작업으로 확인했다.

브라우저 수동 시각 검증은 수행하지 않았으며, 이번 판단은 코드 대조, `renderer-contract`, build, Rust unit/clippy 검증
범위다.

## 비차단 확인 사항

- `recordTextRunCoverageGaps`는 `renderTextRun`에서 실제 drawText 전에 호출되므로, CJK typeface 부재로 조기 return되는
  경우에도 TextRun 효과 gap은 먼저 기록된다. 이후 `textRunFont`가 추가로 기록되는 구조라 진단 중복은 의도된 것으로
  보인다.
- `LayerTextStyle`에 추가된 필드는 JSON emission(`src/paint/json.rs`)에서 이미 내려오는 이름과 맞는다.

## 최종 처리

최신 head 기준 GitHub Actions가 통과했고 `MERGEABLE` + `CLEAN` 상태를 확인한 뒤 admin merge 완료.

#536은 전체 remaining CanvasKit parity 범위를 닫지 않고 open 유지했다. PR 후속 코멘트에는 이번 PR이
diagnostics 정합 단계라는 점과 로컬/CI 검증 결과를 남겼다.
