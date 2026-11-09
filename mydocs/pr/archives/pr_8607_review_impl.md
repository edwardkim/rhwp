# PR #8607 누적 검토 구현 계획

## 대상과 단계

[원 PR #8607](https://github.com/edwardkim/rhwp/pull/8607)의 원 head는 `b7b93df883ef0eed535d1e80b5c45ea6d60ce3a9`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `d8ded0cc7171d54e3b76be63ed7125b97aae9d67` | `77987a41b56d58825e39749a5dbb2bd9f14c017f` | fix(typeset): #8606 빈 host 의 자기 오프셋 자리차지 본표는 머리 표에 밀린 표가 아니다 |
| `8ee296b76e33e79a8b225385e252b12f35c81a65` | `8a40e56e6a6504e0cbce5471b2099db97a74eb3a` | docs(#8606): 필수 편람 Native 383쪽 TSV — devel 7fdd5441ff 와 바이트 동일 |
| `b7b93df883ef0eed535d1e80b5c45ea6d60ce3a9` | `61dc7c2e0d7bafc1383904d872c2725a5071a353` | ci: Canvas visual diff 의 Chrome 시작 시간 초과 재실행 (#8607) |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](../assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](../assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
