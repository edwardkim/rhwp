# 구현계획서 — task_m100_3695

- **이슈**: #3695
- **상위 이슈**: #1528
- **선행 작업**: #3693, PR #3715, merge commit `fe9749d542f46643e408c23878229c326e341363`
- **브랜치**: `codex/issue-3695-export-structure-auto`
- **수행계획서**: `mydocs/plans/task_m100_3695.md`
- **절차 상태**: Stage 7 PR 리뷰 confidence 보정·최신 `devel` 전체 검증 완료
- **Draft PR**: [#3749](https://github.com/edwardkim/rhwp/pull/3749)
- **다음 승인 경계**: PR 최신 CI·리뷰와 ready/merge 판정
- **WIP 증적**: `8343c98c6`

## 복구 적용 원칙

1. `8343c98c6`의 코드·테스트·CLI 문서를 아래 Stage별 계획과 독립 대조한다.
2. 계획 미충족이나 추가 수정 필요 사항은 소스를 고치지 않고 새 단계 보고서에 기록한다.
3. focused 검증 결과와 채택·수정·폐기 권고를 작성하고 작업지시자 판정 전에는 다음 단계로 넘어가지 않는다.
4. 기존 커밋과 보고서는 절차 누락의 감사 증적으로 보존하며 사후 승인으로 표기하지 않는다.

2026-08-01 독립 대조와 focused 검증 뒤 작업지시자가 WIP 채택을 승인했다. 위 원칙에 따라 기존
커밋은 재작성하지 않고 승인된 #3695 구현으로 사용한다.

## Stage 1 — auto 증거 수집

1. 참조된 paragraph의 para shape를 순회한다.
2. `HeadType::Outline`은 즉시 authoritative outline으로 판정한다.
3. `HeadType::Number`는 약한 outline 증거로 기록한다.
4. 렌더링 질의와 같은 수식 포함 텍스트 조립기로 편·장·절·관·조 marker를 찾는다.

## Stage 2 — effective mode 우선순위 (초기 구현)

1. 명시적 Outline을 가장 먼저 선택한다.
2. Outline이 없으면 primary clause marker를 Number보다 우선한다.
3. primary clause marker가 없을 때만 Number로 outline을 선택한다.
4. 증거가 없으면 clause로 폴백한다.
5. explicit `StructureMode::Outline|Clause`는 selector를 우회한다.

## Stage 3 — synthetic·실문서 회귀 테스트

1. synthetic:
   - 조문 + 일반 Number 혼합 → auto clause
   - pure clause → auto clause
   - explicit Outline + 조문 모양 텍스트 → auto outline
   - single Number-only → auto outline
   - 같은 혼합 문서의 explicit outline/clause 선택 유지
2. 실제 sample:
   - `hwpctl_API_v2.4.hwp` 명시적 Outline 유지
   - `biz_plan.hwp` 복수 Number outline 유지
   - `추진일정.hwp` 단일 Number outline 유지
3. #3693 실문서 clause 회귀와 CLI JSON 계약을 다시 실행한다.

## Stage 4 — 문서·검증·커밋

1. CLI 매뉴얼의 auto 설명을 selector의 실제 우선순위와 맞춘다.
2. 수행계획서의 focused 명령을 순차 실행한다.
3. `mydocs/working/task_m100_3695_stage1.md`와
   `mydocs/report/task_m100_3695_report.md`에 red·green·실측 결과를 기록한다.
4. 오늘할일 #3695 상태를 단계 승인 결과에 맞춰 갱신하고 단계 변경 전 커밋한다.

## Stage 5 — 최신 devel 통합과 PR 준비

1. WIP 감사 커밋을 재작성하지 않고 `upstream/devel` `fe9749d54`를 merge한다.
2. #3715가 확장한 marker·clause gate와 #3695 `select_auto_mode()`를 모두 보존한다.
3. focused 검증 뒤 `cargo test --profile release-test --tests`, fmt, diff check, clippy를 순차 실행한다.
4. 새 Stage 3 보고서와 최종 보고서를 갱신해 로컬 merge commit에 포함한다.
5. 원격 push와 PR 생성은 작업지시자 승인 뒤 수행한다.

## Stage 6 — PR head base drift 동기화

1. draft PR #3749 생성 뒤 최신 `upstream/devel`을 다시 fetch한다.
2. PR #3742 merge로 전진한 `cc3829116`을 merge하고 충돌을 양쪽 기록 보존 방식으로 해결한다.
3. 결합 경계인 structure, #3695, #3693, CLI JSON focused 테스트와 fmt·diff·clippy를 재실행한다.
4. Stage 4 보고와 PR 본문을 실제 검증 기준에 맞춰 갱신한 뒤 head를 push한다.

## Stage 7 — PR 리뷰 confidence 보정

1. Number와 충돌하는 auto selector의 clause 증거를 제목형 `조`로 좁힌다.
2. 탭+끝자리 숫자 목차와 marker 뒤 조사형 상호참조를 `auto_clause_heading_allowed()`에서 제외한다.
3. 실제 시장구조조사 negative, 조사형 상호참조·쪽번호 synthetic negative, 실제 협정서 기반
   Number+조 positive를 추가한다.
4. 조 증거를 이미 찾은 뒤에는 나머지 문단 텍스트 조립을 생략하되, 뒤쪽 explicit Outline 탐색은 유지한다.
5. 기존 devel auto와 보정 auto를 같은 parse 결과에서 비교해 top-level·recursive corpus 영향 0건을 확인한다.
6. `mydocs/pr/archives/pr_3749_review.md`, Stage 5 보고, 최종 보고, CLI manual, 오늘할일을 갱신한다.
7. 최신 devel 결합 트리에서 focused·전체 release-test·fmt·diff·clippy를 순차 실행한다.

## 비적용

- 확률값·튜닝 가능한 threshold 도입 없음
- 새 외부 의존성·fixture·baseline 추가 없음
- CLI parser, WASM binding, MCP schema 변경 없음
