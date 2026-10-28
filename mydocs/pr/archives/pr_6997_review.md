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
- 일반 기여자의 MCP 접근을 전제하지 않고 필요한 한컴 버전별 PDF 직접 출력/첨부와 생성 환경 기록을 명시했다.
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

로컬 빌드/회귀/Clippy/시각 산출, Markdown 링크/공백 검사와 실제 Claude 세션의 지침 준수 검증은 실행하지 않았다.
원격 CI 검증은 아래와 같이 완료했으며 로컬 미실행 항목과 구분한다.

- 검증 head: 96cae42c34d33c8fb61dc973926bf3b7a79df613.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/34488792970): Build & Test, Lint (fmt, clippy, WASM check),
  Native Skia, Frontend package, Archive A/B/C/D 전체 회귀 job이 모두 성공했다.
- [Skill router gate](https://github.com/edwardkim/rhwp/actions/runs/34488792579),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34488793336),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34488793054),
  [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34488793248)이 성공했다.
- 이전 9b762d6d2 및 6508db816의 Archive C는 fmt 명령 표기 계약에서 실패했다.
  96cae42c3에서 테스트 코드를 바꾸지 않고 명령/참조 안내를 보완했으며 새 Archive C job이 성공했다.
- 2026-09-10 확인 시 최신 head는 MERGEABLE/CLEAN, 성공 31개와 정책상 skipped 5개였고 실패/대기 검사는 없었다.
  이 값은 작성 시점 참고값이며 문서 후속 commit 이후 최신 head를 다시 확인한다.

자동 import와 지침 보강은 동작 개선을 위한 조치이며 오류 방지의 강제 실행 장치는 아니다.

## 최종 판정과 후속 조건

- 판정: 승인. 위 head의 원격 CI 성공과 실제 지침 보정 범위를 근거로 한다.
- 작업지시자가 CI 완료 후 merge와 후속 처리를 재지시했고 이번 PR 전용 원격 branch 정리도 승인했다.
- 로컬 문서 링크/공백 검사는 여전히 미실행이다. 이를 통과로 바꾸어 기록하지 않으며,
  실제 Claude 세션의 행동 개선까지 검증됐다고 주장하지 않는다.
- merge 전 조건: 이 기록을 포함한 최신 head의 required CI 성공, mergeability와 head SHA 재확인.
- 이 commit은 검증 head 뒤의 review/오늘할일 기록만 변경한다. 제품 코드와 검증 스킬은 다시 바꾸지 않는다.

## Merge 후 comment 및 후속 처리 계획

새 PNG/PDF나 제품 시각 검증 수치를 만들지 않는다.
과거 원 PR에 추가 comment를 게시하거나 이미 끝난 issue를 다시 종료하지 않는다.
merge 뒤 PR comment에 merge SHA, 위 CI 근거, 최종 문서 head의 CI와 로컬 검사 미실행을 구분해 기록한다.
연결된 종료 대상 issue가 없으면 과거 감사 대상 PR/issue를 다시 닫지 않는다.
review/오늘할일은 PR head에 이미 포함하므로 archive 이동과 별도 기록 PR을 반복하지 않는다.
devel을 upstream/devel로 fast-forward한 뒤 같은 merge/증거의 중복 comment가 없을 때만 게시한다.
이번 작업의 정확한 local/remote branch만 정리하고 기본 작업공간과 공유 target은 보존한다.
이번 PR의 감사/self-review/오늘할일을 재사용하며 같은 기록만을 위한 추가 문서 PR은 만들지 않는다.
공유 target과 기존 증적은 보존한다.
