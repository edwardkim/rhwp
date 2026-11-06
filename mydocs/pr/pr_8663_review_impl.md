# PR #8663 누적 검토 구현 계획

## 대상과 단계

[원 PR #8663](https://github.com/edwardkim/rhwp/pull/8663)의 원 head는 `a0f7203d788415f378f3ea61d3679a9107983666`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 접수 시 적용 계획 | 제목 |
| --- | --- | --- |
| `e3db55a6f16b57c2ef73a3aaaf22f5298ccc8bfc` | pending | fix(typeset): 저장 vpos 가 되감긴 보이는 호스트 글줄을 표와 함께 다음 쪽에서 시작한다 (#8652) |
| `fbcdbf58e0d1b55bdfec9bb6623b3f0a773b9c20` | pending | fix(footnote): 각주 번호와 같은 문단의 글자처럼 취급 표를 번호 뒤에 그린다 (#8657) |
| `a0f7203d788415f378f3ea61d3679a9107983666` | pending | docs: #8652·#8657 Visual Sweep TSV 와 대표 review·overlay PNG |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
