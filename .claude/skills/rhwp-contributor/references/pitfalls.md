# 함정 — 에이전트가 절차를 깨는 자리

## 1. 낡은 fmt 명령을 게이트로 적음

`cargo fmt --check` 만 돌리고 통과했다고 PR 에 쓴다. CI Lint 가 다시
실패한다. 정본은 `cargo fmt --all -- --check`.

## 2. `git add -A`

생성물, 다른 스킬, gym, 로컬 메모가 같이 올라간다. 경로를 지정한다.

## 3. named worktree 훔침

`rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`,
`rhwp-doc-repro` 또는 `git worktree list` 의 기존 이름에 들어가
구현한다. 다른 작업이 죽는다.

## 4. DocumentCore 편집 로직 발명

이슈가 렌더 버그여도 이 스킬은 기여 **절차**다. 새 편집 연산자를
`src/document_core/` 에 짜지 않는다.

## 5. 새 CLI

`rhwp contribute` / `rhwp pr-gate` 같은 명령을  dod 에 넣지 않는다.
있는 명령만 안내한다.

## 6. 영수증 스킬 재작성

`replay` / `audit` / `lineage` 계약을 여기 복제하거나
`rhwp-work-receipt/SKILL.md` 를 고친다. 포인터만 둔다.

## 7. 중복 열린 PR

같은 주제로 두 번째 PR 을 연다. 먼저 `gh pr list --state open`.

## 8. noci 를 FAILURE 로 읽음

문서 전용에 검사가 안 뜬 것을 실패로 보고하고 재실행을 반복한다.
반대: 빨간 Lint 를 "문서라서 무시" 한다.

## 9. 한글 PR 본문을 파이프로 깨뜨림

PowerShell here-string → `gh --body-file -`. BOM/`??` 가 생긴다.
UTF-8 without BOM 파일 + `--body-file`.

## 10. base=`main`

GitHub 기본 선택이 `main` 이다. `--base devel` 을 명시한다.

## 11. fetch 없이 로컬 devel 에서 분기

이미 낡은 devel. `git fetch upstream devel` 이 단 3 의 첫 명령이다.

## 12. 시각 근거 없이 레이아웃 PR

페이지 수가 바뀌는 패치에 SVG 전후가 없다. 게이트 미완.

## 13. 관련 시험 없이 "테스트 통과"

어떤 명령을 돌렸는지 본문에 없다. 관련 `cargo test` 이름을 적는다.

## 14. 이슈 없이 PR

`closes #` 가 비어 있다. 단 1 을 먼저 닫는다.

## 15. gym 으로 기여를 채점

이 경로는 Maker seat 실 PR 이다. gym pack 을 만들지 않는다.
