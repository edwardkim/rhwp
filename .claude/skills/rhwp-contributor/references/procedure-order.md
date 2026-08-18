# 필수 순서 — 이슈 → 분석 → 브랜치 → 구현 → 게이트 → 영수증 → 문서 → PR

기여 1건은 아래 8단을 **이 순서대로** 닫는다. 뒤 단계를 먼저 하고 앞 단계를
끼워 넣지 않는다. 특히 PR 을 연 뒤에 이슈를 만들거나, fmt 없이 `gh pr create`
하지 않는다.

## 8단

| # | 단 | 닫는 증거 | 건너뛰면 |
|---|----|-----------|----------|
| 1 | 이슈 | `gh issue view` 번호, DoD, 중복 PR 없음 | 리뷰어가 왜 바꾸는지 모름 |
| 2 | 분석 | 이슈 본문/댓글에 정본·계약 시험 인용 | 원인 없는 패치 |
| 3 | 브랜치 | `upstream/devel` 에서 만든 isolation worktree | 본진 오염, named 훔침 |
| 4 | 구현 | 경로를 지정한 `git add`, DocumentCore 미발명 | 범위 밖 파일 |
| 5 | 로컬 게이트 | `cargo fmt --all -- --check` + clippy + 관련 test + (해당 시) 시각 | CI Lint 실패 |
| 6 | 작업 영수증 | `rhwp replay --capsule` 포인터 (문서 편집 시) | 증빙 없음 (권장) |
| 7 | 처리 결과 | `mydocs/working/<이름>.md` | 규모 있는 변경의 맥락 소실 |
| 8 | 한국어 PR | `--base devel` · `--body-file` · `closes #` · 첫 칸 fmt | 접수 거부 |

## 하드 규칙

1. 단 5 의 HARD GATE 는 `cargo fmt --all -- --check` 다.
   `cargo fmt --check` 는 낡은 표기다.
2. 단 8 은 단 5 가 통과한 뒤에만 연다.
3. 단 3 은 `git fetch upstream devel` 을 먼저 한다.
4. 단 4 는 `git add -A` 를 쓰지 않는다.
5. 단 3 은 이미 있는 named worktree 를 훔치지 않는다.

## 권위

- 순서·브랜치: `AGENTS.md`, `CONTRIBUTING.md`
- 범위별 검증: `mydocs/manual/pr_review/local_validation.md` §4.3
- PR 본문: `.github/pull_request_template.md` — 첫 체크박스 = fmt 게이트
- 영수증: `AGENTS.md` 작업 증빙 절, 스킬 `rhwp-work-receipt` (포인터만)

## 이 스킬이 닫지 않는 것

리뷰 승인, merge, collaborator 의 오늘할일, gym 채점은 이 순서 밖이다.

픽스처: `fixtures/checklists/step_01_issue.json` … `step_08_pr.json`.
예제: [24_full_procedure_walkthrough.md](../examples/24_full_procedure_walkthrough.md).
