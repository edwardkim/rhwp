# PR #6191 검토 기록

- 통합 PR: [#6191](https://github.com/edwardkim/rhwp/pull/6191)
- 기준: `upstream/devel` `1b91c2025`
- 통합 브랜치: `review/planet6897-ci-green-20260827`
- 제외: #6178은 현재 head의 Build & Test 및 nextest archive B/C 실패 때문에 반입하지 않았다.

## 반입 범위와 출처

현재 head CI가 통과한 planet6897 원본 PR #6158, #6160, #6161, #6162, #6163, #6165, #6166, #6168, #6169, #6170, #6177, #6183을 원래 commit 순서와 `cherry-pick -x` 출처로 통합했다. #6161과 #6170의 다중 commit도 원래 순서를 유지했다.

## 메인터너 통합 보정

- #6158/#6161 fixture 충돌은 두 회귀 기준선을 모두 보존했다.
- #6168/#6169 `layout.rs` 충돌은 paragraph-relative overlay band와 offset-float host-line 보호를 함께 유지했다.

## 검증

| 항목 | 결과 |
| --- | --- |
| source PR current-head CI | 반입 12건 모두 통과/의도적 skip만 존재 |
| manifest, tier policy, format | 통과 |
| integration 전체 회귀 | `8,417 passed`, `43 skipped` |
| Native Skia lib | 통과 |
| WASM build | `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 통과 |
| Studio production build | `npm --prefix rhwp-studio run build` 통과 |
| Studio #6117 E2E | local Vite `127.0.0.1:7701` + headless Chrome 통과 |
| 기준 PDF와 visual sweep | [증적 인덱스](../assets/pr_6191/README.md) 참조 |

## 검토 결론

차단할 통합 결함은 찾지 못했다. #6168/#6169의 SVG PDF export `InvalidImage`은 pristine `upstream/devel`에서도 재현되는 기존 exporter 제한이며, Native Skia PNG fallback과 각 회귀 검증으로 확인했다. 현재 head CI가 완료되면 merge 대상으로 처리한다.
