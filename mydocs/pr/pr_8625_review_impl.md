# PR #8625 누적 검토 구현 계획

## 대상과 단계

[원 PR #8625](https://github.com/edwardkim/rhwp/pull/8625)의 원 head는 `39ceb92ace1a6ad2ca9578c5b72b6940bb096281`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 접수 시 적용 계획 | 제목 |
| --- | --- | --- |
| `693de034cb8edef9bd57c5689a5090a4b161a500` | pending | fix(typeset): #8620 글자처럼 취급 표 host 의 컨트롤 전용 꼬리 줄은 쪽을 넘으면 다음 쪽에 놓는다 |
| `39ceb92ace1a6ad2ca9578c5b72b6940bb096281` | pending | docs(#8620): 필수 편람 Native 383쪽 TSV — devel 과 바이트 동일 |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
