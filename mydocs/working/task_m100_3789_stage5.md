# Stage 5 완료 보고 — Task M100 #3789: 최신 `devel` 재기준화와 focused 검증

- **일자**: 2026-08-27 KST
- **브랜치**: `task_m100_3789-render-boundary`
- **이전 기준**: `upstream/devel@1b91c2025`
- **현재 기준**: `upstream/devel@2166f4065`
- **merge commit**: `39d6aa1dd`
- **이슈**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **문서 성격**: Stage 5 종료 시점에 작성한 contemporaneous 보고

## 재기준화

절차 감사 보정 commit `212fa79a4`의 결과를 작업지시자에게 공유하고 다음 Stage 승인을 받았다. fetch 뒤
branch는 최신 `devel`보다 63커밋 뒤였고, 기존 #3789 commit SHA는 계획·보고의 감사 증거로 참조되고
있었다. 따라서 rebase로 이력을 바꾸지 않고 current-base merge를 선택했다.

`git merge-tree --write-tree HEAD upstream/devel`이 충돌 없는 tree를 만들었고 실제 merge도 ort 자동
병합으로 완료됐다. 양쪽 변경이 겹친 CI policy와 Node test에는 다음 계약이 함께 남았다.

- #3789: caption render source는 Render Diff positive, `src/main.rs`와 structure query는 negative
- #6205: duration-policy resolve·refresh job은 CI audit allowlist에 포함

## focused 검증

| 검증 | 결과 |
| --- | ---: |
| `issue_cli_test_caption_no_panic` | 1/1 통과 |
| `cli_json_contract` | 31/31 통과 |
| `mcp_session_structure_extract_contract` | 6/6 통과 |
| `provenance_contract` | 10/10 통과 |
| `batch_axes_contract` | 17/17 통과 |
| `diagnostics_flag_contract` | 15/15 통과 |
| `cli_exit_codes` | 13/13 통과 |
| `cli_catalog_contract` | 20/20 통과 |
| classifier·policy Node | 67/67 통과 |
| CI workflow Python | 70/70 통과 |

추가로 Render Diff actionlint, Cargo format, integration suite manifest, source unit tier, Markdown 링크 603개와
branch diff 검사가 통과했다. generated integration suite는 ignored 상태이며 제출 대상에 포함하지 않는다.

## 종료 판단과 다음 승인 게이트

최신 기준 merge는 충돌 없이 완료됐고 #3789와 새 upstream CI 계약이 함께 통과한다. Stage 5는 완료로
판정한다. 최신 기준의 전체 release-test와 clippy는 아직 실행하지 않았으며, 이 보고를 작업지시자가
검토·승인한 뒤 Stage 6에서 수행한다. remote push와 PR 생성은 그 뒤에도 별도 승인으로 남긴다.
