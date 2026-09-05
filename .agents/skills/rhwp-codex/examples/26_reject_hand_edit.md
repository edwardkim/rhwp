# 26 — 생성 장 수기 수정 거절

갈래: **유지보수**. 장: `README.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`10_조회.md 의 JSON 오타를 고칠게.`

## 판정

거절. frontmatter 에 `generated:` 가 있으면 수기 금지.
고칠 곳은 `tools/gen_agent_codex.py` 의 LIVE/FAMILIES 다.
손글 정본은 README · 00_서문 · 01_판단트리 뿐이다.
