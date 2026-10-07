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

계획 단계: 이슈 등록과 assignee 확인, 별도 `codex/ci-policy-completed-event` 작업 브랜치를 최신 devel로 fast-forward했다. 구현 및 검증 결과는 다음 단계에서 이 문서에 기록한다.
