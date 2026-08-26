# git add -A 금지

세션 핸드오프는 샌드박스·저널·수거물·임시 `result.json` 을 만든다. `git add -A`
는 그 전부를 스테이징한다. 금지.

## 올바른 스테이징

이름 있는 파일만 더한다.

```bash
git add -- .claude/skills/rhwp-handoff/SKILL.md
git add -- .claude/skills/rhwp-handoff/references/when-to-handoff.md
git add -- mydocs/working/agent_handoff.md
git add -- scripts/tests/test_agent_handoff_skill.py
git add -- tests/cases/agent_handoff_skill_contract.rs
```

`output/handoff/` 는 보통 커밋하지 않는다. 스킬 픽스처로 남길 것만
`fixtures/` 아래에 둔다.

## 인계 산출을 커밋해야 할 때

작업 증빙 캡슐을 남기라는 요청이 있으면 work-receipt 경로로
`*.capsule.json` 을 **이름을 지정해** 더한다. 폴더 전체를 `-A` 하지 않는다.

## 검출

`fixtures/envelopes/git_add_a_rejected.json` 은 `command: "git add -A"`,
`rejected: true`. 시나리오 카탈로그의 해당 항목도 `refuse: true`.

outgoing working doc 의 "하지 말 것" 칸에 `git add -A` 가 있어야 한다.
