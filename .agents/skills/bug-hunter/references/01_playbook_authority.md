# 01 — playbook 이 유일한 권위

정본: [`mydocs/manual/bug_hunting_playbook.md`](../../../../mydocs/manual/bug_hunting_playbook.md).

이 스킬·레퍼런스·픽스처·시험은 playbook 을 **실행**한다. 독자 점수표,
독자 심각도, 독자 "헌팅 등급"을 만들지 않는다. 문장이 어긋나면
playbook 을 고치는 별도 문서 PR 이지, 여기 장에서 덮어쓰지 않는다.

## 왜 복제하지 않는가

playbook 은 이미 실패에서 나온 규칙이다.

- 오라클 통과 ≠ 무손실 (#3551)
- 상신 전 devel 확인 (53건 중 17건이 이미 고쳐짐)
- 표본 1건 일반화 금지 (#3368)
- 가설은 구현해서 기각 (#3518)

같은 규칙을 스킬 안에 다른 말로 다시 쓰면 두 번째 루브릭이 생긴다.
에이전트가 어느 쪽을 따를지 고르게 된다. 그것은 버그다.

## 이 스킬이 더하는 것 (방법론이 아님)

- 요청 → 기존 CLI 매핑
- 정지 규칙 ID (F01–F16) — playbook 문장의 가리킴
- 여정·분류·이슈 템플릿의 **기계 가독 픽스처**
- Claude 얇은 포인터 `.claude/skills/rhwp-bug-hunter/`

가리킴이 playbook 문장과 다르면 픽스처가 틀린 것이다.

## 예시 4단 (playbook 구조 — 여기 변형 금지)

```
### 예시 N — <여정 이름>
- 문제 정의
- 예시(실물)
- 흐름
- 방식(무엇을 찾나)
→ 발견
```

새 여정을 밟으면 playbook 에 이 4단으로 추가할 것을 **제안**한다.
이 스킬 폴더에 두 번째 예시집을 키우지 않는다. `examples/` 는
playbook 여정을 끝까지 도는 **레시피**이지 권위가 아니다.

## 금지 문장

- "우리 스킬의 헌팅 점수"
- "playbook 대신 이 체크리스트"
- "gym pack 으로 같은 여정을 채점"
- "새 헌팅 전용 CLI 가 이 루브릭을 구현한다"

## 관련

- 함정 장: [02_judgment_traps.md](02_judgment_traps.md)
- 여정 카탈로그: [17_journeys.md](17_journeys.md)
- 처리 결과: [`mydocs/working/agent_bug_hunter.md`](../../../../mydocs/working/agent_bug_hunter.md)
