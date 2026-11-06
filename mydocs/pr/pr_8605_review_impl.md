# PR #8605 누적 검토 구현 계획

## 대상과 단계

[원 PR #8605](https://github.com/edwardkim/rhwp/pull/8605)의 원 head는 `5d37286267136c9978d7ac2f804913730ab6becf`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `4f8570309e925468552036f4c20d099011d55a35` | `a1a789015312ee650c31ca57f0ce767ac6b4ae86` | fix(render): WASM Canvas 합성 굵기를 실제 face 윤곽의 fill+stroke로 그린다 (#8430) |
| `3d0b4420dbee220253189fc149c48125cc0e2dc4` | `3d1d24861e2b555f128d2eb91d5263fa0a86b962` | docs(#8430): 30396 권고문 Native/fresh WASM 전쪽 TSV·확대 판독·글꼴 예외 증적 |
| `5d37286267136c9978d7ac2f804913730ab6becf` | `5328c886467c4e799043c237f9f3b976c17c6829` | docs(#8430): 최신 devel rebase head 재측정과 글꼴 예외 source SHA 갱신 |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
