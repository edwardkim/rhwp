# PR #8654 누적 검토 구현 계획

## 대상과 단계

[원 PR #8654](https://github.com/edwardkim/rhwp/pull/8654)의 원 head는 `d971025c2bb85ebcd2453578800e6130445b81b8`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `3e813cb33670dffbf0a495c87b31e9b016581576` | `d45d16004a753c2618de8b979a046e9926423b04` | fix(layout): TAC 표 앞 저장 줄을 흐름 표가 내보내고 그린 줄 전진으로 원점을 맞춘다 (#8641) |
| `2aedb2a8580da9bb1b2d97643d3a97defa4f1660` | `2b8605df5400676afc39806019af6cdfbf2f0315` | test(#8641): 새 공개 샘플 겹침 원장과 Native·fresh WASM TSV·대표 PNG |
| `5fdbd2b17faef122ada2f7aecedda5e3894678f3` | `04416e216df13f5152b93620a4de215bb48d8e96` | fix(layout): 칸 안의 쪽 기준 글 앞으로 표를 칸 흐름에 쌓지 않는다 (#8648) |
| `c546460f55cd9cf94d8106cde271750e0a131c0c` | `c981fbaadbe97148631cf4dbf18c142ea4d4cc4d` | test(#8648): 축소본·코퍼스 Native·fresh WASM TSV와 대표 PNG |
| `cc4003658bc80d463bad6b9784494178fa3fd791` | pure-stack-bridge-no-new-change | merge: 선행 PR #8645 head 를 쌓는다 (#8641) |
| `4db4c75ae4eff93b08e37a5e4ee02d90a3e2d435` | pure-stack-bridge-no-new-change | merge: 선행 PR #8649 head 를 쌓는다 (#8648) |
| `7f85c9f4fa5ee95a9d58eeb569794ad89c9e1dc5` | `8d1dd6ddb44e9d532cf33518be73b86b18cfe5d1` (충돌 보정) | fix(layout): 같은 줄 TAC 표 공유 기준선과 rowspan 초과분이 받는 행 (#8641) |
| `d971025c2bb85ebcd2453578800e6130445b81b8` | `445adbaa70126fb39773e2e10e0d0879390f6ff2` | docs(evidence): #8641 공유 기준선·rowspan Native/fresh WASM TSV와 대표 PNG |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
