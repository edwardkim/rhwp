---
kind: working
status: completed
issue: 4176
last_verified: 2026-08-07
---

# Task #4176 Stage 2 - Render Diff trailing 문서 candidate 탐색 보정

## 원인

`cfe6a1ccc`는 `8119074ce`의 녹색 Render Diff 결과를 재사용한 trailing 문서 commit이다. 따라서
`cfe6a1ccc`의 Canvas job은 정상적으로 `skipped`다. 다음 문서 commit `b3d328ce5`의 preflight는 이
후보를 `canvas-visual-diff-not-success:skipped`로 실패 처리하고, 더 이전의 실제 Canvas 성공 candidate
`8119074ce`를 탐색하지 않아 Canvas를 다시 실행했다.

## 보정

- Render Diff candidate의 Canvas `skipped` 상태를 실패가 아닌 재사용 불가 후보로 분류한다.
- 후보 loop는 해당 후보를 건너뛰고, 같은 PR·branch·repository·base identity를 가진 더 이전의 실제
  Canvas 성공 후보를 계속 탐색한다.
- non-success 실패, identity 불일치, 실행 중 후보는 기존처럼 full Render Diff fallback 또는 대기 사유를
  유지한다.

## 검증 결과

- workflow 계약 테스트에 skipped candidate가 실패 분기로 들어가기 전 별도 상태로 반환되고, 후보 loop가
  계속 탐색하는 구조를 추가했다.
- `python3 -m unittest scripts/tests/test_review_only_fast_pass_workflows.py`
  `scripts/tests/test_cache_sweep_workflow.py` `scripts/tests/test_workflow_contract_wiring.py`를 실행해
  37건이 통과했다.
- `actionlint .github/workflows/render-diff.yml`,
  `python3 scripts/check_markdown_links.py --changed-from HEAD`, `git diff --check`를 실행해 통과했다.
