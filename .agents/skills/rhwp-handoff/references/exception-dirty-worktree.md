# 예외: dirty named worktree

## 신호

`git worktree list` 의 이름 붙은 경로
(`C:\Users\swsz9\rhwp`, `rhwp-desk*`, `rhwp-handoff`,
`rhwp-scaffold-final`, `rhwp-doc-repro`, 그 밖에 이미 브랜치가 앉은 트리)에
커밋되지 않은 변경이 있고, 누군가 그 트리를 핸드오프 자리로 쓰려 한다.

## 하지 않는 것

- `git checkout -- .` / `git reset --hard` 로 자리를 만든다
- `git checkout feat/agent-handoff` 로 그 트리의 브랜치를 갈아끼운다
- dirty 파일을 인계 묶음인 척 `result.json` 옆에 복사한다
- 다른 에이전트의 이름 붙은 트리를 훔친다

## 하는 것

1. 그 트리를 건드리지 않는다
2. 빈 경로에 새 isolation worktree 를 `upstream/devel` 에서 만든다
3. 인계 묶음이 **이미 isolation 쪽에** 있으면 그걸 읽는다
4. 인계 묶음이 dirty named 트리에만 있으면 사람에게 알린다.
   파일을 몰래 복사하지 않는다 (원 작업자의 미커밋 노동)
5. 표본 exit 2 (사용법: 잘못된 자리) —
   `fixtures/exceptions/dirty_named_worktree.json`,
   `fixtures/envelopes/dirty_named_worktree.json`

## 워크스루

[`../examples/10_dirty_named_worktree.md`](../examples/10_dirty_named_worktree.md).
레지스트리: `fixtures/layouts/forbidden-worktrees/registry.json`.
