# 11 — 디스크가 가득하면 더 쓰지 않는다

트리거/갈래: `disk_full`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

레이아웃: `fixtures/layouts/disk-full/`.
부분 `partial_result.json` 을 닫아 성공처럼 만들지 않는다.
같은 디스크에 시트 리필하지 않는다.
working doc `status: blocked`. 표본 exit 1.
