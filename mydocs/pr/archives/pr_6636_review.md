# PR #6636 검토 - U+2007 고정폭 빈칸 전진폭

- 원 PR head: `557fc2a5c8aba224b80e5bc3e9715feec4052ec0`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

U+2007 fallback advance를 `0.5em`에서 `0.25em`으로 바로잡고, 절대 픽셀이 아닌 일반 공백 대비 비율로 회귀를 잠근다. 변경한 golden/trace는 옛 `0.5em` 상수에 의존한 기대값이라 같은 계약 변경에 포함된다.

## 검증

- `issue_6597_figure_space_advance`, `issue_6036_yoongothic_metrics` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.
- 원문 `30307`은 13쪽 HWP5(저장 version `7.5.12.614`)로 확인했다.

## 시각 증적

- 후보 직접 출력: [page 003](../assets/pr_6586_6649_planet6897_integration_20260902/review_6636_issue6597_page_003.png), [page 005](../assets/pr_6586_6649_planet6897_integration_20260902/review_6636_issue6597_page_005.png)
- U+2007의 정확한 advance 변화는 focused test가 비율로 검증한다. 이 세션에서는 새 Hancom PDF를 생성하지 못했으므로 한컴 좌표 동등성을 새로 주장하지 않는다.
- 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 통합 PR에서 이 `-x` 적용분을 수용한다.
