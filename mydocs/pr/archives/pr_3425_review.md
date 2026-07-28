# PR #3425 검토 기록 — 컨트롤 삽입 시 `ctrl_data_records` 정합

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3425](https://github.com/edwardkim/rhwp/pull/3425) — `fix(edit): 컨트롤 삽입이 ctrl_data_records 길이 불일치에서 패닉 (#3214)` |
| 작성자·검토자 | `@chrisryugj` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `3984f2a8398359ed0c8bd0b93d99bb648f2ab0b5` (`chrisryugj/rhwp_fork`) |
| 원 변경 규모 | 6 files, +89 / -0; 기능 commit `683c14e090a1f713b09d61248d873962d6f93742` (source의 devel merge commit은 미적용) |
| 통합 검토 | `review/chrisryugj-20260727`, 기준 `upstream/devel` `2d7303c5bea13eaf072e782cd7f7b4a6db59b35e`; `683c14e0…` → `597dabf07` |
| 관련 이슈 | [#3214](https://github.com/edwardkim/rhwp/issues/3214) |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`; CI·CodeQL·Render Diff·Native Skia·8개 default-feature shard 모두 성공 |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`.

## 원 변경과 판정

`controls`의 삽입 위치는 control 축에서 계산하는데 HWPX 파서는 HWP5 전용 `CTRL_DATA` 레코드를
채우지 않아 `ctrl_data_records.len() < controls.len()`이 정상일 수 있다. 기존 코드는 그 control
인덱스로 더 짧은 vector에 `insert()`해 각주·미주·수식·도형·새 번호 삽입에서 범위 초과 패닉을 낼 수
있었다.

PR은 `Paragraph::align_ctrl_data_records()`를 추가해 부족한 슬롯만 `None`으로 채우고, header/footer
생성 및 모든 해당 인라인 control 삽입 경로에서 control/record 양쪽 insert 전에 호출한다. 더 긴 record
vector를 자르지 않아 기존 HWP5의 추가 원본 정보도 보존한다. 두 회귀 시험은 (1) header 생성 뒤 record
길이 정합과 각주 삽입, (2) HWPX와 같은 의도적 불일치 상태에서의 각주 삽입을 검증한다. 코드상
차단점은 발견하지 못했다.

## Fixture·시각 증적 판단

배열 인덱스 정합과 패닉 방지가 전부이며 renderer·serialization 형식·페이지 배치는 바꾸지 않는다.
새 HWP/HWPX fixture도 추가하지 않았다. 그러므로 visual sweep 및 PNG 증적은 적용 대상이 아니며,
IR field-sweep baseline TSV 등록 trigger도 없다.

## 검증

- `issue3214_header_creation_keeps_ctrl_data_records_aligned`,
  `issue3214_insert_survives_unaligned_ctrl_data_records`: 성공.
- 통합 후보 `cargo test --profile release-test --tests`: **2,962 passed / 0 failed**, IR field sweep 포함.
- Native Skia 공식 3종: **57/0**, **2/0**, **4/0**.
- `cargo fmt --all -- --check`, `git diff --check`, `cargo clippy -- -D warnings`,
  `cargo check --target wasm32-unknown-unknown --lib`: 성공.

## 최종 권고

**기술적으로 수용 가능**. #3421의 공용 HWPX XML 보정과 함께 같은 통합 PR에 넣어도 충돌이나
행동 중첩이 없다. 최신 통합 head CI·mergeable 및 작업지시자 승인을 최종 merge 조건으로 둔다.
