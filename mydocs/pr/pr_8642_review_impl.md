# PR #8642 누적 검토 구현 계획

## 대상과 단계

[원 PR #8642](https://github.com/edwardkim/rhwp/pull/8642)의 원 head는 `7d40345bc014f3cb4eca4235783afcfb670899f9`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 접수 시 적용 계획 | 제목 |
| --- | --- | --- |
| `aceb89c6d3f47c8790c22929e2615b75c3ef3465` | pending | fix(typeset): 다 들어가지 않는 각주는 들어가는 앞 조각만 비용으로 잡고 넘어온 각주를 이어받는 쪽 예산에서 뺀다 (#8468) |
| `8ece4b20852a56cf2523faebb87ea24d7339a5ac` | pending | test: #8468 앞부분만 들어간 각주의 표 줄 — 뒤 본문 줄이 각주 글 줄 위를 채운다 |
| `595099ca975b231117f86bd2e9794ba05c1ce0e8` | pending | test: #8468 T20 측정 주석을 실제 값으로 채우고 fmt 적용 |
| `d9b846cafab496a3d72f9953a743be17e23a7071` | pending | fix(typeset): 블록 표가 든 각주의 조각 높이를 paint 조각 측정으로 잰다 (#8468) |
| `2211d9e95698007c53788aadd15e9cd3d17df6ab` | pending | test: #8468 후속 측정 근거와 Native·fresh WASM TSV·대표 PNG 증적 |
| `7d40345bc014f3cb4eca4235783afcfb670899f9` | pending | ci: #8642 CI 재실행 (runner 디스크 부족으로 실패한 adapter inter-diff·prop roundtrip) |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
