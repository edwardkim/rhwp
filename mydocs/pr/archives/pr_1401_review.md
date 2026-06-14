# PR #1401 검토 — HWPX 셀 필드 이름 파싱 복원

- PR: https://github.com/edwardkim/rhwp/pull/1401
- 제목: `fix(hwpx): hp:tc name(셀 필드 이름) 파싱 누락 — HWPX 로드 시 셀 필드 유실 (#493 부분)`
- 작성일: 2026-06-14
- 작성자: `oksure`
- 관련 이슈: #493
- base: `devel`
- head: `oksure:contrib/fix-493-hwpx-cell-field-name` (`88b8da796ebee2872c9adec676efeeb86aab4341`)
- 검토 브랜치: `review/pr-1401`

## 1. 요약 판단

**수용 가능**으로 판단한다.

PR은 HWPX 파서가 `hp:tc`의 `name` 속성을 읽지 않아 셀 필드 이름이 로드/라운드트립에서
유실되던 문제를 수정한다. 기존 HWPX serializer는 `Cell::field_name`을 `hp:tc name`으로
방출하고 있었으므로, 이번 변경은 파서와 serializer의 비대칭을 줄이는 좁고 적절한 보완이다.

`name=""`는 `None`으로 유지해 무명 셀을 `Some("")`로 오염시키지 않는다. 이 처리는 기존
HWP5 셀 필드 이름 파서의 의미와도 맞다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `MERGEABLE` |
| mergeStateStatus | `CLEAN` |
| 변경량 | 2 files, +94 / -0 |
| 작성자 | `oksure` |
| 관련 이슈 | `Related #493` — 부분 해결, close하지 않음 |

커밋:

- `c9eae174` — `fix(hwpx): hp:tc name(셀 필드 이름) 파싱 누락 — HWPX 로드 시 셀 필드 유실 (#493)`
- `2a21fa3c` — `test: 리뷰 반영 — 셀 수 사전 단언 + 빈 name 명시적 None 대입 (#493)`
- `79ff236e` — `Merge branch 'devel' into contrib/fix-493-hwpx-cell-field-name`
- `88b8da79` — `Merge branch 'devel' into contrib/fix-493-hwpx-cell-field-name`

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

## 3. 변경 검토

### 3.1 코드 변경

`src/parser/hwpx/section.rs`:

- `parse_table_cell()`에서 `hp:tc`의 `name` 속성을 파싱한다.
- 빈 문자열이면 `cell.field_name = None`, 비어 있지 않으면 `Some(v)`로 보존한다.
- 기존 `Cell::default()`와 `field_name: None` 초기값에 의존하지 않고 `name=""` 분기를 명시한다.

`tests/issue_493_hwpx_cell_field_name.rs`:

- `samples/hwpx/basic-table-01.hwpx`를 파싱한다.
- 첫 번째 표의 첫 셀에 `field_name = Some("사업명칭")`을 넣고 두 번째 셀은 `None`으로 둔다.
- `serialize_hwpx()` 후 `parse_hwpx()`로 재파싱해 이름 있는 셀의 보존과 무명 셀의 `None` 유지를 확인한다.

### 3.2 기존 계약과의 정합

- `src/serializer/hwpx/table.rs`의 `write_cell()`은 이미 `cell.field_name.as_deref().unwrap_or("")`를
  `hp:tc name`으로 방출한다.
- 이번 변경은 serializer가 내보내는 값을 parser가 다시 읽도록 맞춘다.
- PR 본문처럼 셀 보호 `protect` 모델링은 별도 작업 범위로 남겨 두는 것이 적절하다.

## 4. 로컬 검증

검토 브랜치: `review/pr-1401`

| 명령 | 결과 |
|---|---|
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo test --test issue_493_hwpx_cell_field_name` | 통과, 1 passed |
| `cargo fmt --check` | 통과 |
| `cargo build --release` | 통과, 2m 19s |
| `cargo test --release --lib` | 통과, 1752 passed / 0 failed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과, 1m 12s |

`wasm-pack` 산출물 `pkg/`는 ignored 검증 산출물이므로 커밋 대상에 포함하지 않는다.

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| 빈 `name=""`가 필드로 노출되는 문제 | 낮음 | 명시적으로 `None` 처리 |
| HWP5/HWPX 필드 이름 의미 차이 | 낮음 | 기존 HWP5 파서도 빈 이름은 `None` |
| #493 전체 해결 오해 | 낮음 | PR 본문이 부분 해결 및 `Related #493`로 명시 |

## 6. 권고

로컬 검증 기준으로는 merge 가능하다.

머지 전 마지막 확인:

- PR #1401 head에 이 검토 문서와 오늘할일 갱신 커밋을 push
- 문서 커밋 push 후 GitHub Actions 전체 통과 확인
- #493은 부분 해결이므로 merge 후 자동 close하지 않는다
