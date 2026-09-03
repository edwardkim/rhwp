# PR #6467 최종 정리·후속 범위 인계

> 이 문서는 2026-08-31의 4단 stack 계획 snapshot이다. #6521 비채택과 #6454 판정 뒤의 현재 3단
> stack·restack 절차는 [Stage 5](../../working/task_m100_6041_stage5.md)가 대체한다.

- 승인일: 2026-08-31
- 승인 범위: #6467 최종화 → 새 이슈 생성 → #6041·#6042 계획 코멘트
- 금지/보류: Ready 전환, merge, 이슈 close, #6521·#6042 구현 및 새 PR 생성
- code candidate: `e37d483fd5f16b2c710a95389f882c9985d50851`
- 시작 head: `5fc2542005ca271c9ac3452ce11416e7a0855ba7`
- 기록 위치: 기존 `codex/issue-6041-budget-first-render-scale`의 trailing mydocs commit

## 실행 순서

1. 원격 head·Draft·본문과 로컬 clean 상태, source/test 불변을 확인했다.
2. focused planner 13건과 TypeScript를 재검증했다. 기존 전체 npm·build·시각 증적은 보존했다.
3. PR을 Stack 2/4로 설명하고 `Closes #6041`을 `Refs #6041`로 바꿨다. Draft는 유지했다.
4. 중복 검색 후 #6521을 생성했다. 기존 #6041과 분리 이유·상호 참조를 명시했다.
5. #6041에 상위 목적·종료 조건, #6042에 tier 소비와 scheduler 책임 경계를 코멘트로 남겼다.
6. 로컬 기록과 원격 본문·링크·Draft 상태를 대조했다. 문서 tail은 같은 source branch에 commit/push한다.

## 이후 승인 게이트

- #6521은 별도 수행·구현 계획 승인 뒤 #6467 head에서 구현하고 Draft PR로 제출한다.
- #6042는 #6521 결과 승인 뒤 해당 PR head에서 시작한다.
- 모든 stack layer가 완료되기 전에는 Ready 전환하지 않는다. 이후 최신 base/diff/CI/시각 검증을
  확인하고 별도 승인으로 bottom-up Ready 전환한다.
- #6041은 두 render-scale 작업이 완료되기 전 종료하지 않는다.

이번 변경은 문서·원격 설명만 다루므로 정책 rollback은 필요 없다. 기록 정정이 필요하면 source/history를
재작성하지 않고 해당 본문·코멘트와 후속 문서 commit으로 바로잡는다.
