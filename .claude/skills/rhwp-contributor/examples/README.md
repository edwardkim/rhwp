# rhwp-contributor 워크스루

실기여 에이전트가 8단을 닫는 레시피. gym 아님. 새 CLI 없음.

정본 게이트: `cargo fmt --all -- --check`.
낡은 표기 `cargo fmt --check` 는 거부.

| 파일 | 한 줄 |
|------|------|
| [01_issue_first.md](01_issue_first.md) | 이슈 선등록 |
| [02_duplicate_open_pr.md](02_duplicate_open_pr.md) | 열린 PR 중복 |
| [03_analyze_canonical.md](03_analyze_canonical.md) | 정본을 읽고 이슈에 기록 |
| [04_branch_from_devel.md](04_branch_from_devel.md) | upstream/devel 에서 분기 |
| [05_isolation_worktree.md](05_isolation_worktree.md) | 격리 워크트리 |
| [06_never_steal_named_worktree.md](06_never_steal_named_worktree.md) | named worktree 를 훔치지 않는다 |
| [07_implement_without_documentcore.md](07_implement_without_documentcore.md) | DocumentCore 편집 로직을 발명하지 않는다 |
| [08_never_git_add_all.md](08_never_git_add_all.md) | git add -A 거부 |
| [09_fmt_all_check.md](09_fmt_all_check.md) | HARD GATE cargo fmt --all -- --check |
| [10_fmt_stale_check_rejected.md](10_fmt_stale_check_rejected.md) | 낡은 cargo fmt --check 는 게이트가 아니다 |
| [11_clippy_deny_warnings.md](11_clippy_deny_warnings.md) | clippy -D warnings |
| [12_related_cargo_test.md](12_related_cargo_test.md) | 관련 cargo test |
| [13_visual_evidence_render.md](13_visual_evidence_render.md) | 렌더/레이아웃 시각 근거 |
| [14_work_receipt_capsule.md](14_work_receipt_capsule.md) | 영수증은 포인터만 |
| [15_working_doc.md](15_working_doc.md) | 처리 결과 문서 |
| [16_korean_pr_body_file.md](16_korean_pr_body_file.md) | 한국어 PR, --body-file |
| [17_pr_first_checkbox_fmt.md](17_pr_first_checkbox_fmt.md) | PR 템플릿 첫 체크박스 = fmt 게이트 |
| [18_sparse_missing_crates.md](18_sparse_missing_crates.md) | 스파스 체크아웃에 crates/ 없음 |
| [19_windows_autocrlf_unix.md](19_windows_autocrlf_unix.md) | Windows autocrlf vs rustfmt Unix |
| [20_ci_noci_vs_failure.md](20_ci_noci_vs_failure.md) | CI noci 와 FAILURE 를 섞지 않는다 |
| [21_closes_issue.md](21_closes_issue.md) | closes #이슈 |
| [22_no_new_cli.md](22_no_new_cli.md) | 새 rhwp CLI 명령을 만들지 않는다 |
| [23_named_file_stage.md](23_named_file_stage.md) | 이름을 댄 파일만 stage |
| [24_full_procedure_walkthrough.md](24_full_procedure_walkthrough.md) | 8단 전 구간 워크스루 |

생성기: [`../references/_gen_pack.py`](../references/_gen_pack.py).
색인: [`../references/recipe-index.md`](../references/recipe-index.md).
