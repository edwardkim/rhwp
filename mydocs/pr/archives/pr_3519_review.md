# PR #3519 검토 — SectionDef CTRL_DATA 중복 직렬화 제거

- 검토일: 2026-07-30
- PR: [#3519](https://github.com/edwardkim/rhwp/pull/3519)
- 관련 이슈: [#3507](https://github.com/edwardkim/rhwp/issues/3507)
- 작성자 / reviewer: `@postmelee` / `@jangster77` (collaborator 매개 외부 PR)
- base / current code head: `devel` `2df8106b8d99251beb93cac9c40b0ccaf4696120` /
  `dc9f7e2eb39f6bf254a23da35d06750e531fcf1c`
- 원 code 변경 규모: 15 files, +998 / -96 (검토 기록 추가 전)

## 변경 범위와 판정

HWP5 `SectionDef`의 첫 중첩 `CTRL_HEADER` 전 첫 직접 자식 `CTRL_DATA`가 기존에는
`Paragraph.ctrl_data_records`와 `SectionDef.extra_child_records` 양쪽에 보존됐다. 저장 시 두 경로가
같은 280-byte payload를 두 번 직렬화해, rhwp 자체 재로드는 되지만 한컴이 저장본을 손상 파일로 거부할 수
있었다.

이번 수정은 그 레코드의 canonical owner를 문단 control 슬롯으로 고정하고, `SectionDef` raw child에서는
제외한다. 추가 직접 자식, 중첩 control의 자식, 중첩 header 뒤의 직접 자식은 원래 raw 위치에 남긴다.
serializer의 legacy 방어도 동일 level·payload가 같고 같은 경계 안에 있는 경우만 제거하므로, 같은 payload라도
중첩 header 뒤의 레코드나 payload가 다른 직접 자식을 삭제하지 않는다.

parser와 serializer unit 회귀는 각각 단일 소유, 추가 direct child 보존, 중첩 child 보존, 경계 뒤 같은
payload 보존을 고정한다. 실물 `samples/복학원서.hwp` 회귀는 CLI `edit set-cell` 저장본의 SectionDef
직접 자식 `CTRL_DATA`가 원본과 같은 280-byte payload로 정확히 한 번만 기록되고 수정 셀이 재독되는지
CFB record 수준에서 확인한다. 이 소유권 변경에 맞춰 IR field-sweep baseline의 기존
`section_def.extra_child_records.len` 발산 항목이 제거됐다.

이는 HWP5 저장 레코드 계약을 바로잡는 parser/serializer 변경이며 renderer·layout·paint·페이지
geometry는 바꾸지 않는다. 따라서 새 PDF/SVG visual sweep은 merge 판단의 필수 근거로 쓰지 않았고,
실물 저장 파일의 byte-level record 회귀를 직접 근거로 삼았다.

## 검증

최신 `upstream/devel` `2df8106b8d99251beb93cac9c40b0ccaf4696120` 위의 merge simulation은 충돌 없이
완료됐고 `git diff --cached --check`를 통과했다. Cargo 검증은
`CARGO_TARGET_DIR=target/review-pr3519`, `CARGO_INCREMENTAL=0`으로 순차 실행했다.

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --test issue_3507_sectiondef_ctrl_data` | 1 passed, 0 failed |
| `cargo test --profile release-test --lib section_def_` | 8 passed, 0 failed |
| `cargo test --profile release-test --tests` | exit 0; library 3,025 passed, 0 failed, 7 ignored 및 전체 integration 통과 |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy --all-targets -- -D warnings` | passed |
| GitHub Actions — CI | preflight, Lint, test archive, Native Skia, default-feature 8 shards, `Build & Test` 모두 success |
| GitHub Actions — CodeQL | preflight와 JavaScript/TypeScript·Python·Rust 분석, aggregate 모두 success |
| GitHub Actions — Canvas visual diff | success (serializer 범위의 보조 CI 근거) |

## 권고와 merge 전 조건

**권고: 수용.** `dc9f7e2eb39f6bf254a23da35d06750e531fcf1c`의 full CI, CodeQL, Canvas visual diff와
`MERGEABLE`·`CLEAN` 상태를 확인했고 GitHub approval도 제출했다. 이 archive review 문서만 추가한
latest head가 review-only fast-pass preflight와 `Build & Test` aggregate를 통과하고 mergeable 상태를
유지하는지 재확인한 뒤, 작업지시자 승인 범위에서 squash merge한다. merge 뒤에는 #3507 종료 확인,
contributor comment, devel sync와 이 review worktree·local branch 정리를 수행한다.
