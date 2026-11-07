# PR #8654 누적 검토 구현 계획

## 대상과 단계

[원 PR #8654](https://github.com/edwardkim/rhwp/pull/8654)의 원 head는 `d971025c2bb85ebcd2453578800e6130445b81b8`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `3e813cb33670dffbf0a495c87b31e9b016581576` | `90b770cfb33ebfdd219699d6a6e9e6c4150a074e` | fix(layout): TAC 표 앞 저장 줄을 흐름 표가 내보내고 그린 줄 전진으로 원점을 맞춘다 (#8641) |
| `2aedb2a8580da9bb1b2d97643d3a97defa4f1660` | `d6678be30053d3030f8c5c93eb8fd2e61fbd87f9` | test(#8641): 새 공개 샘플 겹침 원장과 Native·fresh WASM TSV·대표 PNG |
| `5fdbd2b17faef122ada2f7aecedda5e3894678f3` | `026f8b0831413a379b4c53690788d03cf2ab8445` | fix(layout): 칸 안의 쪽 기준 글 앞으로 표를 칸 흐름에 쌓지 않는다 (#8648) |
| `c546460f55cd9cf94d8106cde271750e0a131c0c` | `e3e6da5d98be96d12ea8c8e84019946af4c5ef69` | test(#8648): 축소본·코퍼스 Native·fresh WASM TSV와 대표 PNG |
| `cc4003658bc80d463bad6b9784494178fa3fd791` | pure-stack-bridge-no-new-change | merge: 선행 PR #8645 head 를 쌓는다 (#8641) |
| `4db4c75ae4eff93b08e37a5e4ee02d90a3e2d435` | pure-stack-bridge-no-new-change | merge: 선행 PR #8649 head 를 쌓는다 (#8648) |
| `7f85c9f4fa5ee95a9d58eeb569794ad89c9e1dc5` | `75d7f457f37fde4a266e78989ba6eeea6ccd176d` | fix(layout): 같은 줄 TAC 표 공유 기준선과 rowspan 초과분이 받는 행 (#8641) |
| `d971025c2bb85ebcd2453578800e6130445b81b8` | `5b9ec17d6f4ca90d28413af386d0894047c06251` | docs(evidence): #8641 공유 기준선·rowspan Native/fresh WASM TSV와 대표 PNG |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
