# #3508 1·2단계 — 구현과 검증

Issue: #3508
브랜치: `task/3508-fork-pr-stale-cancel`

## 구현

### A. `cancel-stale-pr-runs.yml` — `pull_request_target` 전환

- 트리거 `pull_request` → `pull_request_target` (base 브랜치 devel 의 정의로 실행, fork PR
  에서도 write 토큰 유지 — main 반영 불필요).
- same-repo 가드(`if: head.repo == github.repository`) 제거.
- stale 판정을 `run.pull_requests[].number` 매치에서 **`head_repository.full_name` +
  `head_branch` + `head_sha != live head`** 대조로 교체 — fork run 은 `pull_requests` 배열이
  비는 GitHub quirk 로 종전 필터가 fork run 을 잡지 못한다. 다른 fork 의 동명 branch 와
  섞이지 않도록 head_repository 대조를 필수로 한다.
- head repo 가 삭제된 PR(`pull.head.repo == null`) 조기 종료 분기 추가.
- **안전 경계 불변**: PR source checkout 없음, PR 제공 코드 실행 없음, 순수 API 호출.
  "이 경계를 바꾸는 수정은 보안 리뷰 없이 금지" 를 워크플로 주석으로 명시.
- 최신 head 보호(취소 직전 live 재확인)·concurrency·최소 권한은 종전 그대로.

### B. `scripts/cancel_stale_pr_runs.sh <PR번호> [--dry-run]`

multi_pr 2.5 수동 절차의 원커맨드화. 현재 head 확인 → 같은 PR head 의 이전 SHA active run
나열 → force-cancel(일반 `gh run cancel` 미시도, 2.5 규정) → `completed/cancelled` 재확인
(6회×10초) → run URL·SHA 출력(review 기록 요건). 취소 직전 live head 재이동 감지 시 중단.

### C. `multi_pr_update_branch.md` 2.5 갱신

reaper 가 fork PR 을 커버함을 반영하고, 수동 폴백 1순위를 B 스크립트로, script 불가 환경만
종전 수동 API 절차로 남겼다.

## 검증

| 항목 | 결과 |
|---|---|
| 워크플로 YAML 파싱 | OK (actionlint 미설치 — 파싱 검증만) |
| `bash -n` | OK (shellcheck 미설치) |
| 문서 링크·메타데이터 게이트 | 변경 5파일 이상 없음 / 메타데이터 이상 없음 |
| B 실검증 — fork PR 열림(#2529, #3456) | head repo/branch 정확 해석, "active run 없음" exit 0 |
| B 실검증 — 닫힌 PR(#3499) | "OPEN 아님 — 중단" exit 1 |
| **jq 필터 필드 실데이터 검증** | #3456 branch 의 실제 run 이 `head_repository.full_name`·`head_branch` 를 기대 형태로 반환, completed run 정확 제외 |

### 검증 한계 (계획서 3절 그대로)

- **stale run 실취소 경로는 미실증** — 실제 update branch 경합 없이 안전하게 만들 수 없다.
  필터 로직은 reaper 와 동일 구조이고 필드명은 실데이터로 확인했다.
- **A 는 merge 전 발동 불가** — `pull_request_target` 은 base 브랜치 파일로 동작한다.
  merge 후 첫 fork PR update branch 에서 실증하고, 그때까지 이슈 #3508 을 닫지 않는다.
  #3406 전례에 따라 jangster77 격리 저장소 사전 검증 요청을 PR 본문에 선택지로 남긴다.

## 다음

3단계 — PR 생성(**별도 승인**) → CI → merge(**별도 승인**). 4단계 — merge 후 fork PR
update branch 실증 → 이슈 close(**승인**).
