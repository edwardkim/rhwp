# 스테이징 — `git add -A` 금지

기여 커밋은 **이름을 댄 파일만** 올린다. `git add -A`, `git add .`,
`git add -u` 로 워크트리 전체를 긁지 않는다.

## 이유

- 스파스 체크아웃·생성물·로컬 메모가 같이 들어간다
- 다른 작업의 미커밋 파일이 섞인다
- 리뷰어가 이슈 범위를 재구성할 수 없다

## 하는 법

```bash
git status --short
git add -- .claude/skills/rhwp-contributor/SKILL.md
git add -- .claude/skills/rhwp-contributor/references/
git add -- .claude/skills/rhwp-contributor/examples/
git add -- .claude/skills/rhwp-contributor/fixtures/
git add -- mydocs/working/agent_contributor.md
git add -- tests/cases/agent_contributor_skill_contract.rs
git add -- scripts/tests/test_agent_contributor.py
git diff --cached --name-only
```

PowerShell 에서 여러 경로를 한 줄로 넘겨도 된다. 중요한 것은 **글롭
전체가 아니라 이 커밋이 소유하는 경로**다.

## 확인

```bash
git diff --cached --name-only
```

여기에 `gym/`, `src/document_core/`, 다른 스킬, 열린 PR 전용 파일이
있으면 reset 하고 다시 고른다.

## 닫는 증거

인덱스에 이슈 범위 파일만 있다. `add -A` 가 셸 히스토리에 없다.

예제: [08_never_git_add_all.md](../examples/08_never_git_add_all.md),
[23_named_file_stage.md](../examples/23_named_file_stage.md).
