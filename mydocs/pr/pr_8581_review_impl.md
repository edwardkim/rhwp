# PR #8581 누적 검토 구현 계획

## 대상과 단계

[원 PR #8581](https://github.com/edwardkim/rhwp/pull/8581)의 원 head는 `f25efcf2c9dba5793f78a5e5d9c845944c562dfb`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `f25efcf2c9dba5793f78a5e5d9c845944c562dfb` | `3d3ea07fda4af40155bf3a7af08b186860b1bb20` | fix(renderer): 줄 머리 TAC 표 뒤 문단 끝 공백에 줄을 주지 않는다 (#8574) |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
