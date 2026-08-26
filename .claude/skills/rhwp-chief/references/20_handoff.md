# 20. 인계 — 이웃 층을 재작성하지 않는다

Chief 는 접수 창구다. 창구가 이웃 가게의 메뉴판을 고쳐 쓰지 않는다.

| 언제 | 어디로 | 이 PR |
| --- | --- | --- |
| 증상 하나, 패닉 시그니처 | rhwp-fde / fde playbook | 게이트 호출만 |
| 목표+코퍼스+근거 대장 | rhwp-strategist | 손대지 않음 |
| 표 칸을 고치고 되돌리기 | rhwp-table-exchange | 손대지 않음 |
| 누름틀 순번·메일머지 세부 | rhwp-form-fill | fill 핸들러가 기존 CLI 만 |
| 폴더 수백 건 | rhwp-bulk-pipeline | 손대지 않음 |
| 실사례 여정 대조 | bug-hunter | escalate 이후 |
| 출처 표지·주입 | rhwp-provenance | 원칙만 인용 |
| 미지 문서 읽기 | rhwp-doc-triage | diagnose 이상이면 여기 |

`forbiddenSkillsTouch` 목록의 SKILL.md 는 이 작업에서 열어서 고치지 않는다.
존재 확인만 한다 (계약 시험).

에이전트 정의 `.claude/agents/rhwp-chief.md` 는 이 스킬의 진입점이므로
스킬 링크 한 줄을 더할 수 있다. FDE/Strategist 에이전트 본문은 고치지 않는다.
