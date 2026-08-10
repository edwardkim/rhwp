# PR #4507 검토 기록 — #3820 Production Fidelity 동기화

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#4507](https://github.com/edwardkim/rhwp/pull/4507) |
| 작성자 / base | `jangster77` / `devel` |
| 작업 브랜치 | `task/3820-production-fidelity` |
| source commit | `2d403b182` |
| 현재 head commit | `2d403b182` |
| 변경 규모 | additions `71993`, deletions `819`, files `100`, commits `75` |
| mergeable 상태 | `MERGEABLE` (mergeStateStatus: `BLOCKED`, 작성 시점 참고값) |

## 라우팅과 범위

- 기본 경로: [collaborator self-merge](../pr_review/collaborator_self_merge.md)
- 보조 경로: [intake_and_review](../pr_review/intake_and_review.md), [local_validation](../pr_review/local_validation.md)

## 변경 요약

- `task/3820-production-fidelity` 브랜치 기준으로 `upstream/devel`에 대해 리베이스를 수행해 최신 변경을 반영함.
- 리베이스 과정에서 충돌이 난 7개 파일(`src/renderer/layout.rs`, `src/renderer/layout/table_partial.rs`, `src/renderer/page_number.rs`, `src/renderer/pagination.rs`, `src/renderer/pagination/engine.rs`, `src/renderer/typeset.rs`, `src/document_core/queries/cursor_rect.rs`)을 수동 정리.
- `#3820` 연계 증적(`mydocs/pr/assets/task_m100_3820_*`)과 `tests/issue_3820_rowbreak_rowspan_band.rs`를 포함해 시각·회귀 증거를 유지.

## 검증 상태

- PR 생성 시점 기준: `upstream/devel` 기준 리베이스 완료 및 강제 push(`git push --force-with-lease upstream HEAD:task/3820-production-fidelity`).
- 현재는 review 문서/리뷰 구현 계획서는 PR 생성 전 점검 단계로, 로컬 전체 회귀 검증은 아직 별도 실행하지 않음.
- 로컬 실행 이력은 필요 시 review 단계에서 `review.md`에 과거형으로 정리 예정.

## 권고

- PR 번호가 확정된 상태이므로 reviewer assign 후, required check 통과 및 작업지시자 승인 뒤에 merge 판단.
