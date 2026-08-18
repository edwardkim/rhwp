# 7단 — 처리 결과 문서 (`mydocs/working/`)

규모 있는 변경은 `mydocs/working/` 에 무엇을·왜·어떻게·검증 실측을 남긴다.
이 스킬 고도화의 기록은 `mydocs/working/agent_contributor.md` 다.

## 최소 칸

```markdown
---
kind: working
status: active
issue: 5322
---

# 제목 (#이슈)

작업 브랜치: `feat/...`
대상: 경로
```

- 한 줄 요약
- 이슈가 요구한 것 / 하지 말라는 것
- 만진 경로 / 만지지 않은 경로
- 시험 명령
- fmt 게이트 명령
- PR 메모 (`closes #`, `--body-file`, base `devel`)

## 관례

- 파일 이름은 주제를 드러낸다. `agent_contributor.md` 처럼.
- 스테이지를 여러 개로 쪼개지 않아도 된다. 한 파일이 한 파동을 담는다.
- `mydocs/pr/` 메인터너 기록은 기여자가 만들지 않는다 (`CONTRIBUTING.md`).

예제: [15_working_doc.md](../examples/15_working_doc.md).
