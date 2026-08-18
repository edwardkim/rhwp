# 08 — 캡슐이 없으면 추측 재개 금지

트리거/갈래: `missing_capsule`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

레이아웃: `fixtures/layouts/missing-capsule/` (result 만 있음).

수거물이 있으면 위임 결과는 쓸 수 있다. 세션 체인은 끊긴다.
빈 `workCapsule` 을 만들지 않는다.
표본 exit 1: `fixtures/exceptions/missing_capsule.json`.
