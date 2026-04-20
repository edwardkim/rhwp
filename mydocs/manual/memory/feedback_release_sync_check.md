---
name: 릴리즈 작업 전 main 동기화 점검 필수
description: devel→main 머지/태그/릴리즈 작업 시작 전 반드시 로컬 main 을 origin/main 과 동기화 점검. 분기 발견 시 즉시 보고
type: feedback
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
devel → main 머지, 태그 생성, 릴리즈 노트 작성 등 릴리즈 작업 시작 직전에 다음 절차를 반드시 수행:

```bash
git checkout main
git fetch origin main
git pull --ff-only origin main   # 또는 분기 발견 시 멈춤
```

**Why**: 2026-04-19 v0.7.3 릴리즈 작업 시 본 점검을 누락하고 PR 머지 → 태그 생성을 진행. 결과적으로 `git pull --ff-only` 가 fast-forward 불가로 실패했으나 그 시점에는 이미 `gh pr merge` 가 완료된 후였음. 우연히 origin 기준으로는 정상 (`c2e8a34` 위치에 태그 생성) 이었지만, 만약 로컬 main 의 분기된 커밋이 의미 있는 변경이었다면 잃을 뻔함. 절차 누락이 운으로 해결된 케이스.

**How to apply**:
- 릴리즈 관련 작업 시작 시 첫 단계로 main 동기화 점검 명시
- `git pull --ff-only` 실패 시 즉시 작업 중단 + 작업지시자에게 분기 원인 보고
- 분기된 로컬 커밋이 발견되면 (a) 폐기 (`reset --hard origin/main`), (b) cherry-pick 으로 devel 거쳐 재머지, (c) 다른 처리 — 작업지시자 결정 후 진행
- 분기 정리 전에는 PR 머지 / 태그 생성 / push 등 릴리즈 액션 금지
