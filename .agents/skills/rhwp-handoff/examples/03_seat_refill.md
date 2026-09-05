# 03 — 시트 리필은 폴더 경로만

트리거/갈래: `seat_refill`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

후임 프로세스에게 주는 것:

```
output/handoff/t-ord/
mydocs/working/agent_handoff.md
```

주지 않는 것: 원본 절대경로, `.git` 권한, 이름 붙은 워킹트리
(`C:\Users\swsz9\rhwp`, `rhwp-handoff`). 같은 isolation 에서
읽는 것이 기본이다 (23).

오케스트레이터 입력 경계와 같다 — task `inputs` 에 없는 파일은
후임 sandbox 에도 없다.
