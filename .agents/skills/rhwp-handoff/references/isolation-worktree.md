# 이름 붙은 워킹트리를 훔치지 않는다

세션 핸드오프는 **isolation worktree** 에서 한다. never steal named worktrees.
이미 이름과 브랜치가 붙은 트리를 checkout 하거나 reset 해서 자리를 만들지 않는다.

## 금지 경로 (이 스킬 픽스처와 동일)

`fixtures/layouts/forbidden-worktrees/registry.json` 정본.

- `C:\Users\swsz9\rhwp`
- `C:\Users\swsz9\rhwp-desk*`
- `C:\Users\swsz9\rhwp-handoff`
- `C:\Users\swsz9\rhwp-scaffold-final`
- `C:\Users\swsz9\rhwp-doc-repro`

`rhwp-handoff` 는 이 스킬 이름과 닮았지만 **이미 다른 브랜치가 앉아 있는
이름 붙은 트리**다. 세션 인계를 위해 그 폴더를 비우지 않는다.

다른 에이전트가 쓰는 `rhwp-agent-*` 트리도 이름이 있으면 훔치지 않는다.

## 올바른 시작

```bash
git -C <기존 클론> fetch upstream devel
git -C <기존 클론> worktree add -b feat/agent-handoff <빈 경로> upstream/devel
```

`<빈 경로>` 는 위 금지 목록이 아니고, `git worktree list` 에 없는 새 디렉터리다.

후임 세션이 같은 작업을 이어 받을 때:

- **같은 isolation 트리**에서 파일을 읽는다 (권장)
- 새 isolation 이 필요하면 새 경로를 만들고, 인계 묶음만 복사한다
- 이름 붙은 트리로 `git checkout feat/…` 하지 않는다

## dirty named worktree

이름 붙은 트리에 커밋되지 않은 변경이 있으면 그 트리는 **인계 대상이 아니다**.
그 더러움을 정리하려고 `git checkout -- .` / `git reset --hard` / 브랜치 전환을
하지 않는다. 예외 갈래:
[`exception-dirty-worktree.md`](exception-dirty-worktree.md).

## 시트 리필과의 관계

시트 리필은 프로세스를 바꾼다. 워킹트리를 바꾸라는 뜻이 아니다. 후임에게
주는 것은 `output/handoff/<taskId>/` 와 working doc 경로다.
