# PR #8662 누적 검토 구현 계획

## 대상과 단계

[원 PR #8662](https://github.com/edwardkim/rhwp/pull/8662)의 원 head는 `f84e0f3ffe3b154d286bc878fc90facdb7ced153`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `faaf09008aa5e835676275eb6badc0593552ccde` | duplicate-patch → `5554a192b7a62836fc7f9a460623b5da205cf151`와 동일 patch | fix(renderer): Dotum U+2024 은 Haansoft Dotum 이 공급 (#8619) |
| `3d870c1410aa8c4b1387385413a0e846c6112298` | duplicate-patch → `71bd8abf48dbad4afa09d66803f6011546add734`와 동일 patch | docs: #8619 rebase 뒤 Native/fresh WASM sweep 증거 갱신 |
| `cb1ab1d5af6ce0a4614365c6ca7d6a8b5fc0ba77` | `3ffeb3d9441ba2428de7cb6c40ccdbf513f56240` | fix(renderer): 저장 줄 없는 HWP5 TAC 표의 행 높이와 넘친 공백의 줄 끝 처리 (#8639) |
| `c1db11718bbc5319ac78507e7cabb8cc0fe3f8b3` | `c73fcea4254fe3dc15ba2940d07597c1a050c6ca` | fix(renderer): 변경 추적 0x08 문서는 원본 프로그램으로 저장 줄을 다시 짜지 않는다 (#8639) |
| `f84e0f3ffe3b154d286bc878fc90facdb7ced153` | `faefc4a35d72cea17ac7830f4f0eb3d49424c631` | docs: #8639 Native/fresh WASM sweep 증거 (행 높이·끝 공백·변경 추적 0x08) |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
