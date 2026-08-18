# 22 — confidence

값은 `high` / `medium` / `low` 세 토큰이다. 이 스킬이 새 토큰을 만들지
않는다. 현재 `build_menu` 는 low 를 쓰지 않는다.

| affordance | high | medium |
| --- | --- | --- |
| security-sweep | 주입 ≥ 1 | 은닉만 |
| form-fill | 항상 | — |
| table-extract | 항상 | — |
| structure-outline | 노드 ≥ 3 | 노드 1–2 |
| chart-extract | 항상 | — |
| note-structure | 항상 | — |
| long-doc-digest | 쪽 ≥ 20 | 쪽 10–19 |
| triage-overview | 항상 | — |

confidence 로 줄을 다시 세우지 않는다. 은닉(medium) 이 표(high) 보다
위인 것은 우선순위 숫자 때문이다.
