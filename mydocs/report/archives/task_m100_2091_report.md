# 최종 결과보고서 — Task M100 #2091: 라운드 12 (layout_table_item, goal 루프 3/4)

- 이슈: #2091 / 브랜치: `local/task2091` / 2026-07-09 / goal 루프.

## 결과

| 지표 | 시작 (r11) | 완료 (r12) |
|---|---|---|
| `layout_table_item` 공식 CC | **121** (전체 1위) | **48** |
| 신규 `layout_table_control_block` | — | **75** (§5 심사: 표 anchor/TAC/float 단일 국면) |
| 전체 최대 CC | 121 | **120** (`typeset_section_with_variant` — R13 대상) |
| CC>25 예외 | 89 | 90 (+1/라운드, §5 과도기 허용 내) |
| 행동 회귀 | — | **0건** |

## 신규 패턴 — 조기 return 프로토콜

지배 블록에 함수 조기 return 이 있어 `TableControlOut::early_return: Option<(f64,bool)>`
로 신호를 반환하고 caller 가 `if let Some(ret) { return ret; }` 로 복원 — 통이동 가능
범위를 "함수-탈출 포함 블록"까지 확장한 첫 사례.

## 게이트·운영

게이트 전수 통과 (테스트 2,945/0 · issue_1116 · OVR 5샘플 회귀 0 · clippy 0).
운영 사고 1건: 게이트를 트리 안정화 전에 착수 → **중단·재실행** (결과 오염 방지 원칙).
