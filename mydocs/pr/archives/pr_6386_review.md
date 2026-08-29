---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6386
issue: null
author: jangster77
---

# PR #6386 review - HWP5 무인 스크립트 사전 차단 안내

## 라우팅

- PR: [#6386](https://github.com/edwardkim/rhwp/pull/6386)
- 작성자·self-review: `jangster77`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.
- base: `devel`, 기준: `upstream/devel@97c4d7155`.
- code candidate: `3387bac7182d7792fe8e2b1590dc53dad018cd52`.
- 범위: `mydocs/manual/mcp_hwp2024Convert_usage.md` 사용자 가이드 한 파일만 변경한 review-only PR이다.
- 관련 issue: 없음. 배포된 HWP 2024 MCP의 `OnDocument_Open` 무인 사전 차단 동작을 사용자 가이드에 반영한다.
- 적용 절차: `AGENTS.md`, `mydocs/manual/pr_review_workflow.md`,
  `mydocs/manual/pr_review/intake_and_review.md`,
  `mydocs/manual/pr_review/collaborator_self_merge.md`,
  `mydocs/manual/pr_review/review_only_fast_pass.md`,
  `mydocs/manual/pr_review/post_merge.md`.

## 검토 판단

**수용 및 squash merge 권고.** 기존 가이드는 HWP5 구조 손상만 즉시 실패로 설명해, 문서 열기 스크립트가
무인 service에서 실행·승인되지 않는 별도 사유를 사용자가 구분할 수 없었다. 변경은 이를 손상과 분리하고,
암호 HWP5의 전처리 후 재검사, terminal `failed`, 결과물 없음, download 금지를 실제 service 계약에 맞게
설명한다. client artifact·server 설정·변환 source는 바꾸지 않는다.

## 검증과 위험 판정

- `venv\Scripts\python.exe scripts\check_markdown_links.py mydocs\manual\mcp_hwp2024Convert_usage.md`: 통과.
- `venv\Scripts\python.exe scripts\check_document_metadata.py`: 통과.
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`: 통과.
- PR 작성 시점의 code candidate는 review-only fast-pass로 CI preflight와 Build & Test aggregate가 성공했고,
  CodeQL·Adapter inter-diff·Proptest roundtrip의 preflight도 성공했다. heavy worker의 skipped는 문서 전용
  범위에 따른 정상 결과다.
- renderer, parser, sample, 기준 PDF, workflow를 변경하지 않는다. 사용자 화면·출력 산출물 주장을 하지 않으므로
  visual sweep은 적용 대상이 아니다.
- 최종 merge 전에는 이 trailing review 기록이 포함된 최신 PR head의 fast-pass CI와 mergeable 상태를 다시
  확인한다.

## 후속 처리

- user 승인에 따라 최신 head가 녹색이면 squash merge한다.
- merge 뒤에는 `upstream/devel` fast-forward와 이번 작업 브랜치 정리를 수행한다.
