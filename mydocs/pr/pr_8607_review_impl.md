# PR #8607 누적 검토 구현 계획

## 대상과 단계

[원 PR #8607](https://github.com/edwardkim/rhwp/pull/8607)의 원 head는 `b7b93df883ef0eed535d1e80b5c45ea6d60ce3a9`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 접수 시 적용 계획 | 제목 |
| --- | --- | --- |
| `d8ded0cc7171d54e3b76be63ed7125b97aae9d67` | pending | fix(typeset): #8606 빈 host 의 자기 오프셋 자리차지 본표는 머리 표에 밀린 표가 아니다 |
| `8ee296b76e33e79a8b225385e252b12f35c81a65` | pending | docs(#8606): 필수 편람 Native 383쪽 TSV — devel 7fdd5441ff 와 바이트 동일 |
| `b7b93df883ef0eed535d1e80b5c45ea6d60ce3a9` | pending | ci: Canvas visual diff 의 Chrome 시작 시간 초과 재실행 (#8607) |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
