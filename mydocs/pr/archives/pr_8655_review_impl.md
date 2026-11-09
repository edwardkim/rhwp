# PR #8655 누적 검토 구현 계획

## 대상과 단계

[원 PR #8655](https://github.com/edwardkim/rhwp/pull/8655)의 원 head는 `cd4cc992f30fdc50478217d3b954c9fd9e1d34e7`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `8406b522616e2d51e4e88124b5721bf25e9239dd` | `85216fc2651d315f0b2258d2fd21397844f4f329` | fix(layout): TAC 표만 담은 문단의 동반 표를 줄 소유자 기준선에 앉힌다 (#8653) |
| `cd4cc992f30fdc50478217d3b954c9fd9e1d34e7` | `6b0f833f4216f21e608a4b43beae03a3de24ee18` | test(#8653): 축소본·코퍼스 Native·fresh WASM TSV와 대표 PNG |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](../assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](../assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
