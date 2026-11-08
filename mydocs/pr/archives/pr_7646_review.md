---
kind: investigation
status: historical
canonical: mydocs/pr/archives/pr_7646_review.md
last_verified: 2026-10-07
---

# PR #7646 리뷰 — 완료 감사 목록 수렴과 공용 빌드 cache 지침

## 최종 판정

**승인.** 완료 감사에서 누락된 workflow 목록을 제한 재조회하는 구현·실행 계약과 모든 OS의 공용 cache 지침을 승인한다. 새 controller의 운영 효과는 **미검증**이며, 이번 자동 판정 속도를 개선 결과로 보고하지 않는다. 이는 작성자 self-review이며 GitHub 본인 PR Approve event와 구분한다. 사용자는 CI 결과와 활성화 경계를 확인한 뒤 옵션 2의 별도 review·오늘할일 기록 PR, 일반 merge 및 후속 처리를 명시 승인했다.

원 코드 PR은 최종 exact head의 CI·mergeability를 확인하고 2026-10-07 23:16:11 KST에 일반 squash merge했다. 기록 PR은 문서만 포함하는 review-only B 경로로 검증·병합하며, main 승격이나 검증 CI의 병합 후 재실행은 수행하지 않는다.

## 접수 정보

| 항목 | 확인 값 |
| --- | --- |
| PR·작성자·base | [#7646](https://github.com/edwardkim/rhwp/pull/7646) / jangster77 / devel |
| 검증 CI 구현 head | `aa376557c317b476b16cddf91056f58f8608aa15` |
| 최종 PR head | `eb9b636fe4b2b20f4db829fa8aa0d4f1c986b9ca`; 이후 변경은 cache 지침뿐이며 CI source/test 4개는 동일 |
| 검증 base | `23ac6780c4697b32f40bf7aa9a52bf92ca58b791` |
| merge commit | [`9b96fb4cd673892439afd22197db5c87c8cab04b`](https://github.com/edwardkim/rhwp/commit/9b96fb4cd673892439afd22197db5c87c8cab04b) |
| 병합 직전 | MERGEABLE / CLEAN; status rollup 29 SUCCESS, 4 SKIPPED; 실패·진행 중 없음 |
| 변경 범위 | 3 commit, 14 files: controller·helper·계약 검사 4개, 공용 cache 지침 9개, 조사 기록 1개 |
| 관련 이슈 | [#7645](https://github.com/edwardkim/rhwp/issues/7645), Refs; 사용자 후속 지시로 구현 완료 종료, main 반영 후 활성화 |
| route | collaborator self-review + 사용자 지정 옵션 2; intake/review_template/local_validation/review_only_fast_pass/post_merge 및 github_operations |

## 구현 주장과 검증

[조사·로컬 검증 기록](../../working/issue7645_ci_workflow_list.md)의 #7643 최초 감사 목록 선택 실패를 근거로 한다. 최초 raw API 응답은 확보하지 못했으므로 GitHub 내부 지연 원인을 확정하지 않는다.

`.github/workflows/ci-impact-policy.yml:371`의 목록 수집 → `scripts/ci-workflow-evidence.cjs:21`의 제한 재조회 → 기존 exact identity 선택 → 개별 run/job snapshot 검증 경로를 대조했다. audit에서 현재 head에 필요한 workflow가 목록 선택에서 누락된 경우에만 최대 4회·5초 간격·추가 요청 시작 예산 45초로 조회한다. publish는 1회이며, 이미 존재하는 진행 중/실패 실행과 정책상 미실행 workflow를 기다리지 않는다. 이전 부분 목록 뒤 API 오류·잘못된 응답이면 부분 증거를 폐기하고, 미수렴은 success를 합성하지 않는다. 개별 HTTP timeout 10초와 요청 시작 예산은 pagination을 포함한 총 wall time 45초의 보장이 아니다.

repo/branch/head/PR/base/path 선택, attempt와 전후 snapshot 검사, trusted live-base checkout, trigger·권한·concurrency·required context는 보존했다. 새 재조회가 실제 inline script에서 개별 run/job 조회까지 연결되는 검사와 기존 identity 반례를 확인했다. 제품 Rust/Studio/render 경로·기준값 변경이 없어 조판 원칙, Native/fresh WASM·Visual Sweep 및 새 fixture 선행 gate는 **비해당**이다.

| 검사 | 결과·근거 |
| --- | --- |
| 결함 음성 대조 | 기존 devel YAML과 새 재현 검사에서 목록 수집 경계의 assertion FAIL(exit 1), 교정 YAML PASS(exit 0). 모듈 미존재·환경 실패는 결함 증거에서 제외 |
| 로컬 Node | policy/classifier/controller/evidence/report 5개 모듈: 172 PASS, 0 FAIL/skip; 명령은 조사 기록에 보존 |
| 로컬 Python | 루트 Python 3.12 venv, `PYTHONUTF8=1`, workflow 계약 5개 모듈: 96 PASS |
| Actions·문법 | 검증한 actionlint v1.7.12, YAML, `node --check scripts/ci-workflow-evidence.cjs`, `git diff --check`: PASS |
| 최종 Full CI | [CI run 37631160255](https://github.com/edwardkim/rhwp/actions/runs/37631160255): A 3870, B 2379, C 1907, D 2171, 합계 **10,327 PASS / 50 skipped / 0 FAIL**. Lint·Native Skia·Frontend package gates도 SUCCESS |
| 별도 required 검사 | [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/37631160332), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/37631160368), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/37631160293), GHAS CodeQL: SUCCESS |
| Render Diff | 기존 path 정책으로 미실행. 실행·PASS로 기록하지 않음 |
| 자동 policy | [완료 감사 37633788850](https://github.com/edwardkim/rhwp/actions/runs/37633788850), attempt 1, exact 최종 head의 CI Impact Policy SUCCESS; 수동 재실행 없음 |
| cache 문서 | 변경 Markdown 10개 링크·metadata PASS. Windows의 PowerShell·Cygwin POSIX·cmd 예제에서 worktree 이동 뒤에도 기본 clone의 동일 절대 target 확인. macOS/Linux 원격 실행은 미검증 |
| 제출 tree | base `23ac6780c4697b32f40bf7aa9a52bf92ca58b791`, head `eb9b636fe4b2b20f4db829fa8aa0d4f1c986b9ca`의 실제 merge tree `6cc8bb07c843469e1edb07b3779c88a1f26110f3`: conflict·공백 오류 없음 |

증거와 재현 도구는 사용자 다운로드의 `ci-policy-completed-event`에 보존한다. 이 기록은 이전 #7643의 로컬 10,506 PASS와 이번 CI suite 수치를 혼합하지 않는다. 제품 source/test가 바뀌지 않아 완료된 고비용 검증을 반복하지 않았다. 다음 전체 nextest는 사용자 지정 `--test-threads 10`을 유지한다.

## 운영 효과와 남은 범위

코드 merge SHA의 [duration refresh 37635288121](https://github.com/edwardkim/rhwp/actions/runs/37635288121)는 SUCCESS다. exact PR head의 Full CI worker artifact B/C/D를 검증해 `ready=true`, 46 target 실측 갱신을 확인했다. 병합 후 검증 CI를 재실행하지 않았다.

Build & Test 완료는 14:05:20 UTC, policy 발행은 14:05:38 UTC로 **18초**였다. 그러나 실제 privileged controller는 main `1a76570e833917d15817415a53c09ad61ab3203f`의 YAML과 live devel `23ac6780c4697b32f40bf7aa9a52bf92ca58b791`의 기존 helper를 사용했다. 이번에는 이전 목록 누락이 관측되지 않았으며 이 18초를 새 collector의 운영 개선으로 귀속할 수 없다.

devel merge 뒤 새 helper는 trusted base에 들어갔지만, 완료 이벤트에서 새 helper를 호출하는 YAML은 정상 main 승격 후 활성화된다. main 승격은 이번 승인 범위 밖이다. 이후 실제 누락·수렴 이벤트와 exact head status를 확인해야 하며, 사용자는 구현 완료로 #7645 종료를 명시 지시했다. 종료는 운영 효과 검증 완료를 뜻하지 않으며, main 반영 후 활성화·실제 이벤트 관측은 후속 범위다. 합성 계약 검증은 충족, 실제 운영 효과는 미검증으로 구분한다. rollback은 구현 commit의 정상 revert PR이다.

각 host의 기본 rhwp clone 아래 `target/pr-review` 절대 경로를 모든 worktree·Native/WASM/lint/회귀에서 재사용한다. 실행은 순차이며 cache는 종료 후에도 보존한다. 모든 빌드가 cache hit 된다는 보장은 하지 않는다. 현재 PC의 글로벌 AGENTS 정렬은 로컬 설정이며 Git PR에는 포함하지 않았다.

## Merge 후 contributor PR comment 계획

옵션 2 기록 PR을 review-only preflight·heavy worker skip·최종 aggregate 및 전체 status로 확인하고 일반 squash merge한다. 원 코드 merge SHA의 duration refresh 성공 또는 증거 부족 사유를 확인하되 검증 CI를 재실행하지 않는다. 기록 반영 뒤 기본 clone의 devel을 fast-forward한다.

원 PR과 #7645에 한국어로 코드 merge SHA·기록 PR 링크·위 CI 링크·실제 18초와 기존 controller 사실·main 활성화 미검증 및 사용자 지시에 따른 구현 완료 종료를 남긴다. 렌더 변경이 없어 이미지 링크는 비해당이다. UTF-8 without BOM body-file로 게시하고 API의 본문·한글·BOM·물음표 치환을 재검증한다. 기록 PR에 별도 review 기록·오늘할일·issue 댓글을 다시 만들지 않는다.

마지막으로 두 PR의 MERGED·exact head·merge commit의 upstream/devel 포함·clean/소유 상태·다른 열린 PR 사용 여부를 확인한 뒤 이번 전용 worktree와 local branch 2개 및 원 저장소의 임시 remote head 2개를 SHA lease로 정리한다. 기본 작업공간, 다른 작업 자료, 다운로드 증거, 공유 `target/pr-review`는 보존한다.
