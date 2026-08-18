# M04-f proptest 왕복 고도화 요약

이슈 #5465. 기존 `rhwp run` step 4종만 변형·스킵·예외로 펼친다.
DocumentCore 편집 로직은 발명하지 않는다.

## 수량

| 항목 | 건수 |
|---|---:|
| 픽스처 | 6 |
| skip 정직 표 | 295 |
| 유효 계획 | 5632 |
| 무효 계획 | 160 |
| 예외 | 375 |
| 변형 시퀀스 | 62 |
| 조건절 | 81 |

## 픽스처가 주장하는 apply

| action | 주장 픽스처 수 |
|---|---:|
| `fill_fields` | 0 |
| `replace_text` | 3 |
| `set_cell` | 2 |
| `set_checkbox` | 0 |

## skip 이유 분포

| reason | 행 |
|---|---:|
| `all_steps_skipped` | 5 |
| `cell_control_char` | 8 |
| `cell_missing` | 4 |
| `checkbox_missing` | 40 |
| `empty_find` | 5 |
| `field_missing` | 140 |
| `nested_table` | 5 |
| `no_hits` | 61 |
| `occurrence_oob` | 9 |
| `table_missing` | 14 |
| `unclaimed_capability` | 4 |

## 정직 규칙

- 누름틀 없는 픽스처에 fill_fields 를 적용하지 않는다.
- □ 없는 픽스처에 set_checkbox 를 적용하지 않는다.
- 표 없는 픽스처에 set_cell 을 적용하지 않는다.
- needle 없는 replace_text 는 no_hits skip.
- 빈 find 는 empty_find skip (문서 전체 치환 금지).
- 칸 값의 CR/LF/TAB 은 cell_control_char skip.
- 능력 미주장 픽스처(ref_mixed)는 unclaimed_capability.
- 적용 0 인 시퀀스는 왕복 성공으로 세지 않는다.
- insert_text / merge_cells 등 run step 4종 밖 action 은 스키마 거부.

## 이 좌석이 만지지 않는 것

- DocumentCore 새 mutation API
- canvaskit_policy
- pdf renderer
- page-count serializer
- layout-anomaly
- oracle_public
- render_backend
- gym
