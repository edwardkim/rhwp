# PR #1364 검토 — HWPX 양식 개체 직렬화 + 표 열 폭 제어

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1364 |
| 작성자 | `physwkim` |
| 작성자 association | `FIRST_TIME_CONTRIBUTOR` |
| 상태 | open / draft 아님 |
| base | `main` |
| head | `physwkim:feature/hwpx-form-writer` |
| head repo | `physwkim/rhwp` |
| maintainer_can_modify | true |
| 변경 규모 | 8 files, +778 / -8 |
| mergeable | `MERGEABLE` |

## 2. 변경 요약

PR #1364는 hangul-mcp 쪽에서 필요한 기능으로 다음 3가지를 추가한다.

1. HWPX FormObject writer 추가
   - `src/serializer/hwpx/form.rs` 신규
   - `Control::Form`을 HWPX `hp:btn`, `hp:checkBtn`, `hp:radioBtn`, `hp:comboBox`, `hp:edit`로 직렬화
   - 기존 `serialize_hwpx`에서 폼 컨트롤이 저장 시 사라지던 문제를 보완

2. HWPX FormObject 속성 round-trip 보존 확대
   - parser가 `backStyle`, `radioGroupName`, `triState`, edit 전용 속성, `hp:sz` relTo/protect, `hp:pos`, `hp:outMargin`, `listItem displayText`를 `FormObject.properties`에 보존
   - writer가 동일 property key를 읽어 출력

3. 표 열 폭 제어 API 추가
   - `Table::set_column_widths(&[HwpUnit])`
   - `DocumentCore::set_table_column_widths_native(...)`
   - `DocumentCore::fit_table_to_page_native(...)`
   - 열 폭 변경 후 셀 문단 reflow, section raw stream invalidation, recompose/paginate 수행

## 3. GitHub 상태

GitHub Actions:

| 항목 | 결과 |
|---|---|
| statusCheckRollup | 없음 |
| `gh pr checks` | `no checks reported on the 'feature/hwpx-form-writer' branch` |

PR 대화:

- 추가 코멘트 없음

절차상 주의:

- 작성자는 `FIRST_TIME_CONTRIBUTOR`라서 GitHub Actions가 아직 실행되지 않은 상태로 보인다.
- PR base가 `main`이다. 최근 프로젝트 처리 흐름은 `devel` 기준이므로 GitHub에서 그대로 merge하면 안 된다.

## 4. 로컬 검토 방식

검토 기준:

```text
origin/devel @ 3d4c454e
PR head     @ 17d6f445
```

PR 커밋:

```text
76efb21d feat(serializer/hwpx): add FormObject writer
18b1f1b5 feat(hwpx/form): round-trip all standard form attributes
17d6f445 feat(table): set_column_widths + set/fit native table-width commands
```

로컬 브랜치:

```text
local/pr1364-upstream   @ 17d6f445
local/pr1364-merge-test @ 7f0a54b0
```

적용 방식:

- `origin/devel` 기준 검증 브랜치 생성
- PR 커밋 3개 순서대로 cherry-pick
- 첫 커밋에서 충돌 1건 발생
  - 파일: `src/serializer/hwpx/section.rs`
  - 위치: `render_control_slot`
  - 원인: devel의 `PageHide`/`PageNumberPos`/`Header`/`Footer`/`AutoNumber` 분기와 PR의 `Control::Form` 분기가 같은 위치에 추가됨
  - 해결: 양쪽 분기를 모두 유지

## 5. 로컬 검증

실행 완료:

```bash
git diff --check origin/devel..HEAD
cargo fmt --all -- --check
cargo test --test hwpx_form_roundtrip
cargo test --lib serializer::hwpx
cargo test --lib model::table
cargo test --lib document_core::commands::table_ops
cargo test --test hwpx_roundtrip_integration
cargo test --lib
cargo clippy -- -D warnings
```

결과:

