# 정지 규칙

이슈 #5342. 스킬 `rhwp-knowledge-map`. gym 아님. 새 CLI 없음.
이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.

| ID | 언제 | 행동 |
| --- | --- | --- |
| R01 | 요청이 지도 한 절·정본 하나로 닫힘 | llms.txt → 지도 해당 절 → 그 canonical 하나만 연다 |
| R02 | 지도 수치·도구 개수를 믿기 전에 재측정이 필요 | rhwp capabilities / capabilities --mcp / mcp-serve tools/list |
| R03 | 봉투 필드 이름·뜻이 필요 | 지도 §2 에서 이름을 찾는다. 없는 이름은 발명하지 않는다 |
| R04 | 지도 last_verified 가 30일보다 오래됨 | 날짜를 보여주고 중단. 기억으로 사다리를 메우지 않음 |
| R05 | 손에 든 바이너리 버전이 지도 §0 과 다름 | 바이너리가 이긴다. capabilities 로 다시 찍고 지도 숫자는 참고만 |
| R06 | 지도 요약과 canonical 상세가 다름 | canonical 을 따른다. 지도 행을 고쳐 쓰지 않고 상세로 점프 |
| R07 | §2 에 없는 필드 이름을 쓰려 함 | 중단. 철자 변형·암기 별칭을 만들지 않음 |
| R08 | 요청이 실무 작업(채움·표·스윕·배치·세션)으로 닫힘 | 지도에서 절·정본을 고른 뒤 이웃 스킬로 점프. 지도를 더 읽지 않음 |
| R09 | 요청이 대전 장 항해이거나 3층 계약·표면 추가 | rhwp-codex 또는 rhwp-agent-surface 로 인계. 그 스킬을 여기서 재작성하지 않음 |
| R10 | 지도를 처음부터 끝까지 읽으려 함 | 금지. 3문 진입으로 한 절만 고른다 |
| R11 | 지식지도 전용 rhwp 하위명령을 만들려 함 | 금지. 이 스킬은 문서 라우터다 |
| R12 | gym pack 으로 지식 지도를 재현하려 함 | 금지. 실 에이전트 문서 진입점이지 gym 이 아님 |
| R13 | 지도 기존 행을 더 자세히 풀어 쓰려 함 | 금지. 행 재서술 없이 canonical 로 보낸다 |
| R14 | §0·§2·§7 수치를 손으로 고치려 함 | 금지. 재측정 명령으로 실행해 갱신한다 |
| R15 | 필드 사전을 대전이나 표면 스킬에서 재정의하려 함 | 지도 §2 가 단일 출처. 대전의 필드 장은 앵커만 |
| R16 | 이웃 스킬 본문을 이 PR 에서 고치려 함 | 금지. 링크만 |

금지 기본값:

- `rhwp knowledge-map`, `rhwp knowledge_map`, `rhwp map`, `rhwp docs-index`, `rhwp agent-map`, `rhwp field-dict`, `rhwp remap`, `rhwp lookup-field`, `rhwp open-map`, `rhwp first-read`
- gym/ 트리 작성
- rhwp-codex / rhwp-agent-surface 본문 재작성
- DocumentCore 편집 로직
- 지도 행 재서술
