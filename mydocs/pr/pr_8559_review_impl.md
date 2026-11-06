# PR #8559 누적 검토 구현 계획

## 대상과 단계

[원 PR #8559](https://github.com/edwardkim/rhwp/pull/8559)의 원 head는 `efa20ac2eafe1137b370d270f459eac54109b5ca`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `2be042208ee670b545a4c55dc9f0b667e9dcf7fb` | `20e4a8937140dd479b627e84a0923fe64bb66ac5` | fix(layout): 쪽·용지 기준 자리차지 표 띠에 밀린 빈 host 줄은 줄높이와 줄간격을 모두 전진 (#8554) |
| `efa20ac2eafe1137b370d270f459eac54109b5ca` | `5e0d83466a2b2f3ef9d246dca9001a8e87c6d213` | docs: #8554 i39 Native 한자 누락 원인 문구를 #8590 확인 결과로 정정 |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
