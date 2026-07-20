# PR #2516 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2516](https://github.com/edwardkim/rhwp/pull/2516) |
| 작성자 / base | dependabot[bot] / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +14/-14, 2 files, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | `rhwp-chrome` Vite 8.1.4 → 8.1.5 및 lockfile 갱신 |
| 판단 | Dependabot 누적 통합 PR에 수용 |

## 변경 범위와 통합

- PR 본문은 Vite 8.1.5 patch release를 안내한다. 검토 시점 PR 코멘트는 없었다.
- 원 커밋 `2f6717579`을 최신 `upstream/devel` 위에 충돌 없이 적용했고, 통합 브랜치 적용 커밋은 `15dfc276a`다.
- manifest와 lockfile이 같은 Vite 8.1.5 및 Rolldown 1.1.5 해소를 가리킨다.

## 렌더 영향 판정

- Chrome 확장 production bundle의 build tool patch update이며 viewer renderer·layout 계약을 직접 변경하지 않는다. visual sweep 대상이 아니다.

## 검증

- `rhwp-chrome`: `npm ci`, `npm run build` 통과. manifest, content script, WASM, 폰트 복사까지 build script가 완료됐다.
- 전체 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`,
  `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `git diff --check upstream/devel...HEAD`를 통과했다.

## 리스크와 권고

- production build의 기존 HTML script·asset resolution·bundle-size 경고는 새 실패가 아니며 build가 성공했다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤 merge한다.
