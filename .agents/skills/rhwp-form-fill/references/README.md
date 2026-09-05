# rhwp-form-fill references

이 폴더는 실 에이전트가 서식 누름틀을 채우고 메일머지할 때 연다.
gym 과제가 아니고, 새 edit 로직도 없다.

| 파일 | 내용 |
| --- | --- |
| 00_tree.md | 판단 트리 |
| 01_fields_survey.md | `fields --json` 조사 |
| 02_fill_fields.md | 단건 `edit fill-fields` |
| 03_repeat_occurrence.md | `이름[순번]` |
| 04_batch_fill.md | `batch fill` 메일머지 |
| 05_dry_run_verify.md | `--dry-run` / `--verify` |
| 06_sanitize.md | 제출 메타 정리 |
| 07_envelopes.md | `--json` 봉투 |
| 08_pitfalls.md | 실측 함정 |
| 09_journeys.md | 실사용 여정 |
| 10_handoff.md | 이웃 스킬 인계 |
| 11_failure_signals.md | 신호 → 처방 |
| 12_data_formats.md | JSON / @파일 / CSV / JSONL |
| 13_name_field.md | 산출 파일명 |
| 14_insert_image.md | 직인·서명 |
| 15_axis_choice.md | fill-fields vs set-cell |
| 16_worked_traces.md | 재현 트레이스 |
| 17_intent_matrix.md | 발화 → 명령 |
| 18_field_catalog.md | 표본 필드 |
| 19_gate_recipes.md | jq 게이트 |
| 20_exit_codes.md | 종료 코드 #2707 |

픽스처는 `fixtures/`. 생성기는 `_gen_pack.py`.
테스트는 `scripts/tests/test_agent_form_fill.py` 와
`tests/cases/agent_form_fill_skill_contract.rs`.
