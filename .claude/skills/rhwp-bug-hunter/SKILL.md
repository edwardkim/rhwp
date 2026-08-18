---
name: rhwp-bug-hunter
description: 실사례 사용자 여정을 처음부터 끝까지 실행하고 한컴 공식 출력·법정 서식·실제 제출 요건의 정답지와 대조해 재현 가능한 rhwp 결함을 찾는다. Codex 본문·레퍼런스·픽스처는 .agents/skills/bug-hunter/ 가 정본이다. "버그 찾아줘(실사용 기준)", "정답지와 비교해", "playbook 여정 실행" 요청에 사용한다.
---

# rhwp-bug-hunter — Claude 진입 포인터

이 파일은 **얇은 포인터**다. 헌팅 본문·references/·examples/·fixtures/ 는
[`.agents/skills/bug-hunter/`](../../../.agents/skills/bug-hunter/SKILL.md) 에 있다.

권위 방법론은 [버그 헌팅 playbook](../../../mydocs/manual/bug_hunting_playbook.md) 한 장이다.
두 번째 루브릭을 여기서 만들지 않는다. gym 이 아니고, 새 CLI 를 발명하지 않으며,
DocumentCore 를 고치지 않는다.

실문서 여정의 첫 관측은 `rhwp info --json <문서>`로 남긴다. 그 다음 단계와 정답지
대조 기준은 아래 정본 playbook을 따른다.

에이전트 정의: [`.claude/agents/bug-hunter.md`](../../agents/bug-hunter.md).

바로 열 장:

- [실행 계약](../../../.agents/skills/bug-hunter/SKILL.md)
- [판단 트리](../../../.agents/skills/bug-hunter/references/00_tree.md)
- [playbook 권위](../../../.agents/skills/bug-hunter/references/01_playbook_authority.md)
- [정답지 먼저](../../../.agents/skills/bug-hunter/references/04_ground_truth.md)
- [문자 멀티셋 분류](../../../.agents/skills/bug-hunter/references/09_text_multiset.md)
- [fidelity_compare](../../../.agents/skills/bug-hunter/references/12_fidelity_compare.md)
- [이슈 템플릿](../../../.agents/skills/bug-hunter/references/13_issue_template.md)
- [여정 카탈로그](../../../.agents/skills/bug-hunter/references/17_journeys.md)
