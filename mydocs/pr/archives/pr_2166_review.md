# PR #2166 검토 - 표 셀 반복 Enter 캐럿 위치 정정

- PR: https://github.com/edwardkim/rhwp/pull/2166
- 관련 이슈: [#2164](https://github.com/edwardkim/rhwp/issues/2164)
- 작성자: `jangster77`
- base: `devel`
- head: `task_m100_2164`
- 작성일: 2026-07-10
- 사전 검토 시점 참고값: `mergeStateStatus=BLOCKED` (최신 head CI 실행 중)
- merge: `f3712542e980b1eab625624f54eb49e789ddfc5d` (2026-07-10)
- 관련 이슈: GitHub closing keyword로 CLOSED 확인

## 결론

blocking finding 없이 PR #2166은 최신 head CI, CodeQL, Render Diff 성공 뒤 merge commit
`f3712542e980b1eab625624f54eb49e789ddfc5d`로 `devel`에 반영됐다. #2164도 자동 close됐다.

작업지시자 지정 옵션 2에 따라 이 문서, 기준 PDF, visual sweep asset, 오늘할일은 source
PR이 아니라 별도 docs-only fast-pass PR로 보존한다.

## 관련 이슈와 재현

#2164는 Windows Chrome 확장 v0.2.8에서 문단 부호를 켠 표 셀에 텍스트를 입력한 뒤
Enter를 누르면 새 문단과 기존 내용이 겹치고, 반복 Enter에서는 캐럿이 첫 줄로 복귀하는
문제를 다룬다.

재현 원본 `samples/issue2164/의견제출서(양식).hwp`에서 다음 편집 순서를 실제 browser
`cellPath` 상태로 확인했다.

```text
1111 입력 → Enter → 2222 입력 → Enter
```

최종 위치는 `paragraphIndex=2`, `cellParaIndex=2`, 마지막
`cellPath.cellParaIndex=2`, 캐럿 y `445.3px`으로 새 세 번째 문단을 가리킨다.

## 변경 범위

- 셀 문단 분할/병합 뒤 후속 문단 vpos를 다시 연결한다.
- 분할로 새로 생긴 문단의 임시 vpos 원점만 `RowBreak` reset 판정에서 제외하고,
  저장된 실제 reset 경계는 보존한다.
- 셀 구조 편집 뒤 flat 문단 인덱스, 마지막 `cellPath` 인덱스, 캐럿 rect cache를 같은
  새 문단 상태로 갱신한다.
- 실제 제보 원본의 첫 Enter, 반복 Enter, Backspace 뒤 재Enter 문단 y/vpos 회귀를
  `tests/issue_2164_cell_enter_overlap.rs`에 고정한다.
- HWP 2020 기준 PDF, visual sweep, 최종 보고서를 PR diff에 보존한다.

## 검증

- `cargo build --release`: 통과
- `cargo test --release --lib`: 2190 passed, 0 failed, 7 ignored
- `cargo test --profile release-test --tests`: 통과
  - #2164 회귀 3건 통과
  - SVG snapshot 8건 통과
- `cargo fmt --check`, `git diff --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --doc`: 통과 (1 ignored)
- `rhwp-studio`: TypeScript 검사, 183개 테스트, production build 통과
- `rhwp-chrome`: production build 통과
- `wasm-pack build --target web --out-dir pkg`: 통과

## 시각 검증

- HWP 2020 기준 PDF: `pdf/issue2164/의견제출서(양식)-2020.pdf`
- visual sweep: SVG/PDF 1/1쪽, overflow/overlap/question flow/line order 후보 0건
- 원시 sweep 증적: `mydocs/report/assets/task_m100_2164_visual_sweep/issue2164/review/review_001.png`
- PR 안정 asset: `mydocs/pr/assets/pr_2166_issue2164_review.png`

한컴 전용 글꼴 metrics 차이로 ink match는 낮다. 이는 정적 font fidelity의 잔여이며,
이번 PR의 편집 중 문단 생성·캐럿 위치 정정과 혼동하지 않는다. 동적 동작은 실제
브라우저와 작업지시자 화면에서 확인했다.

## 처리 결과

1. 최신 head `9e7381e2` 기준 CI, CodeQL, Render Diff가 모두 success였다.
2. PR은 `MERGEABLE/CLEAN` 상태에서 작업지시자 지시로 merge됐다.
3. 이 문서와 실행 기록은 옵션 2 docs-only PR에서 최종 보존한다.
