# PR #1482 리뷰 기록 - 표 줄/칸 입력·지우기 회귀 보정

- PR: https://github.com/edwardkim/rhwp/pull/1482
- 작성일: 2026-06-23
- 경로: collaborator self-merge 후보
- base/head: `devel` <- `task_m100_1481`
- 문서 작성 시점 참고 head: `8f1716a9` (본 review 문서/최종 보정 커밋 전)

## 1. PR 메타

| 항목 | 확인 내용 |
|------|-----------|
| 작성자 | `jangster77` |
| PR 상태 | 문서 작성 시점 참고값: open, draft |
| base | `devel` |
| 관련 이슈 | `Closes #1481` |
| merge 상태 | 문서 작성 시점 참고값: `BLOCKED`, 최종 merge 전 최신 상태 재확인 필요 |
| CI 상태 | 기존 원격 head 기준 상태는 폐기, force-with-lease push 후 최신 head 기준 재확인 필요 |

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 확정 사실로 기록하지 않는다.
최종 merge 판단은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인 후에만 수행한다.

## 2. 변경 범위

### 2.1 표 줄/칸 추가·삭제 기능

- 한컴 메뉴 기준의 `줄/칸 추가하기(I)...`, `줄/칸 지우기(E)...` 대표 대화상자를 추가했다.
- 줄/칸 추가 방향 4종과 개수 1~63 반복 적용을 연결했다.
- 줄/칸 지우기는 `줄`/`칸` 삭제 대상을 선택하도록 했다.
- 상단 표 메뉴와 표 셀 컨텍스트 메뉴 표기를 대표 항목 중심으로 정리했다.

### 2.2 표 높이 축소 회귀 보정

- `Table::insert_column()` / `Table::delete_column()`에서 일반 표 외곽 height를 보존한다.
- `Table::insert_row()` / `Table::delete_row()` 및 하단선 resize 뒤에도 표시 height가 셀 저장 height 합으로 붕괴하지 않도록 보정했다.
- 표 구조 편집 뒤 `common.height`, raw ctrl data, 렌더 표시 height가 일관되도록 회귀 테스트를 추가했다.

### 2.3 단축키와 대화상자 반복 입력

- macOS/Windows 물리 입력에서 `Alt/Option+Insert`가 안정적으로 전달되지 않는 점을 반영해 줄/칸 추가 단축키를 `Alt/Option+Enter`로 통일했다.
- 줄/칸 지우기는 `Alt/Option+Delete`를 유지한다.
- #1477의 플랫폼별 표시 규칙에 맞춰 macOS는 `⌥Enter`/`⌥Delete`, Windows/Linux는 `Alt+Enter`/`Alt+Delete`로 표시한다.
- 대화상자 닫힘 뒤 편집 textarea 포커스를 복원해 `Option+Enter -> Esc -> Option+Enter`, `Option+Delete -> Esc -> Option+Delete` 반복 입력이 끊기지 않도록 했다.

### 2.4 표 생성 직후 탈출과 빈줄 렌더 회귀 보정

- 자리차지 표 host 문단 시작 위치에서 `Enter`가 들어오면 표 위가 아니라 표 아래 빈 문단으로 이동하도록 보정했다.
- 마지막 셀에서 `Tab`을 누르면 한컴처럼 자동으로 아래 줄을 추가하고 새 줄 첫 셀로 이동하도록 보정했다.
- 일반 빈 문단과 `blank2010.hwp` 템플릿의 SectionDef/ColumnDef 구조 컨트롤만 있는 빈 문단에서 표를 만들 때, 생성 경로의 빈 줄이 표 위에 남지 않도록 보정했다.
- 첫 셀에서 위/왼쪽 이동 시 표 밖 첫 조판부호 위치로 나갈 수 있는 회귀 테스트를 추가했다.
- `cargo clippy --all-targets -- -D warnings`에서 발견된 `cursor_nav.rs`의 `needless_return`을 의미 변경 없이 정리했다.

## 3. 리스크

| 리스크 | 판단 |
|--------|------|
| 구조 편집 후 표 높이 재축소 | `issue_1481` Rust 회귀 테스트로 열/행 추가·삭제, 하단선 resize 경로를 고정했다. |
| 표 생성 직후 위쪽 빈줄 잔존 | `create_empty()`와 `blank2010.hwp` 템플릿 기반 생성 경로를 모두 테스트했다. |
| 표 안에서 탈출 불가 | 표 앞 조판부호 이동, 표 앞 Enter, 마지막 셀 Tab 자동 줄 추가 테스트를 추가했다. |
| 단축키 표시/실행 불일치 | shortcut-map, navigation-keymap, menu-shortcut-labels 및 전체 frontend test로 확인했다. |
| 최신 `devel` 반영 | 이전 PR review 문서 전용 커밋을 제거하고 `upstream/devel` 기준 rebase를 다시 수행했다. |

## 4. 로컬 검증

PR 재작성 전 전체 로컬 검증을 다시 수행했다.

```bash
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
git diff --check
cargo test --test svg_snapshot
cargo clippy --all-targets -- -D warnings
cargo test --doc
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
```

검증 결과:

- `cargo build --release`: 통과
- `cargo test --release --lib`: 통과 (1923 passed, 6 ignored)
- `cargo test --profile release-test --tests`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `cargo test --test svg_snapshot`: 통과 (8 passed)
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --doc`: 통과 (0 passed, 1 ignored)
- `cd rhwp-studio && npx tsc --noEmit`: 통과
- `cd rhwp-studio && npm test`: 통과 (124 passed)
- `wasm-pack build --target web --out-dir pkg`: 통과

## 5. 판단

PR #1482는 #1481의 표 줄/칸 편집 회귀, 한컴식 줄/칸 대표 대화상자, 표 높이 축소 회귀, 표 생성 직후 탈출/빈줄 회귀를 함께 보정한다.

최종 merge 조건:

1. 본 review 문서 2건과 오늘할일 문서가 PR diff에 포함된다.
2. 이전 PR review 문서 전용 커밋이 PR head 히스토리에서 제거된다.
3. force-with-lease push 후 PR head 최신 커밋 기준 GitHub Actions가 통과한다.
4. 작업지시자 승인 상태가 유지된다.

위 조건 충족 시 collaborator self-merge 후보로 merge 수용 가능하다.
