# Task M100 #2164 Stage 3 완료보고서

- 이슈: #2164
- 브랜치: `codex/task_m100_2164`
- 재현 원본: `samples/issue2164/의견제출서(양식).hwp`
- 작성일: 2026-07-10

## 1. 잔존 증상

Stage 2 뒤 문단부호와 빈 문단 자체는 정상 위치에 생겼지만, 실제 편집기에서 `1111`,
Enter, `2222`, Enter를 수행하면 검은 캐럿이 다시 첫 번째 줄의 맨 앞으로 이동했다.
즉 문서 모델의 첫 분할만으로는 재현되지 않는, 반복 구조 편집의 문단 위치와 프론트
커서 경로가 함께 어긋나는 문제였다.

## 2. 원인

### 셀 경로와 flat 위치 불일치

`SplitParagraphInCellCommand`와 `MergeParagraphInCellCommand`가 반환한 위치는
`cellParaIndex`만 갱신했다. 하지만 `CursorState`는 `cellPath`가 있으면 path 기반
rect를 우선 조회한다. 따라서 path의 마지막 `cellParaIndex`와 flat
`paragraphIndex`가 이전 문단을 계속 가리켜 새 문단의 캐럿 위치를 찾지 못했다.

### 두 번째 Enter의 vpos reset 오인

새 문단은 분할 직후 임시 vpos 원점 `0`으로 생성된다. Stage 2의 재계산 helper는 이
값을 저장된 `RowBreak` 원점 reset으로 오인해 재계산을 중단했다. 그 결과 두 번째
Enter로 생긴 세 번째 문단이 첫 번째 문단의 y로 남았다.

## 3. 수정

- `rhwp-studio/src/engine/command.ts`
  - 셀 문단 구조를 바꾼 뒤 반환하는 `DocumentPosition`의 `paragraphIndex`,
    `cellParaIndex`, 마지막 `cellPath.cellParaIndex`를 함께 갱신한다.
  - 기존 `cursorRect` 캐시를 비워 새 path 기준 좌표를 다시 조회한다.
- `src/document_core/commands/text_editing.rs`
  - 셀 문단 vpos 재계산에서 방금 분할로 삽입된 문단까지의 임시 원점 reset은
    `RowBreak` 경계로 취급하지 않는다.
  - 그 뒤에 저장된 실제 reset 경계는 기존처럼 보존한다.
- `tests/issue_2164_cell_enter_overlap.rs`
  - 실제 제보 원본에서 `1111`, Enter, `2222`, Enter를 수행해 네 문단의 vpos와
    캐럿 y가 모두 단조 증가하는 회귀 계약을 추가했다.

## 4. WASM·브라우저 기본 검증

- `wasm-pack build --target web --out-dir pkg`: 통과
- 기존 `http://localhost:7700`에서 실제 `cellPath`를 가진 대상 셀에 다음을 수행했다.
  1. `1111` 입력
  2. Enter
  3. `2222` 입력
  4. Enter
- 두 번째 Enter 직후 반환 위치와 캐럿 좌표:

```text
cellParaIndex = 2
paragraphIndex = 2
cached/path cursor y = 445.3px
flat cursor y = 446.2px
DOM caret top = 455.3px
```

세 번째 문단의 y가 첫 번째 문단 y(약 390px)가 아니라 아래 줄(약 445px)으로
갱신되어, 재현된 첫 줄 복귀 문제가 해소됐다. 작업지시자도 실제 화면에서 동작을
검증했다.

## 5. 검증 범위와 PR 조건

초기 구현 단계에서는 작업지시자 지시에 따라 WASM build와 실제 브라우저 기본 재현만
실행했다. 이후 PR 준비 승인 뒤 전체 사전 검증을 수행했다.

- `cargo build --release`: 통과
- `cargo test --release --lib`: 2190 passed, 0 failed, 7 ignored
- `cargo test --profile release-test --tests`: 통과
  - `issue_2164_cell_enter_overlap`: 3 passed
  - `svg_snapshot`: 8 passed
- `cargo fmt --check`, `git diff --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --doc`: 0 passed, 1 ignored
- `rhwp-studio`: TypeScript 검사, 183개 테스트, production build 통과
- `rhwp-chrome`: production build 통과
- 최종 `wasm-pack build --target web --out-dir pkg`: 통과

기준 PDF와 visual sweep 결과, 정적 글꼴 fidelity 잔여는 최종 보고서에 기록했다.
