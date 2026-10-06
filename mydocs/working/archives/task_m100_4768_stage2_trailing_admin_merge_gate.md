---
kind: working
status: active
issue: 4768
stage: 2
last_verified: 2026-08-14
---

# #4768 Stage 2: 최신 upstream 기록과 maintainer merge gate

## 기준선

code candidate `712873a`은 최신 `upstream/devel@b37c0a79` 위에서 CI를 통과했다. 오늘할일은 이 기준선의
내용을 다시 읽어 다른 PR 기록을 보존한 뒤 #4769 항목을 추가한다.

## trailing 기록

- PR review와 오늘할일은 code candidate 뒤의 문서-only commit으로 추가한다.
- 새 head의 CI와 mergeability를 다시 확인해, code candidate 결과와 trailing 문서 결과를 분리한다.

## maintainer 예외 계약

- `--admin`은 명시 지시, maintainer 권한, 두 head의 녹색 CI, `CLEAN` merge 상태가 함께 있을 때만 허용한다.
- source·test·fixture·workflow·baseline이 trailing commit에 섞이거나 check가 실패·대기 중이면 이 예외를 쓰지
  않는다.
- 권한이 없는 collaborator의 `--admin` 실패는 정상 squash merge 경로로 처리한다.
