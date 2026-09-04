# PR #1360 처리 보고서 — HWPX 표 셀 탭/줄바꿈 인라인 직렬화

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1360 |
| 작성자 | `Mireutale` |
| 관련 이슈 | #1353 |
| 검토 브랜치 | `local/pr1360-merge-test` |
| 통합 방식 | 현재 `origin/devel` 기준 PR 단일 커밋 cherry-pick 검증 |
| 원 PR head | `f41314e8` |
| 반영 커밋 | `a547909e` |
| 문서 정리 커밋 | `1f98b1b6` |
| PR close | `2026-06-10T15:51:04Z` |
| Issue #1353 close | `2026-06-10T15:51:52Z` |

## 2. 처리 내용

PR #1360은 #1353에서 보고된 HWPX serializer의 표 셀 문단 inline element 누락 문제를 정정한다.

증상:

- 표 셀 문단 안의 탭 `\t`와 소프트 줄바꿈 `\n`이 `<hp:t>` 안에 raw text로 저장됨
- 본문 문단 경로는 이미 `<hp:tab/>`, `<hp:lineBreak/>`로 직렬화함
- 표 셀 경로와 본문 문단 경로의 HWPX inline 표현이 불일치함

원인과 수정:

- `src/serializer/hwpx/table.rs::write_cell_text()`가 `BytesText`로 셀 텍스트를 직접 출력하고 있었다.
- 본문 문단 serializer의 `render_hp_t_content()`는 탭 확장 정보와 소프트 줄바꿈을 이미 HWPX inline element로 변환한다.
- 이번 PR은 `render_hp_t_content()`를 `pub(crate)`로 열고, 표 셀 문단도 같은 helper를 사용하도록 변경했다.
- 표 셀 직렬화에서도 `para.tab_extended`를 전달하여 탭 width/leader/type을 보존한다.

변경 파일:

- `src/serializer/hwpx/section.rs`
- `src/serializer/hwpx/table.rs`
- `mydocs/pr/pr_1360_review.md`

## 3. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL Analyze (javascript-typescript) | pass |
| CodeQL Analyze (python) | pass |
| CodeQL Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `git diff --check origin/devel..HEAD` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib serializer::hwpx` | 통과, 89 passed |
| `cargo test --test issue_1267_hwpx_tab_and_diagonal` | 통과, 2 passed |
| `cargo test --test issue_1244_tab_extended_fallback` | 통과, 3 passed |
| `cargo test --test hwpx_roundtrip_integration` | 통과, 22 passed |
| `cargo clippy -- -D warnings` | 통과 |

## 4. 판정

**수용 가능**.

수정 범위는 HWPX 표 셀 문단 텍스트 직렬화에 한정되어 있고, 기존 본문 문단 serializer helper를 재사용하여 HWPX inline element 처리 경로를 통합했다. 텍스트 escape는 helper 내부에서 수행되며, 탭 확장 정보도 본문 문단과 같은 규약으로 사용한다.

주의점:

- 표 셀 serializer는 여전히 한 문단을 하나의 `hp:run`으로 출력하는 기존 구조를 유지한다. char shape run 분할 같은 더 넓은 HWPX fidelity 문제는 이번 PR 범위가 아니다.
- Issue #1353은 PR 본문에서 자동 close 키워드로 연결되지 않았으므로 수용 후 별도 close가 필요하다.

## 5. 후속 절차

처리 진행:

- [x] 작업지시자 승인
- [x] `local/devel`에 PR 커밋 반영 — `a547909e`
- [x] 최종 로컬 검증
- [x] 문서 정리 커밋 — `1f98b1b6`
- [x] `origin/devel` push — `1f98b1b6`
- [x] PR #1360에 메인테이너 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1360#issuecomment-4671961670
- [x] PR #1360 close — `2026-06-10T15:51:04Z`
- [x] Issue #1353 처리 코멘트 작성 — https://github.com/edwardkim/rhwp/issues/1353#issuecomment-4671967110
- [x] Issue #1353 completed close — `2026-06-10T15:51:52Z`
