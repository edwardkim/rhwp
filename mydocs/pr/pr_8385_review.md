# PR #8385 검토 — feat(table): 셀 블록의 너비·높이를 한컴처럼 격자를 나눠 같게 맞춘다

## 최종 판정

**머지 보류 — 검증 진행 중.** 원 head를 고정하고 접수·사전 조사를 완료했다. 이 기록은 기여 거절이나 수용 완료가 아니며, 누적 보정·필수 로컬/시각 검증과 최종 exact-head CI·원격 승인 조건을 완료한 뒤 판정한다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 원 PR·작성자·base | [#8385](https://github.com/edwardkim/rhwp/pull/8385) / semanticist21 / devel |
| 원 repository·branch | `semanticist21/rhwp` / `feat/table-equal-cell-sizes` |
| 원 head | `6a31cb47a088ac13ecbe641900318c9dbc9ff4e3` |
| 고정한 최신 devel | `ed057276c1e0e07e743218d95641e015b46b6715` |
| reviewer·API 확인 | `edwardkim` / `2026-11-05T01:41:06.321854+00:00` UTC / `requested_reviewers.users` 확인·지정 전후 head 동일 |
| 규모·당시 mergeability | +664 / -0 / 16 files / `MERGEABLE`·`CLEAN` (조회 시점 참고값) |
| 검토 branch | `review/integration-20261105-semanticist21-next10` |

기본 경로는 maintainer 일반이며 접수·로컬 검증·다수 PR·시각 증적·후속 처리를 적용한다. 원 PR head·fork branch를 변경하지 않았다. [전체 범위·CI 근거·충돌 보정안](semanticist21_20261105_next10_review_plan.md)을 따른다.

## 원 기여 commit과 변경 범위

- `4e99c30af39deb5407bb8bf9eaa1e4261ce1c8a3` — feat(table): 셀 블록의 너비·높이를 한컴처럼 격자를 나눠 같게 맞춘다
- `6a31cb47a088ac13ecbe641900318c9dbc9ff4e3` — feat(table): 줄마다 칸 수가 다른 블록은 줄마다 그 칸 수로 나눠 맞춘다

제품 변경 경로: `src/document_core/commands/table_ops.rs`, `src/model/table.rs`, `src/model/table_equal_size.rs`, `src/wasm_api.rs`.

기준 devel에서 기존 해결 여부는 코드 존재만으로 확정하지 않고 실제 원본·제품 API와 대조한다. 현재는 본문·실제 diff·독립 자료와 기존 원 해결 이력의 조사를 진행 중이다. 공통 조판 원칙과 실제 측정/배치/저장 소비 경로의 충족 여부는 미검증이며, 해당 검증을 완료하기 전에는 수용하지 않는다.

## CI와 병합 시뮬레이션

실패 check는 발견하지 않았다. 원 PR의 녹색 결과를 새 통합 후보의 통과로 대신하지 않는다.

- `git merge-tree --write-tree --name-only` exit: `0`.
- 계산 tree: `42d37b9e11f9abcc9920ae356b258724756b51c0`.
- 충돌: 개별 merge-tree는 충돌 없이 종료했다. 누적 적용은 미실행이다.

CI 실패의 메인터너 처리 판단은 작업지시자의 위임 범위에서 수행했다. workflow 취소·재실행·정책 변경이나 source 원격 push는 수행하지 않았다. 두 충돌 PR의 보정은 계획서의 방침 승인 뒤 진행한다.

## 입력과 검증 상태

- `pdf/rowspan_staggered_heights/equal-height-partial-block-hancom-mac-12.30.pdf`
- `pdf/table_equal_cell_sizes/equal-width-partial-block-hancom-mac-12.30.pdf`
- `pdf/table_equal_cell_sizes/uneven-height-hancom-mac-12.30.pdf`
- `pdf/table_equal_cell_sizes/uneven-width-hancom-mac-12.30.pdf`
- `tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp`
- `tests/fixtures/table_equal_cell_sizes/equal-width-partial-block.hwp`
- `tests/fixtures/table_equal_cell_sizes/original.hwp`
- `tests/fixtures/table_equal_cell_sizes/uneven-height-hancom.hwp`
- `tests/fixtures/table_equal_cell_sizes/uneven-height.hwp`
- `tests/fixtures/table_equal_cell_sizes/uneven-width-hancom.hwp`
- `tests/fixtures/table_equal_cell_sizes/uneven-width.hwp`

원 head의 실제 Git blob·SHA-256과 기준 devel의 동일 파일 여부를 대조했다. 입력/PDF를 실행한 뒤에는 실행 파일과 검토 commit의 동일성 및 Print 출처·쪽수를 다시 확인한다. 위 목록은 원 PR 변경 목록이며, 누락된 실제 재현 입력·독립 Print 또는 저장 왕복 자료의 충분성을 보증하지 않는다.

집중/전체 회귀·Native/fresh WASM·저장 재열기·전체 TSV·직접 확대 판독은 아직 완료하지 않았다. 기존 contributor 측 숫자·화면·SVG 동일성 주장을 이번 통합 후보의 시각 통과로 기록하지 않는다. 적용 범위와 실제 결과를 확보한 뒤 검증 절을 확정한다.

## Merge 후 contributor PR comment 계획

아직 확정하지 않았다. 실제 원 PR이 주장한 문서·페이지의 Native/fresh WASM review·overlay와 지표·남은 차이를 확인한 뒤 안정 경로와 SHA 고정 URL을 이 절에 기록한다. 이미지·해시·원본/쪽 대응을 게시 전에, API 본문·실제 화면을 게시 뒤 확인한다. 이 접수 기록을 완료 댓글로 게시하지 않는다.