| 명령 | 결과 |
|---|---|
| `git diff --check` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `hwpx_form_roundtrip` | 통과, 1 passed |
| `cargo test --lib serializer::hwpx` | 통과, 94 passed |
| `cargo test --lib model::table` | 통과, 57 passed |
| `cargo test --lib document_core::commands::table_ops` | 통과, 2 passed |
| `cargo test --test hwpx_roundtrip_integration` | 통과, 22 passed |
| `cargo test --lib` | 통과, 1631 passed / 0 failed / 6 ignored |
| `cargo clippy -- -D warnings` | 통과 |

## 6. 검토 의견

차단 이슈는 현재 로컬 검증에서는 발견하지 못했다.

수용 가능하다고 보는 이유:

- HWPX form writer는 기존 `parse_form_object`의 역방향으로 구성되어 있고, `Control::Form` 직렬화 누락을 직접 해결한다.
- parser와 writer가 같은 `FormObject.properties` key를 사용해 표준 form 속성을 round-trip하도록 맞춘 점은 타당하다.
- 신규 `tests/hwpx_form_roundtrip.rs`가 `samples/hwpx/form-01.hwpx`의 5개 폼 보존을 확인한다.
- `Table::set_column_widths`는 병합 셀 폭을 span 열 폭 합으로 계산하고, 기존 `update_ctrl_dimensions`/`rebuild_grid` 흐름과 맞춰져 있다.
- `set_table_column_widths_native`는 변경 후 cell paragraph reflow, raw_stream invalidation, recompose, paginate까지 수행한다.
- devel 기준 충돌은 단순 additive 충돌이며 수동 해결 후 전체 lib/clippy가 통과했다.

주의점:

- GitHub CI가 아직 실행되지 않았다. first-time contributor PR이므로 maintainer가 workflow 실행을 승인하거나, devel 반영 후 GitHub CI를 별도로 확인해야 한다.
- PR base가 `main`이다. 그대로 GitHub merge하면 프로젝트의 `devel` 중심 흐름과 어긋난다. 수용 시 지금처럼 `devel` 기준 cherry-pick 처리하는 편이 맞다.
- `fit_table_to_page_native`는 `MIN_COL = 200` 보정 후 새 폭 합이 `target`을 초과할 수 있다. target이 너무 좁은 경우 물리적으로 피할 수 없는 상황일 수 있으나, 반환 JSON의 `changed: true`와 `tableWidth > pageContentWidth` 가능성은 후속 개선 포인트다.
- 새 table width native API는 이 PR 안에서는 wasm/frontend 라우팅에 연결되지 않는다. hangul-mcp에서 Rust API를 직접 사용할 목적이라면 문제 없지만, rhwp-studio 사용자 기능으로 바로 노출되는 변경은 아니다.
- HWPX serializer의 form fidelity는 샘플 기반으로 검증됐다. 다양한 실제 form 문서에 대한 시각 판정은 별도 진행이 필요하다.

## 7. 권장 처리

권장: **조건부 수용 가능**.

조건:

1. PR을 GitHub에서 `main`에 직접 merge하지 않는다.
2. `devel` 기준 cherry-pick 방식으로 반영한다.
3. GitHub CI가 아직 없으므로, 반영 전후로 maintainer가 CI 실행 상태를 확인한다.
4. 필요 시 `samples/hwpx/form-01.hwpx` roundtrip 결과를 rhwp-studio 또는 한컴에디터로 추가 확인한다.

권장 절차:

1. 작업지시자 승인.
2. `local/devel`에 PR 커밋 3개 반영.
3. `render_control_slot` 충돌은 검토 브랜치와 동일하게 양쪽 분기 유지로 해결.
4. 최종 검증:
   - `git diff --check`
   - `cargo fmt --all -- --check`
   - `cargo test --test hwpx_form_roundtrip`
   - `cargo test --lib serializer::hwpx`
   - `cargo test --test hwpx_roundtrip_integration`
   - `cargo test --lib`
   - `cargo clippy -- -D warnings`
5. 처리 보고서 작성.
6. `origin/devel` push.
7. PR #1364에 메인테이너 코멘트 작성 후 close.

## 8. 승인 요청

위 검토 결과 기준으로 PR #1364를 `devel` 기준 cherry-pick 방식으로 수용 처리해도 되는지 승인 요청한다.
