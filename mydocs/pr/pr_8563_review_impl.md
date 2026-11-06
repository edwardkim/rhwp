# PR #8563 누적 검토 구현 계획

## 대상과 단계

[원 PR #8563](https://github.com/edwardkim/rhwp/pull/8563)의 원 head는 `bc8a04e056ccfbbe371b21239ecfd13696cea0b2`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `f3148d52472745de33fff5ae8feb33a1e9805943` | `43985fa6c808dc3614048a74aa924670b6d9f982` | fix(typeset): 빈 host 자리차지 표 뒤 다음 문단이 빈 자리차지 표 host 여도 저장 host 줄 프레임을 받는다 (#8560) |
| `bc8a04e056ccfbbe371b21239ecfd13696cea0b2` | `b17d708d9e38c3dd8eaa941b5eb780ed53b0f4ba` | test: #8560 측정 근거·검증 증거와 새 원본 IR 왕복 baseline 등록 |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
