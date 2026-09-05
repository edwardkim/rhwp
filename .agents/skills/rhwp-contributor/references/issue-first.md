# 1단 — 이슈 선등록

코드를 쓰기 전에 이슈가 있어야 한다. 이슈 번호 없이 연 PR 은 이 스킬의
완주가 아니다.

## 하는 일

1. `gh issue list --repo edwardkim/rhwp --search "<키워드>" --state all`
   로 같은 주제가 이미 있는지 본다.
2. `gh pr list --repo edwardkim/rhwp --search "<키워드>" --state open`
   로 **열린 PR** 이 있는지 본다. 있으면 새 PR 을 만들지 않는다
   ([exceptions.md](exceptions.md) 중복 경로).
3. 없으면 `gh issue create` 또는 이미 열린 이슈 번호를 쓴다.
4. 이슈 본문에 무엇을 / 왜 / 판단 근거 / DoD 를 남긴다.

## 이슈 본문 최소 칸

| 칸 | 내용 |
|----|------|
| 왜 | 사용자·에이전트가 막히는 지점 |
| 범위 | 만질 경로, 만지지 않을 경로 |
| DoD | 검증 명령과 제출 형태 (PR base=`devel`, 한국어) |
| 금지 | gym, 새 CLI, DocumentCore 발명, `git add -A` 등 |

## 명령

```bash
gh issue list --repo edwardkim/rhwp --search "contributor" --state open
gh pr list --repo edwardkim/rhwp --search "contributor" --state open
gh issue view <N> --repo edwardkim/rhwp
```

이슈가 이미 있으면(`#5322` 처럼) **만들지 않는다.** 그 번호를 쓴다.

## 닫는 증거

- 이슈 번호 `N`
- 중복 열린 PR 없음 (또는 예외 경로로 그 PR 에 합류)
- DoD 가 이슈 본문에 있다

예제: [01_issue_first.md](../examples/01_issue_first.md),
[02_duplicate_open_pr.md](../examples/02_duplicate_open_pr.md).
