# PR #2514 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2514](https://github.com/edwardkim/rhwp/pull/2514) |
| 작성자 / base | dependabot[bot] / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +16/-16, 2 files, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | `rhwp-studio` Vite 8.1.4 → 8.1.5 및 lockfile 갱신 |
| 판단 | Dependabot 누적 통합 PR에 수용 |

## 변경 범위와 통합

- PR 본문은 Vite 8.1.5 patch release의 bundled-dev, client, optimizer, SSR 보완을 안내한다. 검토 시점 PR 코멘트는 없었다.
- 원 커밋 `7b5f58f13`을 최신 `upstream/devel` 위에 충돌 없이 적용했고, 통합 브랜치 적용 커밋은 `73bc884de`다.
- manifest와 lockfile이 같은 Vite 8.1.5를 가리키며, Studio 외 경로의 의존성 선언은 이 PR에서 바꾸지 않는다.

## 렌더 영향 판정

- 개발 서버·번들 도구 patch update이며 renderer·layout 산출 계약을 직접 변경하지 않는다. visual sweep 대상이 아니다.

## 검증

- `rhwp-studio`: `npm ci`, `npm test`, `npm run build` 통과.
- production 의존성 기준 `npm audit --omit=dev`는 취약점 0건이다.
- 전체 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`,
  `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `git diff --check upstream/devel...HEAD`를 통과했다.

## 리스크와 권고

- production build의 CanvasKit `fs`/`path` externalization 및 bundle-size 경고는 변경 전부터 있던 build 경고로, 빌드를 실패시키지 않는다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤 merge한다.
