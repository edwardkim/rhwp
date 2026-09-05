# 레시피 색인

예제 파일과 픽스처를 단계에 묶는다. 생성기 `_gen_pack.py` 가
`examples/README.md` 와 `fixtures/catalog.json` 을 같은 목록으로 맞춘다.

| 예제 | 단 | 픽스처 |
|------|----|--------|
| 01_issue_first.md | 1 이슈 | checklists/step_01_issue.json, envelopes/issue_created.json |
| 02_duplicate_open_pr.md | 1 예외 | envelopes/duplicate_open_pr.json |
| 03_analyze_canonical.md | 2 분석 | checklists/step_02_analyze.json |
| 04_branch_from_devel.md | 3 브랜치 | checklists/step_03_branch.json, transcripts/fetch_devel.json |
| 05_isolation_worktree.md | 3 워크트리 | envelopes/worktree_created.json |
| 06_never_steal_named_worktree.md | 3 예외 | layouts/forbidden-worktrees/registry.json |
| 07_implement_without_documentcore.md | 4 구현 | checklists/step_04_implement.json |
| 08_never_git_add_all.md | 4 스테이징 | envelopes/git_add_a_rejected.json |
| 09_fmt_all_check.md | 5 HARD GATE | envelopes/fmt_pass.json |
| 10_fmt_stale_check_rejected.md | 5 함정 | envelopes/fmt_stale_check_only.json |
| 11_clippy_deny_warnings.md | 5 clippy | envelopes/clippy_pass.json |
| 12_related_cargo_test.md | 5 test | envelopes/test_related_pass.json |
| 13_visual_evidence_render.md | 5 시각 | envelopes/visual_required.json |
| 14_work_receipt_capsule.md | 6 포인터 | envelopes/receipt_pointer.json |
| 15_working_doc.md | 7 문서 | checklists/step_07_working.json |
| 16_korean_pr_body_file.md | 8 PR | pr-bodies/closes_5322.md |
| 17_pr_first_checkbox_fmt.md | 8 템플릿 | checklists/step_08_pr.json |
| 18_sparse_missing_crates.md | 예외 | layouts/sparse-missing-crates/README.md |
| 19_windows_autocrlf_unix.md | 예외 | envelopes/fmt_fail_crlf.json |
| 20_ci_noci_vs_failure.md | 예외 | envelopes/ci_noci.json, envelopes/ci_failure.json |
| 21_closes_issue.md | 8 | pr-bodies/closes_5322.md |
| 22_no_new_cli.md | 4 | envelopes/new_cli_rejected.json |
| 23_named_file_stage.md | 4 | transcripts/stage_named.json |
| 24_full_procedure_walkthrough.md | 1–8 | checklists/step_*.json |

시나리오 전수는 `fixtures/scenario_catalog.json` 이다. 명령 발명은
`gh` / `git` / `cargo` / `rhwp replay|audit|lineage` / `python` / `node` 만
허용한다.
