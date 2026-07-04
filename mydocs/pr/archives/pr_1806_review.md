# PR #1806 리뷰 — CanvasKit object gap diagnostics 정렬

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1806 |
| 작성자 | @seo-rii |
| base / head | `devel` / `render-p32` |
| 작성 시점 규모 | 3 files, +95 / -6 |
| 작성 시점 참고 head | `aaba99a1b00869df8f44e62741c4ca767a3e170c` |
| 작성 시점 참고 상태 | `MERGEABLE`, `Build & Test` 진행 중 |
| merge 결과 | 2026-07-03 KST merge 완료, merge commit `9c25eb6288ceacc3b18ebe7beabdd7cbf91a1e6b` |
| reviewer assign | @jangster77 지정 완료 |

## 운영 메모

이 PR 은 원래 Draft 상태였고, 이번 검토에서는 메인터너가 수동으로 Ready for review 로 전환한 뒤 리뷰를
진행했다. merge 후 감사 코멘트에는 다음부터 빠른 검토를 원하면 처음부터 Ready for review 상태로 열어 달라는
요청을 정중히 포함한다.

## 변경 범위

- `src/renderer/canvaskit_policy.rs`
  - `PaintOp::Equation` / `PaintOp::RawSvg` gap item 에 `unsupportedDirectReplay` detail 을 부여한다.
  - equation/rawSvg object fragment gap 이 policy JSON 에서 같은 이유로 노출되는지 테스트를 추가한다.
- `rhwp-studio/src/view/canvaskit-renderer.ts`
  - Studio CanvasKit runtime 이 `equation:unsupportedDirectReplay`, `rawSvg:unsupportedDirectReplay` 를
    `unsupportedOps` 에 기록하도록 명시한다.
- `rhwp-studio/e2e/renderer-contract.test.mjs`
  - TextRun fallback 과 object fragment fallback 을 분리한다.
  - equation/rawSvg 가 단순 `op.type` fallback 으로 되돌아가지 않고 runtime diagnostic detail 을 유지하는지
    contract test 로 고정한다.

## PR 내용 기준 판단

이 PR 의 목적은 equation/rawSvg 를 CanvasKit 에서 직접 재생하는 것이 아니라, 아직 직접 재생하지 못하는
object fragment 계열의 gap 을 policy/runtime/contract test 에서 같은 어휘로 드러내는 것이다. 따라서 visual
sweep 대상은 아니며, renderer 출력 차이가 아니라 diagnostics contract 정합성을 검증 기준으로 본다.

검토 결과, PR 본문의 non-goal 과 구현이 일치한다. direct replay 구현은 추가하지 않았고,
`unsupportedDirectReplay` detail 을 policy 와 Studio runtime 양쪽에 맞춘 뒤 contract test 로 guard 했다.

## 로컬 검증

검토 브랜치: `local/pr1806-review`

- `rm -rf /Users/tsjang/rhwp/target/*`
- `git diff --check upstream/devel..HEAD` 통과
- `env CARGO_INCREMENTAL=0 cargo fmt --all -- --check` 통과
- `env CARGO_INCREMENTAL=0 cargo test renderer::canvaskit_policy::tests --lib` 통과
- `cd rhwp-studio && npm run e2e:renderer-contract` 통과
- `cd rhwp-studio && npm run build` 통과
  - Vite 의 `fs`/`path` externalize 및 chunk size warning 만 확인했고, 실패는 없음
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과

## GitHub 후속 처리

- 승인 리뷰: https://github.com/edwardkim/rhwp/pull/1806#pullrequestreview-4620825199
- 감사 코멘트: https://github.com/edwardkim/rhwp/pull/1806#issuecomment-4870033809
- PR 본문에 `Closes #...` 대상은 없어 별도 issue close 는 수행하지 않았다.

## 결론

PR 내용 기준으로 merge 후보로 판단했고, 최신 PR head 기준 GitHub Actions 통과와 mergeable 상태를 재확인한 뒤
merge 완료했다.
