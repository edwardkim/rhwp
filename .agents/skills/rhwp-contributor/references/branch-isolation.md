# 3단 — 브랜치 (`upstream/devel` 에서)

작업 브랜치는 항상 최신 `upstream/devel` 에서 만든다. base 는 `devel` 이다.
`main` 에서 시작하지 않는다.

## 원격

| remote | 뜻 |
|--------|-----|
| `upstream` | `edwardkim/rhwp` |
| `origin` | 기여자 포크 (`kevin9327/rhwp` 등) |

```bash
git fetch upstream devel
git rev-parse upstream/devel
```

## 브랜치 만들기

isolation worktree 안에서:

```bash
git worktree add -b feat/<주제> <새경로> upstream/devel
```

이미 브랜치 이름만 필요하면:

```bash
git switch -c feat/<주제> upstream/devel
```

이 스킬의 주제 브랜치 관례는 `feat/agent-contributor` 다. 다른 주제는
이슈가 정한 이름을 쓴다.

## 하지 않는 것

- 오래된 로컬 `devel` 에서 분기하지 않는다. 반드시 `fetch` 후 `upstream/devel`.
- 본진 `C:\Users\swsz9\rhwp` 에서 바로 구현하지 않는다.
- 다른 작업의 named worktree 를 checkout 해서 쓰지 않는다
  ([isolation-worktree.md](isolation-worktree.md)).

## 닫는 증거

```
git status -sb          # feat/<주제>... 또는 로컬 브랜치
git merge-base --is-ancestor upstream/devel HEAD
```

예제: [04_branch_from_devel.md](../examples/04_branch_from_devel.md).
