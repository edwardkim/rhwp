---
kind: working
status: active
canonical: mydocs/working/issue7645_ci_workflow_list.md
last_verified: 2026-10-07
---

# 완료 감사의 workflow 목록 수렴 (#7645)

Issue: [#7645](https://github.com/edwardkim/rhwp/issues/7645). 사용자 지시: PR #7643을 기준으로 빠른 CI Impact Policy 판단 가능성을 검토하고 이슈 등록 후 개선. 최신 기준 `upstream/devel`은 `23ac6780c4697b32f40bf7aa9a52bf92ca58b791`이다.

## 근거와 계획

[#7643 감사 attempt1](https://github.com/edwardkim/rhwp/actions/runs/37625746256/attempts/1)은 완료 이벤트를 처리했지만 workflow 세 개가 목록 선택에서 누락되어 pending을 발행했다. [동일 감사 attempt2](https://github.com/edwardkim/rhwp/actions/runs/37625746256/attempts/2)는 같은 PR/head에 success를 발행했다. GitHub 내부 지연 원인이나 최초 전체 API 응답은 확정하지 않는다. 기존 run/job 수렴 helper는 `selectLatestWorkflowRun`이 null을 반환하는 경계에 도달하지 않는다.

O3 실행 계약 변경이다. 완료 이벤트의 audit에서 필요한 workflow 목록만 제한 재조회하고, 선택은 기존 repo/branch/head/PR/base/path 계약을 그대로 사용한다. 최대 4회, 5초 간격, 추가 조회를 시작하는 경과 예산 45초를 적용한다. API 요청에도 timeout을 설정한다. 목록이 존재하면 실제 run/job 검증으로 바로 넘어가며, 실제 실행 중인 worker를 기다리거나 worker를 재실행하지 않는다. 끝까지 누락되면 pending을 유지하고 exact 감사 수동 재실행 경로를 남긴다.

수정 전/후에 다음 경계를 검증한다: 목록 누락 뒤 수렴, 계속 누락, 오류 후 수렴/지속 오류, 잘못된 head/PR/base/repository/path, 이미 존재하는 진행 중/실패 실행, 선택 후 run/job 재수집, 실제 YAML inline script의 helper 연결. 관련 Node 및 Python workflow 계약·YAML·actionlint를 실행한다. 제품 소스·Rust/WASM·조판에는 비해당이며 고비용 제품 회귀를 반복하지 않는다.

권한·trigger·concurrency·required context·privileged checkout의 trusted base 경계를 바꾸지 않는다. `devel` 반영과 default branch `main`의 workflow 활성화를 구분한다. 이 작업의 사용자 지시는 이슈 게시와 로컬 개선을 승인했으며, 별도 PR 생성·원격 push·main 승격·merge는 아직 실행하지 않는다. 정상 승격 후 실제 완료 이벤트의 status 수렴을 운영 확인해야 한다. rollback은 해당 개선 commit의 revert다.

## 실행 결과

이슈 등록과 assignee `jangster77`, UTF-8 본문 보존을 확인했고 별도 `codex/ci-policy-completed-event` 작업 브랜치를 최신 devel로 fast-forward했다.

`collectWorkflowRunList`를 기존 evidence 모듈에 추가했다. 완료 audit에서 `expectedWorkflowMap`으로 필요한 현재 head의 workflow를 정하고, 기존 `selectLatestWorkflowRun`의 exact identity 선택을 목록 수렴 조건과 실제 수집에 함께 사용한다. 최초 publish는 한 번만 조회한다. 누락 목록에는 최대 세 번의 5초 대기가 추가되지만, 이미 존재하는 진행 중/실패 실행과 path 정책으로 제외된 workflow는 목록 대기를 추가하지 않는다. 응답이 수렴하면 기존 개별 run/job snapshot 검증을 계속한다. 오류나 잘못된 목록 응답은 이전 부분 증거를 유지하지 않는다. 경과 예산은 추가 조회 시작을 제한하며 이미 실행 중인 pagination/HTTP 요청을 강제 종료하는 전체 wall-time 보장은 아니다.

검증 환경: Windows PowerShell, 저장소 Python 3.12 venv, Node 내장 test runner. 제품 Rust/WASM/조판·시각 증적과 Cargo 공유 cache는 변경하지 않았다.

| 검사 | 결과와 의미 |
| --- | --- |
| 실제 workflow inline script 음성 대조 | 기존 devel YAML + 새 재현 검사: 의도한 목록 수집 경계에서 assertion FAIL, exit 1. 교정 YAML: PASS, exit 0. 환경/모듈 미존재 실패를 결함 증거로 쓰지 않았다. |
| Node policy/classifier/controller/evidence/report | `node --test scripts/tests/ci-workflow-evidence.test.cjs scripts/tests/ci-impact-controller-contract.test.cjs scripts/tests/ci-impact-policy.test.cjs scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-report.test.cjs`: 172 PASS, 0 FAIL/skip. |
| Python workflow 계약 | `PYTHONUTF8=1`과 루트 `venv/Scripts/python.exe -m unittest`로 policy/impact/wiring/review-only/trusted-postmerge 5개 모듈 실행: 96 PASS. 최초 기본 CP949 실행의 기존 문서 읽기 오류 1건은 환경 오류로 분리하고 UTF-8 실행을 완료했다. |
| YAML·Actions lint | SHA-256 검증한 Windows actionlint v1.7.12로 변경 controller를 검사: PASS. shellcheck/pyflakes는 이 변경의 JavaScript 실행 계약에 비해당으로 비활성화했다. |
| JavaScript 문법·공백 | `node --check scripts/ci-workflow-evidence.cjs`, `git diff --check`: PASS. |

계약 검사에는 전체/부분 목록 누락 뒤 수렴, 지속 누락·API 오류·잘못된 응답, 이전 부분 목록 폐기, 잘못된 repo/head/branch/path/PR/base, 정상 진행 중/실패 반환, 정책상 미실행 경로, 최대 횟수·경과 예산이 포함된다. 실제 controller 스크립트에서 수렴 뒤 세 workflow의 개별 run/job 조회까지 이어지는지 확인했다. 감사의 success를 합성하거나 필수 worker·실패를 면제하지 않는다.

검사 로그와 재현 도구는 사용자 다운로드의 `ci-policy-completed-event` 폴더에 보관한다. 첫 감사의 raw API 목록은 없으므로 모의 empty/partial/mismatched 목록의 통과를 GitHub 내부 지연 원인의 확정으로 바꾸지 않는다. 현재는 로컬 개선·검증 완료이며 PR Actions와 main 활성화 뒤 실제 이벤트 수렴은 미검증이다. 이슈는 운영 확인까지 열린 상태로 유지한다. 다음 단계는 별도 승인된 원격 push와 `devel` 대상 PR, 정상 main 승격 후 동일 이벤트 관찰이다.
