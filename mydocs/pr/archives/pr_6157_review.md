---
kind: pr-review
status: approved-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6157 review - HWP 2024 MCP 손상 입력 안내

## 경로와 기준

- PR: https://github.com/edwardkim/rhwp/pull/6157
- 작성자: `jangster77` self PR, reviewer 별도 지정 없음
- base: `devel`, 작업 시작 기준 `upstream/devel@9c9ad3485dd1`
- code candidate: `b36904e2fd3bf4341028be2c9fe0cdc69fcbfbac`
- PR 생성 시점 상태: non-draft, `mergeable` 및 `mergeStateStatus`는 GitHub 계산 대기 상태였다.

## 변경 범위

- `mydocs/manual/mcp_hwp2024Convert_usage.md` 한 파일만 변경했다.
- 손상 HWP 구조 사전 검증 오류가 terminal `failed`로 끝나며 download 대상이 아님을 명시했다.
- timeout 또는 engine 변경 재시도를 금지하고 원본 재수령·복구 절차를 안내했다.
- 배포 MCP에서 PDF·HWPX 비동기 job이 worker 시작 전 손상 오류로 종료된 증적을 기록했다.

## 검증

- `git diff --check` 통과
- `cargo fmt --all` 실행
- `cargo fmt --all -- --check` 통과
- 변경 범위가 `mydocs` 한정이므로 `local_validation.md` 4.3의 Cargo 테스트 생략 규칙을 적용했다.
- renderer, source, fixture, PDF 산출물은 변경하지 않았으므로 시각 검증은 대상이 아니다.

## 검토 결론

문서가 실제 배포 동작과 일치하며, 인증 정보와 server 주소를 포함하지 않는다. 최신 trailing head의
required CI가 성공하고 작업지시자의 merge 승인이 있을 때 병합할 수 있다.
