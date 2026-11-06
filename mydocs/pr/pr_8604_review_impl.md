# PR #8604 누적 검토 구현 계획

## 대상과 단계

[원 PR #8604](https://github.com/edwardkim/rhwp/pull/8604)의 원 head는 `013459bff4ff5313af53ece5646e79c48647a3aa`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `032dcb80e3de44b3ed4a7ca09549a197b6a18067` | `0430969a1e00388fabb21efdf6ec0c5f120b8260` | fix(typeset): #8596 앞 TAC 표 host 줄 안에 저장된 자리차지 표 프레임은 다음 쪽에서 시작한다 |
| `e74368c345437ca63591ea064c4b972ade999ee7` | `f19c42854af9e6f75e7dd81bf03bebb419369c97` | docs(#8596): 필수 편람 Native 383쪽 TSV — devel 7fdd5441ff 와 바이트 동일 |
| `013459bff4ff5313af53ece5646e79c48647a3aa` | `cad4a9728b8d264f1b155dbce3f9a43ac288aa02` | docs(#8596): 최신 devel rebase head 재측정 — base·head 전쪽 TSV, 대표 PNG, 편람 gate |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
