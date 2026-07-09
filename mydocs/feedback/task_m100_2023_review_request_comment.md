@edwardkim @jangster77 프론트 웹 리팩터링 계획 리뷰를 요청드립니다.

#1883의 Rust 리팩터링 접근과 동일하게, 프론트 웹도 SOLID + 복잡도 2축으로 계획을 수립했습니다.
다만 이번 대상은 `@rhwp/editor` 단독이 아니라 `rhwp-studio`, `/web` legacy, `web/fonts`,
브라우저 확장, VS Code extension, npm 공개 계약을 포함한 프론트 웹 전체입니다.

리뷰 부탁드릴 핵심 안건은 다음입니다.

1. #1883과 동일하게 SOLID + 복잡도 2축을 프론트 리팩터링 거버넌스로 적용해도 되는지.
2. 예비 SOLID 점수 54/100과 S/I 우선순위 판단이 타당한지.
3. `/web` legacy 삭제보다 `web/fonts` canonical 이전을 먼저 하는 순서가 맞는지.
4. `web/fonts` 새 canonical 위치를 `rhwp-studio/public/fonts` 실체화로 둘지, 공통 `assets/fonts`
   신설로 둘지.
5. React/Vue/Svelte 등 runtime UI framework 도입 금지를 이번 리팩터링 금지 목록에 명시해도
   되는지.
6. `@rhwp/editor` iframe/무의존 계약을 public contract로 고정해도 되는지.
7. 확장 CSP/`web_accessible_resources`/sender·URL 검증 guardrail을 실행 PR 체크리스트에
   넣어도 되는지.
8. 실행 하위 이슈는 Phase 0과 Phase A만 먼저 만들고, Phase B 이후는 재평가 후 분리하는
   방식이 적절한지.

계획 초안과 진단 문서는 위 최종 보고서 댓글에 정리해 두었습니다.
리뷰 회신을 받은 뒤 v2로 개정하고, 우선 Phase 0 baseline freeze와 Phase A `web/fonts`
canonical 이슈부터 분리하겠습니다.
