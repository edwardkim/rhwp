# PR #8662 누적 검토 구현 계획

## 대상과 단계

[원 PR #8662](https://github.com/edwardkim/rhwp/pull/8662)의 원 head는 `f84e0f3ffe3b154d286bc878fc90facdb7ced153`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 접수 시 적용 계획 | 제목 |
| --- | --- | --- |
| `faaf09008aa5e835676275eb6badc0593552ccde` | duplicate-patch | fix(renderer): Dotum U+2024 은 Haansoft Dotum 이 공급 (#8619) |
| `3d870c1410aa8c4b1387385413a0e846c6112298` | duplicate-patch | docs: #8619 rebase 뒤 Native/fresh WASM sweep 증거 갱신 |
| `cb1ab1d5af6ce0a4614365c6ca7d6a8b5fc0ba77` | pending | fix(renderer): 저장 줄 없는 HWP5 TAC 표의 행 높이와 넘친 공백의 줄 끝 처리 (#8639) |
| `c1db11718bbc5319ac78507e7cabb8cc0fe3f8b3` | pending | fix(renderer): 변경 추적 0x08 문서는 원본 프로그램으로 저장 줄을 다시 짜지 않는다 (#8639) |
| `f84e0f3ffe3b154d286bc878fc90facdb7ced153` | pending | docs: #8639 Native/fresh WASM sweep 증거 (행 높이·끝 공백·변경 추적 0x08) |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
