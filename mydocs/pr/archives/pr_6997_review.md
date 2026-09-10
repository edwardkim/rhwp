# PR #6997 self-review

## 대상과 경로

- PR: https://github.com/edwardkim/rhwp/pull/6997
- 작성자: jangster77; collaborator self-review. reviewer assign은 하지 않았다.
- base: devel; 준비 기준 SHA: 8644cf0a4a1431de74df7ea5f3acbde9bac05f10.
- branch: docs/closed-pr-100-claude-audit-20260910.
- 최초 감사 문서 commit: c07fc14384665d721f88c1c505dc3171cdfeee43.
- 기본 경로: collaborator_self_merge; 보조 경로: intake_and_review.
- 실제 CLAUDE.md/스킬 변경을 포함하므로 review-only fast-pass B로 취급하지 않는다.

## 실제 반영 내용

[최근 종료 PR 100건 감사](../../working/closed_pr_100_claude_audit_20260910.md)의 제안을
사용자 지시에 따라 같은 PR에서 실제 지침에 반영했다.

- 루트 CLAUDE.md: AGENTS 자동 import, 역할별 정본, 독립적인 기대값과 원인/반례/출력 비교 완료 기준.
- contributor SKILL.md: 현재 CONTRIBUTING과 역할별 검토 절차를 우선하는 활성 라우터.
- 활성 자식 8개: 분석, 구현 범위, 검증, 시각 증거, 예외, 템플릿, 순서와 반복 오류 방지.
- native Clippy/related test만으로 완료하는 규칙, --generate 제출 절차, 첫 체크박스 fmt 강제,
  과거 작업의 전역 수정 금지와 고정 suite/3회 반복 규칙을 활성 경로에서 제거했다.
- 과거 예제/fixture 전체는 이번에 수정하지 않았다. 실행 정본이 아님을 명시하고 활성 라우터에서 분리했다.
- 제품 source/test/workflow/sample/PDF/PNG, 생성 suite/manifest와 실행 로그는 변경 범위에 포함하지 않는다.
- 기존 CAP-4561의 역할과 진입점은 유지한다. 새 capability나 실행 hook을 추가하지 않는다.

## 조사와 검증 상태

감사 시 GitHub 본문 100개, 일반 댓글 140개, review 이벤트 7개, inline thread 4개/댓글 7개,
commit 항목 480개와 변경 파일 목록을 수집했다.
25건은 제품 오류 건수가 아니라 동작/안전성/시험/증적 보정 확인 하한이다.
Claude 표기만으로 실제 모델 실행이나 결함 책임을 단정하지 않았다.

이번 지침 변경에 대해 빌드/회귀/Clippy/시각 산출, Markdown 링크/공백 검사,
스킬 router/계약 검사와 실제 Claude 세션의 지침 준수 검증은 실행하지 않았다.
기존 문구에 의존하는 계약 테스트와의 호환성도 미확인이다.
자동 import와 지침 보강은 동작 개선을 위한 조치이며 오류 방지의 강제 실행 장치는 아니다.

## 최종 판정과 후속 조건

- 판정: 머지 보류. 실제 지침 보정은 반영했으나 관련 문서/스킬 검증 결과는 미확인이다.
- 해제 조건: 변경 범위에 맞는 문서/스킬 검증과 최신 head required CI, mergeability 확인.
- 사용자 요청 범위는 실제 지침 수정과 기존 PR 반영이며 이번 단계에서 merge하지 않는다.
- 이번 보정에 대한 검증 완료 SHA는 없다. 최초 감사 SHA를 보정본의 검증 SHA로 사용하지 않는다.

## Merge 후 comment 및 후속 처리 계획

새 PNG/PDF나 제품 시각 검증 수치를 만들지 않는다.
과거 원 PR에 추가 comment를 게시하거나 이미 끝난 issue를 다시 종료하지 않는다.
실제 merge 승인 후 merge SHA와 최신 head CI를 기록하고 devel 동기화 및 승인된 branch 정리를 수행한다.
이번 PR의 감사/self-review/오늘할일을 재사용하며 같은 기록만을 위한 추가 문서 PR은 만들지 않는다.
공유 target과 기존 증적은 보존한다.
