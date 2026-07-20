# PR #2526 검토 - Dependabot 의존성 patch update 통합

| 항목 | 내용 |
|---|---|
| PR | [#2526](https://github.com/edwardkim/rhwp/pull/2526) |
| 작성자 / base | jangster77 / `devel` |
| 대상 | Dependabot 원 PR [#2514](https://github.com/edwardkim/rhwp/pull/2514), [#2515](https://github.com/edwardkim/rhwp/pull/2515), [#2516](https://github.com/edwardkim/rhwp/pull/2516), [#2517](https://github.com/edwardkim/rhwp/pull/2517) |
| 검토자 | @jangster77 |
| 판단 | 최신 통합 PR head CI 성공을 조건으로 수용 |

## 통합 범위

- Studio, Chrome, Firefox의 Vite를 8.1.4에서 8.1.5로 통일하고 각 package lockfile을 함께 갱신한다.
- `serde`, `serde_core`, `serde_derive`를 1.0.229로 갱신한다. derive macro 경로에는 `syn 3.0.1`이 추가되며 기존 소비자는 `syn 2.0.114`를 유지한다.
- 원 PR별 검토 기록은 [PR #2514 검토 기록](pr_2514_review.md), [PR #2515 검토 기록](pr_2515_review.md), [PR #2516 검토 기록](pr_2516_review.md), [PR #2517 검토 기록](pr_2517_review.md)에 보관한다.
- Dependabot 원 커밋은 rewrite하지 않고 저자 보존 체리픽했다. 새 maintainer 소스 보정은 없다.

## 검증

- `rhwp-studio`, `rhwp-chrome`, `rhwp-firefox`에서 각각 `npm ci`, `npm run build`를 통과했다.
- Studio `npm test`를 통과했고, `npm audit --omit=dev`는 production 취약점 0건이다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과.
- `git diff --check upstream/devel...HEAD` 통과.

## 렌더 영향 판정

- package manifest·lockfile의 patch update만 포함한다. renderer, layout, PDF/SVG 출력 계약은 바뀌지 않아 visual sweep 대상이 아니다.
- Chrome과 Firefox extension build는 manifest, content script, WASM, 폰트 복사까지 완료되는 것을 확인했다.

## Merge 전 조건과 후속

- [#2526](https://github.com/edwardkim/rhwp/pull/2526) 최신 head의 필수 GitHub Actions가 성공해야 한다.
- 작업지시자의 merge 승인 뒤에만 merge한다.
- merge 뒤 `upstream/devel`을 동기화하고, 각 원 Dependabot PR에 통합 PR 링크와 개별 검토 기록을 남긴 뒤 close한다. merge SHA와 close 결과는 GitHub 원천 기록으로 확인한다.
