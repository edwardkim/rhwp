# PR #6613 검토 - reset 없는 다쪽 1x1 중첩 표 canonical 투영

- 원 PR head: `c7226cf02863962c50cbba156a135c0e697e94f6`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

reset이 없고 물리 높이가 두 쪽을 넘는 중첩 표에만 canonical projection을 적용한다. 기존 한 쪽 안팎 및 깊은 wrapper 반례를 전면 포함하지 않는 제한된 게이트다.

## 검증

- `issue_4915_reset_free_multi_page_projection`, `issue_4889_nested_fragment_origin` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.
- `18098267`은 Hancom Office 2020 저장 HWP5, 3쪽으로 확인했다.

## 시각 증적

- 후보 직접 출력: [page 003](../assets/pr_6586_6649_planet6897_integration_20260902/review_6613_issue4915_page_003.png)
- 마지막 쪽 표와 각주가 물리 페이지 안에서 종료하는 것을 확인했다. 파일 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.
- 이 세션에서는 새 Hancom PDF를 만들지 못했으므로 Hancom 좌표 동등성은 주장하지 않는다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 통합 PR에서 이 `-x` 적용분을 수용한다.
