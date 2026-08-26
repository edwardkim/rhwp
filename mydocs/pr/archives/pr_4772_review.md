# PR #4772 검토 - ParaShape 자동 간격과 쪽나눔 보호 비트 분리

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4772](https://github.com/edwardkim/rhwp/pull/4772) |
| 관련 이슈 | `Closes #2777` |
| 작성자 | `jangster77` |
| 검토 방식 | 작성자 self-review |
| base / head | `devel` / `task_m100_2777_parashape_attr_contract` |
| code candidate | `4421bb9811f34eb9672e787bda74de117c949026` |
| 규모 | 3 commits, 10 files, +254 / -36 |
| 작성 시점 상태 | `MERGEABLE`, `CLEAN` |

`mergeable`, `mergeStateStatus`, head SHA 및 CI 결과는 작성 시점의 참고값이다. 최종 merge 직전에
trailing head와 GitHub Actions 상태를 다시 확인한다.

## 변경 범위와 판단

- HWPX `breakSetting`은 ParaShape `attr1`의 16~19비트, `autoSpacing`은 `attr2`의 4~5비트로
  읽고 쓰도록 분리했다.
- HWP5 구규약 이관은 식별 가능한 쪽나눔 보호 비트만 대상으로 한다. `attr2` bit 5는
  auto-spacing과 구별할 수 없으므로 보호 속성으로 재해석하지 않고 보존한다.
- parser, serializer, style resolver, 서식 명령이 같은 비트 계약을 사용하도록 맞추고 세 회귀
  단정을 추가했다.
- PR 준비와 collaborator 외부 PR의 문서 절차를 명확히 하는 문서 변경도 함께 포함한다.
- `src/renderer/style_resolver.rs`를 수정했지만 paint 또는 layout 알고리즘은 변경하지 않았다.
  실제 HWPX 왕복과 GitHub Canvas visual diff로 출력 경로 무회귀를 확인했다.

## 완료된 검증

- `cargo build`를 성공했다.
- 다음 focused 단정을 실행해 통과했다.
  - `para_shape_break_setting_and_auto_spacing_use_distinct_bits`
  - `write_para_pr_does_not_treat_auto_spacing_num_as_widow_orphan`
  - `para_shape_migrates_identifiable_legacy_break_bits_without_repurposing_bit5`
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 10 --no-fail-fast`를
  실행해 6,023개 통과, 38개 skip, 실패 0건을 확인했다.
- `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets -- -D warnings`를 모두 통과했다.
- `issue1853_caption_precedes_body_split.hwpx`를 HWPX로 왕복해 `--verify --verify-pages`를
  실행했다. IR 차이는 없었고 페이지 수는 52쪽으로 유지됐다.
- code candidate의 GitHub Actions도 통과했다.
  - [CI](https://github.com/edwardkim/rhwp/actions/runs/31800461851): Lint, Native Skia, archive,
    regular 3 shards, slow shard, Build & Test aggregate 성공
  - [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31800461657): Rust 분석 성공
  - [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31800461568): Canvas visual diff 성공

## 위험과 후속 범위

- HWP5 `attr2` bit 5의 구규약 의미는 auto-spacing과 충돌해 손실 없이 판별할 수 없다. 이번 변경은
  이를 widow/orphan 보호로 오판하지 않아 기존 auto-spacing 값을 보존한다.
- 이번 PR은 ParaShape 속성 계약만 다룬다. 별도의 줄바꿈·쪽나눔 레이아웃 보정은 범위 밖이다.
- 추가 결함은 발견하지 못했다.

## 최종 권고

조건부 merge를 권고한다. 이 문서와 오늘할일만 담은 trailing head의 preflight 및 Build & Test
aggregate가 성공하고, merge 직전에 최신 head SHA, `MERGEABLE`, `CLEAN`을 다시 확인한 뒤에만
작업지시자가 명시한 maintainer `--admin` squash merge를 수행한다.
