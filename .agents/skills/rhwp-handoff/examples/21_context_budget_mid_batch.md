# 21 — 배치 중간에도 예산을 보면 닫는다

트리거/갈래: `context_budget`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

폴더 200 건을 `rhwp batch info` 로 돌리다 창이 가득 찬다.
남은 목록을 `s18-failed.txt` 처럼 파일로 남기고 절차 A.
배치를 '끝낼 때까지' 붙잡지 않는다.
트랜스크립트: `fixtures/transcripts/T21_budget_mid_batch.json`.
세션 스토리 s16–s19.
