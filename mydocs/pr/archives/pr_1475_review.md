# PR #1475 리뷰 기록 - 스타일 적용과 표 캡션/모양복사 보정

- PR: https://github.com/edwardkim/rhwp/pull/1475
- 작성일: 2026-06-22
- 작성자: collaborator self-merge 후보 경로
- 문서 작성 시점 참고 head: `71df3b8b7b4107d19c1e27dee243dfc97f57fa52`
- base: `devel`
- head: `task_m100_1470`

## 1. PR 메타

| 항목 | 확인 내용 |
|------|-----------|
| 작성자 | `jangster77` |
| PR 상태 | open, draft 아님 |
| merge 상태 | 문서 작성 시점 `MERGEABLE`, `BLOCKED` |
| 관련 이슈 | `Closes #1470` |
| 규모 | 문서 작성 시점 32 files, +2271 / -178 |
| 커밋 수 | 7개 + 본 self-merge review 문서 커밋 예정 |

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 이 문서는 작성 시점 값을 참고로만 기록한다.
최종 merge 판단은 merge 직전 최신 PR head 기준으로 다시 확인한다.

## 2. 변경 범위

### 2.1 스타일 적용/편집 정합화

- 스타일의 왼쪽 여백/첫 줄/오른쪽 여백 값을 직접 문단 모양 적용 경로와 같은 단위로 저장·조회하도록 보정했다.
- 스타일 줄간격 변경 뒤 기존 `LineSeg`/페이지네이션이 남지 않도록 스타일 적용 문단을 재흐름 대상으로 고정했다.
- 스타일 적용/편집 시 문단의 직접 글자 서식을 불필요하게 초기화하지 않도록 보정했다.
- 선택 블록 대상 문단에 스타일을 순회 적용하는 경로를 추가했다.

### 2.2 표 생성과 TAC 표 렌더 보정

- Studio 표 만들기 상세 옵션의 너비/높이/글자처럼 취급 값을 `createTableEx` 경로까지 전달한다.
- 표 생성 직후 표가 현재 위치가 아니라 다음 페이지로 밀리는 흐름을 줄이기 위해 표 크기/배치 정보를 실제 생성 옵션에 반영했다.
- TAC 표가 캡션과 함께 있을 때 같은 표 컨트롤이 중복 렌더되는 경로를 보정했다.

### 2.3 표 캡션 자동 번호 보정

- 표 캡션 생성 시 literal 숫자만 남기지 않고 `표  ` 접두와 AutoNumber 앵커 구조를 함께 생성한다.
- 표 캡션 생성/수정/삭제 뒤 남은 표 캡션 번호를 1부터 재배정한다.
- 중간 표 캡션 삭제 뒤 stale 번호가 남지 않도록 회귀 테스트를 추가했다.
- 그림 캡션의 기존 `그림 1`, `그림 2` 흐름은 유지한다.

### 2.4 모양 붙여넣기 회귀 복구

- PR #1446에서 추가됐던 `edit:format-paste` 명시 커맨드 경로를 복구했다.
- `EditorContext.hasCopiedFormat`, `InputHandler.hasCopiedFormat()`, `InputHandler.performFormatPaste()`를 복구했다.
- 편집 메뉴와 기본/표 셀 컨텍스트 메뉴에 `모양 붙여넣기`를 다시 노출했다.
- 기존 `Alt+C` 일회성 모양복사 동작과 툴바 `모양 복사` 버튼은 유지했다.
- `format-paste-availability.ts`와 `format-paste-command.test.ts`를 추가해 명시적 모양 붙여넣기 경로가 다시 제거되지 않도록 고정했다.

## 3. 리스크

| 리스크 | 판단 |
|--------|------|
| 스타일/문단 모양 공통 경로 변경 | `issue_1470` focused 테스트와 전체 `cargo test --release --lib`로 문단 모양 적용 회귀를 확인했다. |
| 표 캡션 AutoNumber 구조 변경 | 표/그림 캡션 prefix와 삭제 후 재정렬 회귀 테스트를 추가해 번호 구조를 고정했다. |
| 렌더링/페이지네이션 영향 | `cargo test --profile release-test --tests`, `wasm-pack build`, 사용자 수동 검증으로 표 생성 위치와 TAC 중복 렌더를 확인했다. |
| Studio 명령/메뉴 회귀 | `npx tsc --noEmit`, `npm test` 110 passed, 신규 `format-paste-command.test.ts`로 커맨드/메뉴/컨텍스트 경로를 고정했다. |
| PR 규모 | 32 files, +2271 / -178로 중간 규모지만 대부분 단계 문서와 focused 회귀 테스트가 포함된 변경이다. |

## 4. 검증

로컬 검증:

```bash
cargo fmt && cargo test --release issue_1470_table_caption --lib
cargo test --release issue_1470 --lib
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
git diff --check
```

작업지시자 수동 검증:

- 스타일 왼쪽 여백 15 입력값이 30으로 변하지 않는지 확인했다.
- 스타일 줄간격이 실제 화면 레이아웃에 반영되는지 확인했다.
- 표가 현재 위치에 생성되고 TAC 표가 중복 렌더되지 않는지 확인했다.
- 표 캡션이 `표 1`, `표 2`, `표 3` 형태로 보이고, 삭제 뒤 번호가 재정렬되는지 확인했다.
- 모양 붙여넣기 명시 메뉴/컨텍스트 메뉴 경로가 동작하는지 확인했다.

GitHub Actions 작성 시점 참고값:

- Build & Test: in progress
- Canvas visual diff: in progress
- CodeQL: in progress
- WASM Build: skipped

본 review 문서 커밋 push 후 GitHub Actions가 다시 실행되므로, merge 전 최신 head 기준으로 위 상태를 재확인한다.

## 5. 판단

작성 시점 기준으로 #1470의 주요 피드백인 스타일 여백 배율, 스타일 줄간격 반영, 표 생성 위치, 표 캡션 prefix/번호 재정렬, PR #1446 모양 붙여넣기 회귀가 모두 PR 범위에 포함되어 있다.

최종 조건:

1. 본 review 문서 2건과 오늘할일 문서가 PR head에 포함된다.
2. push 후 최신 PR head 기준 GitHub Actions가 통과한다.
3. 작업지시자 승인 상태가 유지된다.

위 조건 충족 시 collaborator self-merge 후보로 merge 수용한다.
