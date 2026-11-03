---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# semanticist21 오래된 미배정 PR 10건 — 누적 검토 계획

## 현재 단계

접수·최소 사전 분류·reviewer 지정·source/CI 조회·실제 merge-tree 시뮬레이션을 완료했다.
다섯 PR의 충돌 보정 방침은 아래에 구체화했다. 이 계획은 현재 base의 실행·시각 검증 완료를 뜻하지 않는다.
원 head의 기존 녹색 CI와 앞으로 검증할 누적 후보를 구분하며 원 PR별 판정을 유지한다.

## 대상과 기준

- 작성자 열린 PR 38건에서 요청 reviewer 또는 제출 review가 있는 5건을 제외했다. Draft는 없었다.
- 선정 시 미배정 33건 중 생성 시각 오름차순 10건이다. 요청 user/team reviewer와 기존 제출 review를 모두 대조했다.
- #8203은 postmelee의 APPROVED review가 있어 제외했다. #8226·#8285·#8286·#8289는 postmelee 요청이 있어 제외했다.
- 10건 모두 edwardkim reviewer 요청을 먼저 지정하고 API로 재확인한 뒤 상세 조사했다.
- 최신 기준: `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`. 직전 devel 이후 #8517의 renderer 통합 및 #8523/#8524 운영 문서를 반영했다.
- 로컬 branch: `review/integration-20261103-semanticist21`, 기본 작업공간 `/home/edward/mygithub/rhwp`.
- 대표 단일 이슈가 없는 열 건이라 날짜 형식의 누적 검토 branch를 사용한다. 다른 작업 worktree/WIP는 보존한다.

