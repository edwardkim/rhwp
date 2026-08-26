---
name: rhwp-skill-author
description: 새 Claude 스킬(.claude/skills/*/SKILL.md)을 만들거나 고칠 때 카탈로그·intent·그래프에 등록하고 3-pass 게이트를 명령마다 세 번 통과시킵니다. 트리거 — "새 스킬", "스킬 만들어", "create a skill", "SKILL.md 작성". 새 rhwp CLI·이웃 스킬 본문·test_route.py 수정은 책임 밖입니다.
---

# rhwp-skill-author — 스킬 작성 게이트

새 스킬은 감으로 넣지 않는다. `.claude/skills/<id>/SKILL.md` 를 쓰거나
고친 뒤 **같은 계약**을 세 번 통과해야 끝이다. 이 스킬은 새 rhwp CLI 를
만들지 않고, 다른 스킬 본문과 `test_route.py` 를 고치지 않는다.

권위: [`tools/skill_router/README.md`](../../../tools/skill_router/README.md).
기여 PR 쪽 스킬 경로 게이트는 [`rhwp-contributor`](../rhwp-contributor/SKILL.md).
요청 라우팅은 [`rhwp-skill-router`](../rhwp-skill-router/SKILL.md).

등록: `CAP-5706` 작업 묶음 / `rhwp-skill-author`. 전용 Issue 가 생기기
전에는 새 CAP 번호를 발명하지 않는다.

## 절차

1. `.claude/skills/<id>/SKILL.md` 를 만든다. frontmatter 에 `name`(폴더명과
   동일)과 `description`(20자 이상) 이 있어야 한다. 본문에 실행 가능한
   ASCII 소문자 rhwp 명령이 있어야 한다. 자리표시는 세지 않는다.

2. 카탈로그·intent·그래프에 **한 줄씩** 등록한다.
   - `tools/skill_router/catalog.json` — unique `id`
   - `tools/skill_router/intents.py` — INTENT_SPEC 하나
   - `tools/skill_router/graph.py` — builder 하나
   - `mydocs/manual/agent_capability_registry.md` — 새 unique CAP 이 필요할
     때만 한 행. ID 를 복제하지 않고, 한 칸에 마크다운 링크를 두 개 넣지 않는다.

3. 프로브로 스킬이 문서 경로를 다루는지 확인한다.

```bash
rhwp capabilities
rhwp info <파일> --json
rhwp export-svg <파일> -p 0 --profile print
```

## 3-pass 게이트 — 끝내기 전에 명령마다 세 번

`.claude/skills/*/SKILL.md` 를 만들거나 고친 뒤에는, 스킬을 끝났다고
보기 전에 아래를 **명령마다 세 번** 돌린다.

```bash
python tools/skill_router/gate_new_skill.py
python -m unittest tools/skill_router/test_route.py
cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture
```

한 번이라도 실패하면 고치고, **연속 3회 통과**할 때까지 다시 돈다.
`skills_have_valid_frontmatter_and_are_executable` 가 실패하면 하드
페일이다. 새 rhwp CLI 를 만들지 않는다.

## 하지 않는 것

- 새 rhwp CLI 발명
- 다른 스킬의 SKILL.md 수정
- `test_route.py` 수정
- `git add -A` · push
- gym 경로로 대체
