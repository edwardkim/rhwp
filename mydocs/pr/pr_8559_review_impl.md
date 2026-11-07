# PR #8559 누적 검토 구현 계획

## 대상과 단계

[원 PR #8559](https://github.com/edwardkim/rhwp/pull/8559)의 원 head는 `efa20ac2eafe1137b370d270f459eac54109b5ca`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `2be042208ee670b545a4c55dc9f0b667e9dcf7fb` | `b2cc752de12a473d0ff6c7f94fc36af506226f24` | fix(layout): 쪽·용지 기준 자리차지 표 띠에 밀린 빈 host 줄은 줄높이와 줄간격을 모두 전진 (#8554) |
| `efa20ac2eafe1137b370d270f459eac54109b5ca` | `3874b65165813118bc26078bad7ecda97860a2ce` | docs: #8554 i39 Native 한자 누락 원인 문구를 #8590 확인 결과로 정정 |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
