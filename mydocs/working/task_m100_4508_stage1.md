# task_m100_4508 stage1 — 스킬 표류 가드

- 이슈: #4508 / 브랜치: task_m100_4508 (base devel)
- 산출물: tests/skills_contract.rs — 스킬↔CLI 정합의 하네스 가드 2종

## 무엇을 고정하나

1. skills_reference_only_real_commands — 11개 스킬 본문의 `rhwp <명령>` 참조
   전수를 **테스트 시점의 자기서술**(capabilities 71개 명령 ∪ --help 목록)과
   대조. 골든 박제 없음 — CLI 가 진화하면 기준도 따라 진화한다.
2. skills_have_valid_frontmatter_and_are_executable — frontmatter(name=폴더명,
   description ≥ 20자) + 실행 가능성(실명령 참조 ≥ 1: 스킬은 안내문이 아니라
   실행 규약).

## 실측

- 현재 표류 0건 (2/2 green) — 스킬 11개 전부 실재 명령만 안내.
- 개발 함정 실측 1건: `capabilities --json` 은 단독 사용법 오류(exit 2,
  "--json 은 --search 전용") — 단독 `capabilities` 가 곧 JSON 자기서술.
  가드 주석에 기록.
- v1 범위: 참조 머리 토큰까지(edit/inspect 그룹 하위 2단 검증은 후속).

## stage2 — 그룹 하위명령 2단 검증 (후속 이행)

- 하위명령 실명 출처 = --help (capabilities 는 edit·inspect 우산 이름만, 2단 0개 실측).
- 수확 2경로: 실토큰 줄("edit replace-text …")과 **대안 플레이스홀더**("batch
  <export-text|info|…>" — 안 거두면 batch 가 {fill}만 가진 그룹이 되어 정당한
  `rhwp batch info` 가 오탐됨을 실측으로 확인 후 보강).
- 검출 UX: 실패 메시지가 스킬·참조·실재 하위 목록까지 명세 (`rhwp edit replace`
  같은 오기가 정확히 짚힌다).
- 11개 스킬 2단 참조 전건 실재 확인 (2/2 green).
