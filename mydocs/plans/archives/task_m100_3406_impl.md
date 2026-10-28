# 구현계획서 — #3406 stale PR run reaper

## workflow 계약

새 `.github/workflows/cancel-stale-pr-runs.yml`은 `pull_request`의 `synchronize`에서만 실행한다. `devel`
대상의 동일 저장소 head PR만 허용하며 source checkout 단계·shell 단계가 없다. `actions/github-script`
안에서 GitHub API만 호출한다. 외부 fork PR은 GitHub Actions가 write token을 읽기 전용으로 낮추므로 job을
명시적으로 skip한다.

| 단계 | 처리 | 안전 장치 |
| --- | --- | --- |
| 1 | live PR 정보를 조회 | closed PR이면 종료하고 event payload의 낡은 SHA를 그대로 신뢰하지 않음 |
| 2 | 같은 head branch의 `pull_request` run 열거 | `pull_requests[].number`가 현재 PR 번호인 run만 남김 |
| 3 | active + stale SHA를 선별 | current head SHA run과 완료 run은 제외 |
| 4 | 각 force-cancel 직전 live PR head 재확인 | 그 run SHA가 최신이면 건너뜀 |
| 5 | REST force-cancel 호출 및 로그 | workflow 이름을 하드코딩하지 않고 대상 run ID·SHA를 남김 |

top-level concurrency는 PR 번호별로 이전 reaper를 취소한다. 실제 CI run의 기존 concurrency 정의는
바꾸지 않는다.

## 문서 변경

`mydocs/manual/pr_review/multi_pr_update_branch.md`는 Update branch 직후 자동 reaper 완료와 stale run
상태를 먼저 확인하도록 바꾼다. reaper가 없거나 실패했을 때만 기존 force-cancel API 수동 절차를 쓴다.

## 배포

workflow는 `devel` 변경 PR로만 반영한다. `main`은 메인터너 전용 릴리즈 경로이므로 이 이슈의 변경 대상이
아니다. devel 외부 fork PR의 stale run은 기존 수동 force-cancel 절차를 적용한다.
