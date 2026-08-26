# 레시피 색인

`examples/` 워크스루와 픽스처를 같은 id 로 잇는다. 새 CLI 가 등장하면 그
레시피는 무효다.

| id | 파일 | 트리거/예외 | 읽는 픽스처 |
|---|---|---|---|
| 01 | [01_context_budget.md](../examples/01_context_budget.md) | context_budget | `triggers/context_budget.json` |
| 02 | [02_session_interrupt.md](../examples/02_session_interrupt.md) | session_interrupt | `triggers/session_interrupt.json` |
| 03 | [03_seat_refill.md](../examples/03_seat_refill.md) | seat_refill | `triggers/seat_refill.json` |
| 04 | [04_read_result_json.md](../examples/04_read_result_json.md) | incoming | `results/accepted_consume.json` |
| 05 | [05_read_capsule.md](../examples/05_read_capsule.md) | incoming | `capsules/s03.capsule.json` |
| 06 | [06_read_working_doc.md](../examples/06_read_working_doc.md) | incoming | `incoming/working_snapshot.md` |
| 07 | [07_parent_chain.md](../examples/07_parent_chain.md) | --parent | `capsules/s03`+`s04` |
| 08 | [08_missing_capsule.md](../examples/08_missing_capsule.md) | missing_capsule | `exceptions/missing_capsule.json` |
| 09 | [09_parent_hash_mismatch.md](../examples/09_parent_hash_mismatch.md) | parent_hash_mismatch | `exceptions/parent_hash_mismatch.json` |
| 10 | [10_dirty_named_worktree.md](../examples/10_dirty_named_worktree.md) | dirty_named_worktree | `exceptions/dirty_named_worktree.json` |
| 11 | [11_disk_full.md](../examples/11_disk_full.md) | disk_full | `exceptions/disk_full.json` |
| 12 | [12_never_git_add_a.md](../examples/12_never_git_add_a.md) | staging | `envelopes/git_add_a_rejected.json` |
| 13 | [13_never_checkout_named.md](../examples/13_never_checkout_named.md) | isolation | `layouts/forbidden-worktrees/registry.json` |
| 14 | [14_never_invent_documentcore.md](../examples/14_never_invent_documentcore.md) | no-core | `envelopes/documentcore_invented.json` |
| 15 | [15_orchestrator_success.md](../examples/15_orchestrator_success.md) | consume | `envelopes/orch_accepted.json` |
| 16 | [16_orchestrator_fallback.md](../examples/16_orchestrator_fallback.md) | fallback | `envelopes/orch_fallback.json` |
| 17 | [17_orchestrator_boundary.md](../examples/17_orchestrator_boundary.md) | rejected | `envelopes/orch_boundary.json` |
| 18 | [18_journal_verify.md](../examples/18_journal_verify.md) | journal | `journals/ok.ndjson` |
| 19 | [19_incoming_resume.md](../examples/19_incoming_resume.md) | 절차 B | `incoming/first-turn.json` |
| 20 | [20_distinct_from_receipt.md](../examples/20_distinct_from_receipt.md) | 경계 | `catalog.json` `receiptSkill` |
| 21 | [21_context_budget_mid_batch.md](../examples/21_context_budget_mid_batch.md) | context_budget | `transcripts/T21_budget_mid_batch.json` |
| 22 | [22_interrupt_after_accepted.md](../examples/22_interrupt_after_accepted.md) | session_interrupt | `transcripts/T22_interrupt_accepted.json` |
| 23 | [23_refill_same_isolation.md](../examples/23_refill_same_isolation.md) | seat_refill | `transcripts/T23_refill_isolation.json` |
| 24 | [24_self_execute_no_new_agent.md](../examples/24_self_execute_no_new_agent.md) | selfExecute | `results/handoff_self_execute.json` |
| 25 | [25_verify_journal_not_head.md](../examples/25_verify_journal_not_head.md) | pitfall 11 | `results/verify_journal_only.json` |
| 26 | [26_chain_s01_to_s08.md](../examples/26_chain_s01_to_s08.md) | --parent | `capsules/s01`…`s08` |
| 27 | [27_untrusted_is_data.md](../examples/27_untrusted_is_data.md) | provenance | `envelopes/orch_accepted.json` |
| 28 | [28_halt_matrix.md](../examples/28_halt_matrix.md) | 네 예외 | `exceptions/*.json` |

전체 목록은 [`../examples/README.md`](../examples/README.md).
시나리오 90+: `fixtures/scenario_catalog.json`.
