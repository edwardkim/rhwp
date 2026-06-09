# PR #1324 리뷰 — 빈 문단 0-length field fieldBegin/fieldEnd 순서 보정

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1324 |
| 제목 | Task #1321 (#1289,#1298의 후속 작업) fix: 빈 문단(text == "") + 0-length field fieldBegin/fieldEnd 순서 역전 수정 |
| 작성자 | Martinel2 |
| 대상 이슈 | #1321 |
| 대상 브랜치 | `devel` |
| PR base SHA | `ea23403786649c5c601d2577a637605579a52cec` |
| PR head SHA | `f1165a0cf42cb8133d5a5ebb04ca24d7762cc71d` |
| 현재 devel | `a29987793316d43a414f2f66147f401e00ace834` |
| 상태 | open / draft 아님 / mergeable |

## 2. 변경 범위

PR은 총 11개 파일, +865줄 변경이다.

| 구분 | 파일 | 판단 |
|---|---|---|
| 기능 수정 | `src/serializer/hwpx/section.rs` | 실제 수용 대상 |
| 작업 문서 | `mydocs/plans/task_m100_1321.md` | PR branch 포함 문서 |
| 작업 문서 | `mydocs/plans/task_m100_1321_impl.md` | PR branch 포함 문서 |
| 작업 문서 | `mydocs/report/task_m100_1321_report.md` | PR branch 포함 문서 |
| 작업 문서 | `mydocs/working/task_m100_1321_stage1.md` | PR branch 포함 문서 |
| 작업 문서 | `mydocs/working/task_m100_1321_stage2.md` | PR branch 포함 문서 |
| 작업 문서 | `mydocs/working/task_m100_1321_stage3.md` | PR branch 포함 문서 |
| 과거 문서 | `mydocs/pr/pr_1299_review.md` | 현재 `mydocs/pr/archives/`에 이미 존재 |
| 과거 문서 | `mydocs/pr/pr_1299_report.md` | 현재 `mydocs/pr/archives/`에 이미 존재 |
| 과거 문서 | `mydocs/report/task_m100_1298_report.md` | 과거 task 문서 재도입 |
| 과거 문서 | `mydocs/working/task_m100_1298_stage1.md` | 과거 task 문서 재도입 |

현재 프로젝트 문서 정리 정책상 PR 리뷰/보고 문서는 archive로 이동되어 있으므로, 이번 PR을 그대로 머지하면 `pr_1299_*` 문서가 active 폴더에 재도입된다. 따라서 기능 코드는 수용하되 문서 파일은 maintainer-side 통합에서 제외하는 편이 안전하다.

## 3. 문제 구조

#1298에서는 일반 문단 텍스트 안의 0-length field range 순서를 보정했다. 그러나 `para.text == ""`인 빈 문단은 메인 문자 루프가 0회 실행되기 때문에 #1298에서 추가한 pre-char 검사를 통과하지 않는다.

기존 흐름:

1. `for (idx, c) in para.text.chars().enumerate()`가 0회 실행
2. post-loop fieldEnd 처리에서 `<hp:fieldEnd>`가 먼저 출력
3. remaining slots 처리에서 `<hp:fieldBegin>`이 나중에 출력

결과적으로 HWPX XML 순서가 `<fieldEnd>` → `<fieldBegin>`으로 뒤집힌다.

## 4. PR 수정 방식

`render_run_content`에서 `field_end_emitted` 초기화 직후 `para.text.is_empty()` 전용 블록을 추가한다.

- 빈 문단이면 남은 slots를 먼저 모두 출력한다.
- 이후 0-length field range의 `fieldEnd`를 출력한다.
- `slot_idx == slots.len()` 및 `field_end_emitted[i] == true` 상태가 되므로 기존 post-loop 처리는 no-op이 된다.

단일 필드 케이스에서는 `fieldBegin`이 `fieldEnd`보다 먼저 출력된다는 목적을 정확히 달성한다.

## 5. 로컬 검증

검증 브랜치: `local/pr1324-upstream`

```text
cargo fmt --all -- --check
통과

cargo test --lib -- serializer::hwpx::section::tests::task1321_zero_length_field_in_empty_paragraph --nocapture
1 passed

cargo test --lib -- serializer::hwpx::section::tests --nocapture
15 passed

cargo clippy --lib -- -D warnings
통과
```

## 6. 검토 의견

기능 수정 방향은 수용 가능하다. #1298에서 의도적으로 남겨둔 빈 문단 + 0-length field 케이스를 좁게 메우며, 기존 `task1298_*` 테스트도 함께 통과한다.

다만 현재 구현은 빈 문단에서 `slots`를 모두 먼저 방출한 뒤 `fieldEnd`를 방출한다. 이번 이슈의 단일 field 조건에서는 충분하지만, 빈 문단에 여러 inline slot이 섞인 더 복잡한 케이스에서는 `fieldBegin`/`fieldEnd` 쌍의 정확한 interleaving을 별도 검증할 필요가 있다. 현재 PR의 blocker로 보지는 않는다.

## 7. 권장 처리

권장안: **코드 변경만 maintainer-side로 수용**

이유:

- PR base가 현재 `devel`보다 뒤처져 있다.
- 기능 수정은 작고 검증 결과가 좋다.
- PR에 포함된 `mydocs/pr/pr_1299_*` 문서는 이미 archive에 있는 문서를 active 폴더로 재도입한다.
- `mydocs/plans/report/working`의 task 문서도 현재 문서 정리 정책과 충돌할 가능성이 있다.

권장 절차:

1. `local/devel` 기준 통합 브랜치 생성
2. `src/serializer/hwpx/section.rs` 변경만 적용
3. `cargo fmt --all -- --check`
4. `cargo test --lib -- serializer::hwpx::section::tests --nocapture`
5. `cargo clippy --lib -- -D warnings`
6. maintainer-side 커밋 후 `devel` push
7. PR #1324에 수동 통합 코멘트 작성 후 close
8. Issue #1321 close 여부 확인 및 필요 시 수동 close

## 8. PR 코멘트 초안

```markdown
검토했습니다. #1298에서 남겨둔 빈 문단(`text == ""`) + 0-length field 케이스에서 `fieldEnd`가 `fieldBegin`보다 먼저 출력되는 문제를 정확히 좁혀 처리한 것으로 확인했습니다.

로컬에서 다음 검증을 통과했습니다.

- `cargo fmt --all -- --check`
- `cargo test --lib -- serializer::hwpx::section::tests::task1321_zero_length_field_in_empty_paragraph --nocapture`
- `cargo test --lib -- serializer::hwpx::section::tests --nocapture`
- `cargo clippy --lib -- -D warnings`

다만 PR branch에 과거/작업 문서가 함께 포함되어 있어, 프로젝트 문서 정리 정책상 기능 코드만 maintainer-side로 반영하는 방식으로 처리하겠습니다. 감사합니다.
```
