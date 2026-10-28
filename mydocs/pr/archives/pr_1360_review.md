# PR #1360 검토 — HWPX 표 셀 탭/줄바꿈 인라인 직렬화

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1360 |
| 작성자 | `Mireutale` |
| 상태 | open / draft 아님 |
| base | `devel` |
| head | `Mireutale:fix/issue-1353-hwpx-cell-inline` |
| 관련 이슈 | #1353 |
| 변경 규모 | 2 files, +34 / -13 |
| mergeable | `MERGEABLE`, `BEHIND` |

## 2. 변경 요약

HWPX serializer가 표 셀 문단 안의 탭과 소프트 줄바꿈을 일반 문자로 `<hp:t>`에 직접 쓰던 문제를 정정한다.

관련 이슈 #1353의 증상:

- `src/serializer/hwpx/table.rs::write_cell_text()`가 `\t`, `\n`을 raw text로 출력
- 본문 문단 경로는 이미 `src/serializer/hwpx/section.rs::render_hp_t_content()`에서
  - `\t` → `<hp:tab .../>`
  - `\n` → `<hp:lineBreak/>`
  로 변환
- 표 셀 문단도 본문 문단과 같은 HWPX inline element 규약을 따라야 함

핵심 변경:

- `render_hp_t_content()`를 `pub(crate)`로 공개
- 표 셀 문단 직렬화에서 `para.tab_extended`를 함께 전달
- 표 셀 텍스트 출력 시 본문 문단과 동일한 inline serializer 재사용
- 표 셀 안 `A\tB\nC`가 `<hp:tab/>`, `<hp:lineBreak/>`로 출력되는 단위 테스트 추가

## 3. GitHub 상태

GitHub Actions:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL Analyze (javascript-typescript) | pass |
| CodeQL Analyze (python) | pass |
| CodeQL Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

PR 대화:

- 추가 코멘트 없음

관련 이슈:

- #1353은 open 상태
- PR 본문은 related issue로 #1353을 언급하며 자동 close 키워드는 사용하지 않음

## 4. 로컬 검토 방식

검토 기준:

```text
origin/devel @ 4b06a162
PR head     @ f41314e8
```

로컬 브랜치:

```text
local/pr1360-upstream   @ f41314e8
local/pr1360-merge-test @ 4bc20b82
```

적용 방식:

- `origin/devel` 기준 검증 브랜치 생성
- PR 단일 커밋 `f41314e8` cherry-pick
- 충돌 없음

## 5. 로컬 검증

실행 완료:

```bash
git diff --check origin/devel..HEAD
cargo fmt --all -- --check
cargo test --lib serializer::hwpx
cargo test --test issue_1267_hwpx_tab_and_diagonal
cargo test --test issue_1244_tab_extended_fallback
cargo test --test hwpx_roundtrip_integration
cargo clippy -- -D warnings
```

결과:

| 명령 | 결과 |
|---|---|
| `git diff --check` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib serializer::hwpx` | 통과, 89 passed |
| `issue_1267_hwpx_tab_and_diagonal` | 통과, 2 passed |
| `issue_1244_tab_extended_fallback` | 통과, 3 passed |
| `hwpx_roundtrip_integration` | 통과, 22 passed |
| `cargo clippy -- -D warnings` | 통과 |

## 6. 검토 의견

차단 이슈는 발견하지 못했다.

수용 가능한 이유:

- 문제 지점인 표 셀 문단 직렬화만 좁게 수정한다.
- 본문 문단에서 이미 사용 중인 `render_hp_t_content()`를 재사용하므로 HWPX inline 출력 규약이 한 곳으로 모인다.
- 일반 텍스트는 기존 helper 내부의 XML escaping을 거쳐 출력되므로 raw XML write로 바뀌어도 텍스트 escape 계약은 유지된다.
- `para.tab_extended`를 표 셀 경로에도 전달하여 탭 폭, leader, type 보존 흐름이 본문과 일치한다.
- 신규 테스트가 표 셀 안 탭/소프트 줄바꿈의 XML 출력을 직접 확인한다.
- 기존 HWPX serializer, 탭 확장, HWPX roundtrip 통합 테스트가 모두 통과했다.

주의점:

- `write_cell_text()`가 `Writer`의 inner writer에 생성된 XML 조각을 직접 쓰는 구조가 되었다. 다만 이 방식은 helper가 완성된 HWPX inline XML 조각을 반환하고, 텍스트 escape를 helper 내부에서 수행하므로 현재 변경 범위에서는 문제가 확인되지 않았다.
- 표 셀 serializer는 여전히 한 문단을 하나의 `hp:run`으로 출력하는 기존 구조를 유지한다. char shape run 분할 같은 더 넓은 HWPX fidelity 문제는 이번 PR 범위가 아니다.
- PR은 `BEHIND` 상태이나 최신 `origin/devel` 기준 cherry-pick은 충돌 없이 적용됐다.

## 7. 권장 처리

권장: **수용 가능**.

권장 절차:

1. 작업지시자 승인.
2. `local/devel`에 PR 커밋 반영.
3. 최종 검증:
   - `git diff --check`
   - `cargo fmt --all -- --check`
   - `cargo test --lib serializer::hwpx`
   - `cargo test --test issue_1267_hwpx_tab_and_diagonal`
   - `cargo test --test hwpx_roundtrip_integration`
4. 처리 보고서 작성.
5. `origin/devel` push.
6. PR #1360에 메인테이너 코멘트 작성 후 close.
7. Issue #1353은 자동 close되지 않으므로 필요 시 별도 close.

## 8. 승인 요청

위 검토 결과 기준으로 PR #1360 수용 절차를 진행해도 되는지 승인 요청한다.
