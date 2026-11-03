# 선정·원 head·충돌과 기여 보존

접수 API 확인: `2026-11-03T19:35:06.825008+09:00`. 고정 devel은 `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`다.

열린 38건 중 기존 reviewer/review가 있는 5건을 제외했다. Draft는 없었고 미배정 33건에서 생성 시각 오름차순 10건을 선정했다. #8203은 postmelee APPROVED, #8226·#8285·#8286·#8289는 postmelee 요청이 있어 제외했다. 선정한 열 건에만 edwardkim reviewer를 요청하고 각각 API로 재확인했다.

| 원 PR | 원 head | 보존한 적용 merge | 단독 base 충돌 | 원 head CI (새 후보 CI와 구분) |
| --- | --- | --- | --- | --- |
| [#8211](https://github.com/edwardkim/rhwp/pull/8211) | `82b959af244dfdbfeaffb4186910c371f2afdc29` | `53128ad702` | 5파일 | [success 31, skipped 4](https://github.com/edwardkim/rhwp/actions/runs/40151919183/job/121050273714) |
| [#8235](https://github.com/edwardkim/rhwp/pull/8235) | `96c066b2dd267e76e282d1dccfa3b5e66efeaa20` | `0919a2a8bd` | 20파일 | [success 34, skipped 5](https://github.com/edwardkim/rhwp/actions/runs/40404473537/job/121878200389) |
| [#8281](https://github.com/edwardkim/rhwp/pull/8281) | `fa6c4b7203408b36d405ba165348731ecb44001d` | `28dad1c94b` | 1파일 | [success 32, neutral 1, skipped 6](https://github.com/edwardkim/rhwp/actions/runs/40414382733/job/121908094788) |
| [#8287](https://github.com/edwardkim/rhwp/pull/8287) | `92d19ea0e3690446106a126b747e0e5f611a0837` | `5b2aa67c02` | 0파일 | [success 34, skipped 5](https://github.com/edwardkim/rhwp/actions/runs/40341766517/job/121661471327) |
| [#8290](https://github.com/edwardkim/rhwp/pull/8290) | `7e5b3f40d2f83ebba6c01b02be4327c564a28902` | `bb780b8391` | 0파일 | [success 28, neutral 1, skipped 6](https://github.com/edwardkim/rhwp/actions/runs/40334117416/job/121632216794) |
| [#8292](https://github.com/edwardkim/rhwp/pull/8292) | `1f26297e9cae86955dc36f2f100585cae4b06784` | `8edd7f0a5f` | 1파일 | [success 32, neutral 1, skipped 6](https://github.com/edwardkim/rhwp/actions/runs/40412731330/job/121902984487) |
| [#8293](https://github.com/edwardkim/rhwp/pull/8293) | `4447d39d2fa09155b1162203f4d1ed9f4e6fd1e6` | `71692728f8` | 0파일 | [success 33, skipped 4](https://github.com/edwardkim/rhwp/actions/runs/40341526011/job/121663628941) |
| [#8298](https://github.com/edwardkim/rhwp/pull/8298) | `b46dde6798da1fde750bdf43a4a3078af1c139b9` | `07a307c236` | 1파일 | [success 31, skipped 4](https://github.com/edwardkim/rhwp/actions/runs/40413434176/job/121906227704) |
| [#8299](https://github.com/edwardkim/rhwp/pull/8299) | `33b6a58200152b06f22b51d6cdfad223bb5a8d25` | `02b7ad04bc` | 0파일 | [success 32, neutral 1, skipped 6](https://github.com/edwardkim/rhwp/actions/runs/40518742319/job/122278211416) |
| [#8308](https://github.com/edwardkim/rhwp/pull/8308) | `b3cd705556a997bd1126925d804048e30115aad5` | `97f2d5c8ec` | 0파일 | [success 32, skipped 5](https://github.com/edwardkim/rhwp/actions/runs/40471805330/job/122093002227) |

원 head CI의 skip·neutral은 실행 성공으로 세지 않았다. 누적 후보의 보정 뒤에는 별도 로컬 검증과 새 head GitHub CI가 필요하다.

## 충돌의 실제 경로

- #8211: `src/renderer/layout/table_layout.rs`, `src/renderer/typeset.rs`, `src/renderer/typeset/controls/order.rs`, `src/renderer/typeset/section/vpos.rs`, `src/renderer/typeset/table/block/entry.rs`
- #8235: `src/document_core/commands/clipboard.rs`, `src/document_core/commands/html_import.rs`, `src/document_core/commands/text_editing.rs`, `src/document_core/queries/cursor_rect.rs`, `src/paint/builder.rs`, `src/paint/json.rs`, `src/paint/schema.rs`, `src/renderer/composer/line_breaking.rs`, `src/renderer/composer/stored_cell_reflow.inc.rs`, `src/renderer/composer/stored_lines_runs.inc.rs`, `src/renderer/layout/table_partial.rs`, `src/renderer/render_tree.rs`, `src/wasm_api.rs`, `tests/cases/form_object_caret_slot.rs`, `tests/cases/issue_6520_svg_layer_contract.rs`, `tests/cases/issue_7444_cut_boundary_object.rs`, `tests/cases/reflow_body_paragraph_range.rs`, `tests/cases/styled_text_clipboard_field_affinity.rs`, `tests/cases/styled_text_html_field_affinity.rs`, `tests/common/passthrough_exemptions.rs`
- #8281: `src/renderer/layout/paragraph_layout.rs`
- #8292: `src/model/table_row_grid.rs`
- #8298: `src/document_core/commands/document.rs`

단독 충돌은 5건·28개 고유 파일이다. 현재 논리 편집 API·Storage 예산·schema 35·#8517 동작을 보존하는 메인터너 보정 방침을 승인받았다. 개별 파일에 ours/theirs를 일괄 적용하지 않았다.

## 범위와 원격 권한

로컬 branch는 `review/integration-20261103-semanticist21`이다. 선정한 열 head는 누적 merge history의 조상이며 제외한 다섯 head는 포함되지 않는다. 원 PR의 과거 devel merge를 새 기능으로 세지 않는다. 원 head·credit을 보존하기 위해 최종 통합은 squash가 아닌 merge commit을 제안한다.

원격에서 실제 수행한 작업은 승인된 reviewer 요청뿐이다. source branch push·공개 review/comment·PR 생성·merge·CI 재실행은 수행하지 않았다. 보정된 누적 후보의 원격 처리는 별도 승인이 필요하다.

## 제출 준비 종료 시 재확인

`2026-11-04T00:29:15.525324+09:00`에 원격 devel `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`, 원 10건의 head 불변·OPEN, 본인 계정의 열린 PR 0건을 확인했다. 원격에 새 코드를 올리지는 않았다.
