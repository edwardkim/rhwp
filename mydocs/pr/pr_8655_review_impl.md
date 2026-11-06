# PR #8655 누적 검토 구현 계획

## 대상과 단계

[원 PR #8655](https://github.com/edwardkim/rhwp/pull/8655)의 원 head는 `cd4cc992f30fdc50478217d3b954c9fd9e1d34e7`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `8406b522616e2d51e4e88124b5721bf25e9239dd` | `7cdfeba76200d1ec1e77a53f8ad16973b8af1d57` (충돌 보정) | fix(layout): TAC 표만 담은 문단의 동반 표를 줄 소유자 기준선에 앉힌다 (#8653) |
| `cd4cc992f30fdc50478217d3b954c9fd9e1d34e7` | `e64f2329cac7256409c6ac032afc60c096cdc235` | test(#8653): 축소본·코퍼스 Native·fresh WASM TSV와 대표 PNG |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
