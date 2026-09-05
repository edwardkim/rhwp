# 23 — 시트만 바꾸고 트리는 그대로

트리거/갈래: `seat_refill`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

`git worktree list` 로 지금 트리가 금지 목록이 아닌지 확인.
같은 isolation 에서 세 파일을 읽는다.
새 트리가 필요하면 빈 경로만. `rhwp-handoff` 를 비우지 않는다.
트랜스크립트: `T23_refill_isolation.json`.
