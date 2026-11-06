# PR #8589 누적 검토 구현 계획

## 대상과 단계

[원 PR #8589](https://github.com/edwardkim/rhwp/pull/8589)의 원 head는 `a68704a019cf56f4dca8db0ee5ff8ae5103febb8`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `0a63dba9bb1d43172add7bb1905cd86ce79a6fd4` | `c2308cd6783507961dc45a6526916ab551a8ea4a` | fix(layout): 어울림 표가 뒤 문단을 띠로 흡수하면 흐름상 다음 문단으로 host 뒤 사다리 일치를 판정 (#8567) |
| `a68704a019cf56f4dca8db0ee5ff8ae5103febb8` | `106ff957b446b110567de7be06dfac5291075c0b` | test: #8567 측정 근거와 검증 증거 등록 |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
