# bug-hunter references

이 폴더는 [버그 헌팅 playbook](../../../../mydocs/manual/bug_hunting_playbook.md)을
에이전트가 실행하기 위한 장이다. **두 번째 루브릭이 아니다.** 판정 기준이
playbook 과 어긋나면 playbook 이 이긴다.

이슈 #5324. 실 에이전트 경로. gym 아님. 새 CLI 없음. 헌팅이지 버그픽스가 아님.

| 장 | 내용 |
| --- | --- |
| 00_tree.md | 6단 판단 트리 |
| 01_playbook_authority.md | playbook 이 유일한 권위 |
| 02_judgment_traps.md | 판정 함정 4종 |
| 03_journey_selection.md | 실물 정부·법정 서식 선택 |
| 04_ground_truth.md | 정답지를 여정 실행보다 먼저 |
| 05_hangul_pdf_provenance.md | 도구·버전·경로·폰트 |
| 06_self_consistency_limit.md | 기준 없으면 render-diff 만 |
| 07_run_to_final.md | 최종 산출물까지 |
| 08_pixel_visual.md | 픽셀/시각 후보 |
| 09_text_multiset.md | 소실·과잉·치환 |
| 10_reread_values.md | 기록값 재독 |
| 11_exit_json_contract.md | 종료 코드·JSON |
| 12_fidelity_compare.md | tools/fidelity_compare |
| 13_issue_template.md | 재현·파일:라인·정답지 |
| 14_no_filing.md | 접수·로그인·실명인증 금지 |
| 15_utf8_console.md | 콘솔 착시는 결함 아님 |
| 16_pitfalls.md | 헌팅 함정 |
| 17_journeys.md | 여정 카탈로그 |
| 18_worked_traces.md | 재현 트레이스 |
| 19_intent_matrix.md | 발화 → 명령 |
| 20_classification.md | 비교 분류표 |
| 21_handoff.md | 이웃 스킬 |
| 22_failure_signals.md | 정지 규칙 |
| 23_gate_recipes.md | 게이트 |
| 24_existing_cli.md | 기존 CLI 화이트리스트 |

픽스처는 `../fixtures/`. 예제는 `../examples/`.
생성기: `_gen_pack.py` (픽스처만. 본 장은 수기).
