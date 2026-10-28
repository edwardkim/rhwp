# PR #1364 처리 보고서 — HWPX 양식 개체 직렬화 + 표 열 폭 제어

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1364 |
| 작성자 | `physwkim` |
| 작성자 association | `FIRST_TIME_CONTRIBUTOR` |
| 관련 이슈 | 없음 |
| PR base | `main` |
| 처리 기준 | `devel` 기준 cherry-pick |
| 검토 브랜치 | `local/pr1364-merge-test` |
| 원 PR head | `17d6f445` |
| 반영 커밋 | `2093c5dd`, `543e9edc`, `77d0b618` |
| 문서 정리 커밋 | `ceb5ce51` |
| PR close | `2026-06-10T16:28:47Z` |

## 2. 처리 내용

PR #1364는 hangul-mcp에서 필요한 HWPX form round-trip 및 표 열 폭 제어 기능을 추가한다.

변경 요약:

- HWPX FormObject writer 추가
  - `src/serializer/hwpx/form.rs`
  - `Control::Form`을 HWPX `hp:btn`, `hp:checkBtn`, `hp:radioBtn`, `hp:comboBox`, `hp:edit`로 직렬화
  - `render_control_slot`에서 form control 출력 연결
- FormObject 속성 round-trip 보존 확대
  - `backStyle`, `radioGroupName`, `triState`
  - edit 전용 속성
  - `hp:sz` relTo/protect
  - `hp:pos`, `hp:outMargin`
  - `listItem displayText`
- 표 열 폭 제어 API 추가
  - `Table::set_column_widths`
  - `DocumentCore::set_table_column_widths_native`
  - `DocumentCore::fit_table_to_page_native`

## 3. devel 반영 방식

PR은 GitHub상 base가 `main`이므로 직접 merge하지 않았다.

`origin/devel @ 3d4c454e` 기준으로 PR 커밋 3개를 cherry-pick했다.

원 PR 커밋:

```text
76efb21d feat(serializer/hwpx): add FormObject writer
18b1f1b5 feat(hwpx/form): round-trip all standard form attributes
17d6f445 feat(table): set_column_widths + set/fit native table-width commands
```

devel 반영 커밋:

```text
2093c5dd feat(serializer/hwpx): add FormObject writer
543e9edc feat(hwpx/form): round-trip all standard form attributes
77d0b618 feat(table): set_column_widths + set/fit native table-width commands
```

충돌:

- 첫 커밋 적용 중 `src/serializer/hwpx/section.rs::render_control_slot`에서 충돌 발생
- 원인: devel의 page hide/page number/header/footer/autonum 분기와 PR의 form 분기가 같은 위치에 추가됨
- 해결: 양쪽 분기를 모두 유지

## 4. 검증 결과

GitHub checks:

| 항목 | 결과 |
|---|---|
| PR branch checks | 없음 |
| 사유 | first-time contributor PR로 Actions가 아직 보고되지 않음 |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `git diff --check origin/devel..HEAD` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --test hwpx_form_roundtrip` | 통과, 1 passed |
| `cargo test --lib serializer::hwpx` | 통과, 94 passed |
| `cargo test --lib model::table` | 통과, 57 passed |
| `cargo test --lib document_core::commands::table_ops` | 통과, 2 passed |
| `cargo test --test hwpx_roundtrip_integration` | 통과, 22 passed |
| `cargo test --lib` | 통과, 1631 passed / 0 failed / 6 ignored |
| `cargo clippy -- -D warnings` | 통과 |

## 5. 판정

**조건부 수용 완료**.

수용 조건:

- GitHub에서 `main`으로 직접 merge하지 않고 `devel` 기준 cherry-pick으로 처리
- devel 충돌은 양쪽 기능을 모두 유지하도록 해결
- first-time contributor PR의 GitHub checks 부재를 로컬 검증으로 보완

주의점:

- `fit_table_to_page_native`는 `MIN_COL = 200` 보정 후 새 폭 합이 `target`을 초과할 수 있다. target이 너무 좁은 경우의 후속 개선 포인트로 기록한다.
- 새 table width native API는 이 PR 안에서는 wasm/frontend 라우팅에 연결되지 않는다. rhwp-studio 사용자 기능으로 즉시 노출되는 변경은 아니다.
- HWPX form fidelity는 `samples/hwpx/form-01.hwpx` 기반 round-trip으로 검증했다. 더 다양한 실제 form 문서는 후속 시각 판정이 필요할 수 있다.

## 6. 후속 절차

처리 진행:

- [x] 작업지시자 조건부 수용 승인
- [x] `local/devel`에 PR 커밋 3개 반영
- [x] devel 충돌 해결
- [x] 최종 로컬 검증
- [x] 문서 정리 커밋 — `ceb5ce51`
- [x] `origin/devel` push — `ceb5ce51`
- [x] PR #1364에 메인테이너 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1364#issuecomment-4672215550
- [x] PR #1364 close — `2026-06-10T16:28:47Z`
