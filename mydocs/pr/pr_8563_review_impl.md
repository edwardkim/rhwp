# PR #8563 누적 검토 구현 계획

## 대상과 단계

[원 PR #8563](https://github.com/edwardkim/rhwp/pull/8563)의 원 head는 `bc8a04e056ccfbbe371b21239ecfd13696cea0b2`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `f3148d52472745de33fff5ae8feb33a1e9805943` | `fb8628cc689bae4972942282c3623cc1bb7babf5` | fix(typeset): 빈 host 자리차지 표 뒤 다음 문단이 빈 자리차지 표 host 여도 저장 host 줄 프레임을 받는다 (#8560) |
| `bc8a04e056ccfbbe371b21239ecfd13696cea0b2` | `9633d736764381b428644a0f0e5ae18d8d4366ff` | test: #8560 측정 근거·검증 증거와 새 원본 IR 왕복 baseline 등록 |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
