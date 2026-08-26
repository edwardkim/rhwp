# 26 — 조례 여정 s01–s08

트리거/갈래: `--parent`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

트리아지 → 구조 → 표 추출 → 연도 정정 → CSV 되돌리기 →
누름틀 조사 → dry-run → 저장.
각 캡슐은 이전 outputSha256 을 다음 inputSha256 으로 받는다.
`fixtures/capsule_index.json` 의 해당 children 을 본다.
이 여정은 실사용 레시피이지 gym pack 이 아니다.
