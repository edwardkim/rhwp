# #665 — PR 제출 준비

Issue: #665

## 적용 절차와 승인 범위

- base route: `collaborator_self_merge` — 인증 계정 edwardkim의 본인 task PR 준비.
- modifiers: `intake_and_review`, `local_validation`, `visual_fixture_evidence`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 역할·보조 문서,
  `docs_and_git_workflow.md`, `visual_verification_governance.md`.
- 준비 전 head: `5dddc5d315113b2c56f0acb190cd277b2e4aea73`.
- 제품 source: `d78048a6f0e5f71aa3bdb7d32e703bd5971e3019` 이후 동일.
- 원격 처리 승인 전에는 PR 번호를 예측하거나 번호 기반 review·오늘할일을 만들지 않는다.

## 사전 조건 확인 — 2026-11-03

`gh api user`로 edwardkim을 확인하고 모든 base·draft를 포함한 본인 열린 PR을 조회했다.
결과는 0건이며 선행 보정 대상은 없다. 원격 devel은
`ea86816b5316dfeeb47ecf20df97abfd4b02da73`로 유지됐다. rebase는 필요하지 않았다.

전체 회귀 12,378개·Native 공식 3종 4,064개·Rust lint·최적화 fresh WASM·보안·성능 검증은
[최종 보고서](../report/task_m100_665_report.md)의 실제 결과를 재사용한다. 제품·테스트 source를
바꾸지 않았으며 같은 검사를 반복하지 않았다.

제출 규칙에 맞춰 빠져 있던 동일 입력의 devel 비교 27개를 추가 출력했고, 기존 3개는 재사용했다.
15개 시나리오 × Native/fresh WASM의 30개 전쪽 비교 모두 하락폭 0.00%p다.
입력/PDF·글꼴·DPI·프로필과 각 원본 TSV는
[비교 기록](../tech/investigations/issue-665/devel-comparison.json)에 있다.

신규 조사 README의 필수 front matter와 보고서의 Issue 표기를 보완했다.
실제 코드·회귀 assertion·입력 파일·기준 PDF·기준값은 바꾸지 않았다.

## 원격 처리 준비

PR 본문과 실행 명령은 ignored `output/issue665/pr-body.md`,
`output/issue665/pr-open-commands.sh`에 준비한다. 본문에는 실제 제출 SHA로 고정한
입력/HWPX/HWP/PDF·원본 TSV·대표 PNG, 독립 Print 출처·전쪽 비교·실제 검증 결과를 넣는다.

승인 뒤 `task/665-ingest-images`를 upstream 작업 branch에 push하고 devel 대상 Open PR을
생성한다. 생성 직전 본인 열린 PR·base/head·충돌 여부를 다시 확인한다.
PR 채번 뒤 정확한 code head CI가 성공하면 번호 기반 self-review와 필요한 기록을 작성한다.
remote push·Open PR 생성·후속 self-review 기록 push는 별도 승인 대기다.
