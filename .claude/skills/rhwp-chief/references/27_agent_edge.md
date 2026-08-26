# 27. 에이전트 가장자리

결정적 코어가 처리하지 못한 요청만 지능의 몫이다.

```
루프 ──needs-agent──▶ .claude/agents/rhwp-chief.md
                         ├─ 기존 스킬로 해결
                         ├─ result/response 갱신
                         └─ 반복 유형이면 표+핸들러 PR
```

에이전트 정의가 저장소에 있으면 이 스킬이 그 파일을 가리킨다.
없으면 같은 절차를 사람이 따른다. 코어는 에이전트 없이 표 안 요청을
끝낸다.

## 에이전트가 존중할 판정

- `done` — 다시 열지 않음
- `escalated` — 그 문서에 goal 강행 금지. FDE §4
- `invalid-input` — 원본 재확보. 변환 금지
- `failed` — 게이트가 거부한 것. 같은 인자로 재실행하지 않음
- `needs-agent` — 여기가 일터

## 금지 판단

코어 구현 변경, 한컴 최종 판정, 머지 판단은 maintainer 몫 (C14).
에이전트는 재현과 이슈화까지만 (FDE 에스컬레이션 계약).

## 도구

에이전트 frontmatter 는 Bash / Read / Grep / Glob.
루프 가동:

```
python3 tools/chief/service_loop.py --queue <큐> --bin target/release/rhwp --watch 10
```

needs-agent 수거:

```
# 각 요청 폴더의 result.json status
```

새 rhwp 하위명령을 제안하지 않는다. 표에 없는 일은 기존 스킬 명령으로.
