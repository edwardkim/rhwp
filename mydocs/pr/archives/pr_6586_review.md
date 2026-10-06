# PR #6586 검토 - 페이지 고정 블록의 `vpos=0` 재설정

- 원 PR head: `7068b527eb99f1333f17cd6f248b404d68d84337`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 기능 통합 후보: `review/planet6897-open-batch-20260902`의 `208a18b8d7cd86568a3b1c15e026f202454631a9` 이전 18개 `-x` 체리픽 집합
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

`#6535`의 비-TAC 페이지 고정 블록에서만 `vpos=0`을 저장 사다리 재설정으로 해석한다. TAC와 일반 흐름의 기존 갈래를 넓히지 않는 제한된 조건이며, 회귀 시험이 그 조건을 잠근다.

## 검증

- `issue_6535_page_anchored_block_vpos_zero` 통과.
- 통합 후보에서 `cargo fmt --check`, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.
- 원본 `36399617`은 Hancom Office 2020 저장 HWPX, 1쪽으로 확인했다.

## 시각 증적

- 후보 브랜치 직접 출력: [page 001](../assets/pr_6586_6649_planet6897_integration_20260902/review_6586_issue6535_page_001.png)
- 파일 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 고정했다.
- 한컴 PDF를 이 세션에서 새로 생성하지 못했으므로, 이 이미지는 rhwp 후보 직접 출력이며 Hancom PDF 동등성 주장에는 사용하지 않는다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 통합 PR에서 이 `-x` 적용분을 수용한다.