| 순서 | 원 PR | 생성 시각 KST | 원 head | 현재 devel merge-tree |
| ---: | --- | --- | --- | --- |
| 1 | [#8211](https://github.com/edwardkim/rhwp/pull/8211) 표 삽입·원본 본문 흐름 | 10-25 10:46 | `82b959af24` | 충돌 5파일 |
| 2 | [#8235](https://github.com/edwardkim/rhwp/pull/8235) 양식의 논리 편집·선택 | 10-26 02:21 | `96c066b2dd` | 충돌 20파일 |
| 3 | [#8281](https://github.com/edwardkim/rhwp/pull/8281) 저장 줄 없는 개체 뒤 캐럿 | 10-26 19:36 | `fa6c4b7203` | 충돌 1파일 |
| 4 | [#8287](https://github.com/edwardkim/rhwp/pull/8287) 다각형 회전·대칭 왕복 | 10-26 21:42 | `92d19ea0e3` | clean |
| 5 | [#8290](https://github.com/edwardkim/rhwp/pull/8290) 문단 간격 편집·되돌리기 | 10-26 22:50 | `7e5b3f40d2` | clean |
| 6 | [#8292](https://github.com/edwardkim/rhwp/pull/8292) 중복 표 ID의 HWPX 저장 | 10-26 23:08 | `1f26297e9c` | 충돌 1파일 |
| 7 | [#8293](https://github.com/edwardkim/rhwp/pull/8293) 머리말·꼬리말 넘침 | 10-26 23:38 | `4447d39d2f` | clean |
| 8 | [#8298](https://github.com/edwardkim/rhwp/pull/8298) 넘친 표의 저장 높이 | 10-27 07:34 | `b46dde6798` | 충돌 1파일 |
| 9 | [#8299](https://github.com/edwardkim/rhwp/pull/8299) 저장 줄 없는 문단의 공간 | 10-27 07:38 | `33b6a58200` | clean |
| 10 | [#8308](https://github.com/edwardkim/rhwp/pull/8308) 중첩·분할 셀의 hit-test clip | 10-27 15:05 | `b3cd705556` | clean |

## 선행 해결과 중복 범위

- #8093 → #8319, merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`: Form의 기본 논리 한 칸과 공개 재현 계약. [검토](archives/pr_8093_review.md).
- #8187, merge `3724ab9dc43bd60796a20e48c00c5f5ad1f053b9`: 공통 논리 편집·필드 원자성·NBSP·범위 재조판·분할 셀 원본 문단 범위. [검토](archives/pr_8187_review.md).
- devel 조상 `3a71190ec382ee94c991ae4a25f0794e2727f6e4`: inline flow의 문자 논리 주소. #8281 전체가 해결됐다고 확대하지 않고 끝 런·끝 높이를 따로 실행한다.
- #8174/#8183: Storage 기반 자원 예산·취소/실패 전달과 PDF 정책. #8292/#8298 및 Form composer에 무한도 clone/project 경로를 복원하지 않는다.
- #8517, merge `c97709b59742372a817b3886f442144728fca646`: 최근 줄 구성·저장 줄·표 경계·그림 흐름 보정. [통합 및 원 PR 11건의 검토](archives/pr_8517_review.md)를 보존 대상에 포함한다.

## 충돌 보정 방침 — 승인할 로컬 범위

| 원 PR | 실제 충돌 | 통합 방침 |
| --- | --- | --- |
| [#8211](https://github.com/edwardkim/rhwp/pull/8211) | `src/renderer/layout/table_layout.rs`, `src/renderer/typeset.rs`, `src/renderer/typeset/controls/order.rs`, `src/renderer/typeset/section/vpos.rs`, `src/renderer/typeset/table/block/entry.rs` | 측정 주소 이동을 공통 삽입 경로에 연결하고 현재 TAC 제어 전용 꼬리·숨긴 첫 줄·쪽 원점·분할 host handoff를 보존한다. 마지막 저장 표 줄의 간격은 같은 줄 소유 결과에서 한 번만 계상한다. |
| [#8235](https://github.com/edwardkim/rhwp/pull/8235) | `src/document_core/commands/clipboard.rs`, `src/document_core/commands/html_import.rs`, `src/document_core/commands/text_editing.rs`, `src/document_core/queries/cursor_rect.rs`, `src/paint/builder.rs`, `src/paint/json.rs`, `src/paint/schema.rs`, `src/renderer/composer/line_breaking.rs`, `src/renderer/composer/stored_cell_reflow.inc.rs`, `src/renderer/composer/stored_lines_runs.inc.rs`, `src/renderer/layout/table_partial.rs`, `src/renderer/render_tree.rs`, `src/wasm_api.rs`, `tests/cases/form_object_caret_slot.rs`, `tests/cases/issue_6520_svg_layer_contract.rs`, `tests/cases/issue_7444_cut_boundary_object.rs`, `tests/cases/reflow_body_paragraph_range.rs`, `tests/cases/styled_text_clipboard_field_affinity.rs`, `tests/cases/styled_text_html_field_affinity.rs`, `tests/common/passthrough_exemptions.rs` | 기존 #8187 API와 NBSP·필드 경계·공통 삭제를 유지하고 Form의 정확한 형제·물리 행 소유를 연결한다. schema 35·Print 기하 계약을 되돌리지 않는다. composer와 빈 중첩 HTML 표는 현행 Storage 전달·취소·자원 제한 경로에 연결한다. |
| [#8281](https://github.com/edwardkim/rhwp/pull/8281) | `src/renderer/layout/paragraph_layout.rs` | text_logical_starts를 유지하고 남은 문단 끝 런·높이만 공통 inline flow 계획과 실제 배치에서 함께 소비한다. 기존 Storage push 경로와 표로 끝나는 정상 대조군을 보존한다. |
| [#8292](https://github.com/edwardkim/rhwp/pull/8292) | `src/model/table_row_grid.rs` | ID 충돌 처리를 project_with_storage에 연결하고 tree_entry·clone_table·grid rebuild·fingerprint·record push의 예산과 오류 전달을 유지한다. 뒤에 나오는 기존 ID와 재할당 값 충돌도 검증한다. |
| [#8298](https://github.com/edwardkim/rhwp/pull/8298) | `src/document_core/commands/document.rs` | 성장 높이를 기존 측정 결과와 Storage 기반 snapshot 경로에 연결하고 HWPX lineage·되감김 프레임·cellSz를 보존한다. 손대지 않은 표·중첩 표의 적용 범위와 HU 반올림 근거도 검증한다. |

28개 충돌 파일에 ours/theirs를 일괄 선택하지 않는다. 이미 반영된 API·메타데이터는 현재 구현을
소비하게 하고 남은 기여 동작을 연결한다. 누적 적용 중 같은 계약의 충돌도 이 방침으로 처리한다.
기능 의미나 검증 범위를 바꿔야 하는 경우에만 추가 판단을 요청한다.

## PR별 실행 범위

| 원 PR | 핵심 검증 |
| --- | --- |
| #8211 | table_insert_measurement_ownership, table_insert_body_flow; 공개 53개 호출의 live·HWPX 재열기; 원본 뒤 본문·중첩 표·CellBreak·Center·None/Square 대조 |
| #8235 | form_inline_editing, form_object_caret_slot; 본문/셀·같은 위치 형제·양쪽 경계·범위 태그·거부 원자성·HWP/HWPX 왕복; 기존 #8093/#7444/#8187 대조와 실제 WASM 선택 API |
| #8281 | no_lineseg_inline_object_caret_anchor; 그림 앞뒤·그림만 있는 문단·끝 글자 모양·HWPX 저장 재열기·표 끝 대조 |
| #8287 | polygon_transform_roundtrip; 한컴 rotated HWP/HWPX·90도/대칭/크기 변경·역연산·저장 재열기 |
| #8290 | paragraph_spacing_moves_stored_lines; 앞/뒤 간격·이웃 문단·undo/redo·저장 재열기·줄 리셋 |
| #8292 | hwpx_table_grid_duplicate_ids; 빈 논리 행·0/같은 ID·중첩·비중복 ID 보존·최소/정상 자원 예산·기존 #7998 저장 대조 |
| #8293 | header_overflow_paragraphs_render; 한컴 header-overflow HWP/HWPX·넘침/정상·본문 첫 줄·캐럿/hit-test |
| #8298 | hancom_table_grow_saves_table_height; grow3 전후·본문/중첩 표·무편집·HWP/HWPX 왕복·저장 취소/예산 실패 원자성 |
| #8299 | composer_without_stored_lines 및 private vpos 회귀; 공개 문단 서식 API·연속 무저장 문단·리셋/대조·source-unit 정책 |
| #8308 | nested_wide_table_clip, split_row_clip_hit; 한컴 nested-wide·rowbreak 16쪽·Tab/클릭·완전/부분 clip·실제 WASM hit-test |

## 실행 순서와 완료 조건

1. 동일 branch에서 오래된 순서로 원 head/기능 commit과 적용 순서를 기록하고 누적 검토한다. 원 기여 commit과 credit을 보존한다. 원 PR의 devel merge를 신규 기능으로 세지 않는다.
2. 이미 해결된 기반 동작과 남은 기능을 작은 실제 API·원본 입력으로 먼저 분리한다. 수정 전 FAIL/수정 후 PASS·정상 대조를 확보한다. 소스 관측을 실행 결과로 보고하지 않는다.
3. 표·양식·문단·저장의 변경값을 생산 → 측정 → 실제 배치/커서/저장 소비까지 추적한다. #8211의 흐름·TAC와 #8298의 저장 높이는 내용 컷·예약 높이·다음 내용·종료까지 대조한다.
4. 기존 원본/독립 한컴 Print를 우선 재사용한다. #8211의 notice live/reopened, #8235 form-01, rotated, header-overflow, grow3, nested-wide 등 입력별 대응을 확인한다. 기준이 부족하면 해당 입력·편집 상태의 독립 출력을 확보한다.
5. 조판 영향 범위는 기준 devel/누적 후보 각각 Native/fresh optimized WASM 전쪽 TSV·직접 확대 판독을 완료한다. 한 쪽이라도 raw 90% 미만·누락·쪽수 차이 또는 base 대비 2%p 초과 하락이면 원인을 보정하고 재검증한다. #8308은 실제 paint 무변경과 클릭·Tab·선택 결과를 별도로 입증한다.
6. 검증을 마친 입력에 대해서만 정식 관계 회귀와 측정 첫머리를 보완한다. 기존 정확한 기대값·Storage 보안 검사·공개 테스트 이관 보호를 유지하며 source-unit 예산을 늘리지 않는다.
7. 제품 보정이 있는 누적 후보는 Rust lint 묶음·전체 release-test nextest·Native 3종·fresh WASM·관련 Studio/API를 공용 `target/pr-review`에서 순차 완료한다. 독립 source의 녹색 CI를 새 보정 후보의 검증으로 대신하지 않는다.
8. 각 원 PR의 최종 판정·후속 처리안을 정리한다. push·공개 댓글/review·PR 생성·merge는 별도의 실제 원격 승인 범위로 구분한다. 수용할 PR은 원 기여와 검증 범위를 명시하고 필요한 보정/댓글도 PR별로 준비한다.

## 검증 현황과 권한

- 완료: 대상 선정, 10건 reviewer 지정/확인, 최신 devel 동기화, 원 head 보존 fetch, 원 CI/코드·입력 목록 조사, 10건 merge-tree.
- 기존 원 head 체크에 failure/timed_out/cancelled는 없다. neutral/skip은 실행 성공으로 세지 않았다. 현재 기준에서의 재현·누적 후보의 실행·시각 검증은 아직 수행하지 않았다.
- 로컬 코드 보정은 [local_validation.md 4.2](../manual/pr_review/local_validation.md#42-merge-simulation)의 “conflict 해결 방침은 작업지시자 결정이 필요하다”에 따라 위 다섯 PR의 통합 방침 승인을 기다린다.
- 이번 계획 승인은 10건의 로컬 보정·필수 검증을 한 단위로 완료하는 범위다. 원격 제출·merge 승인과 구분한다.

[선정·reviewer·원 head CI·충돌 원본 증거](assets/semanticist21-20261103/intake.json).
